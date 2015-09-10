#![allow(dead_code, unused_variables, unused_imports)]

use {Async, Stream, Sender, AsyncResult, AsyncError};
use syncbox::ArrayQueue;
use std::{mem, ops};
use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const MAX_IN_FLIGHT: usize = (1 << 16) - 1;

pub fn process<T, U, F>(source: Stream<T, U::Error>, in_flight: usize, action: F) -> Stream<U::Value, U::Error>
        where T: Send + 'static,
              U: Async,
              F: FnMut(T) -> U + Send + 'static {

    // New stream
    let (tx, rx) = Stream::pair();

    // Wait for consumer interest
    if in_flight > 0 {
        tx.receive(move |res| {
            if let Ok(tx) = res {
                setup(source, in_flight, action, tx);
            }
        });
    }

    rx
}

fn setup<T, U, F>(source: Stream<T, U::Error>, in_flight: usize, action: F, dest: Sender<U::Value, U::Error>)
        where T: Send + 'static,
              U: Async,
              F: FnMut(T) -> U + Send + 'static {

    let mut inner = Inner::new(source, in_flight, action, dest);
    inner.maybe_process_next(false);
}

struct Core<T: Send + 'static, U: Async, F> {
    max: usize,
    queue: ArrayQueue<AsyncResult<U::Value, U::Error>>,
    sender: Option<Sender<U::Value, U::Error>>,
    source: Option<Source<T, U, F>>,
    consume_state: AtomicState,
    produce_state: AtomicState,
}

struct Source<T: Send + 'static, U: Async, F> {
    stream: Stream<T, U::Error>,
    action: F,
}

struct Inner<T: Send + 'static, U: Async, F>(Arc<UnsafeCell<Core<T, U, F>>>);

impl<T: Send + 'static, U: Async, F: FnMut(T) -> U + Send + 'static> Inner<T, U, F> {
    fn new(source: Stream<T, U::Error>,
           in_flight: usize,
           action: F,
           dest: Sender<U::Value, U::Error>) -> Inner<T, U, F> {

        let core = Core {
            max: in_flight,
            queue: ArrayQueue::with_capacity(in_flight),
            source: Some(Source { stream: source, action: action }),
            sender: Some(dest),
            consume_state: AtomicState::new(),
            produce_state: AtomicState::new(),
        };

        Inner(Arc::new(UnsafeCell::new(core)))
    }

    fn maybe_process_next(&mut self, dec_in_flight: bool) {
        // Attempt to acquire the consume lock and increment the number of
        // in-flight queries. An acquire ordering is used to ensure that the
        // source value is readable.
        if self.try_acquire_consume_lock(dec_in_flight, Ordering::Acquire) {
            // Access to the source has been acquired
            let Source { stream, mut action } = self.source.take().expect("source should be present");
            let mut inner = self.clone();

            // Wait for the next value to be provided
            stream.receive(move |res| {
                match res {
                    Ok(Some((val, rest))) => {
                        // Process the value and wait for the result
                        let val = action(val);

                        // Return the source. Release ordering is used to flush
                        // the memory so that another thread may access it.
                        inner.source = Some(Source { stream: rest, action: action });
                        inner.consume_state.release_lock(Ordering::Release);

                        let mut inner2 = inner.clone();

                        // Wait for the result
                        val.receive(move |res| {
                            match res {
                                Ok(val) => {
                                    inner2.receive_value(Ok(val));
                                }
                                Err(err) => {
                                    inner2.receive_value(Err(err));
                                }
                            }
                        });

                        // Since the results are buffered, attempt to read
                        // another value immediately
                        inner.maybe_process_next(false);
                    }
                    Ok(None) => {}
                    Err(AsyncError::Failed(_)) => {
                        unimplemented!();
                    }
                    _ => unimplemented!(),
                }
            });
        }
    }

    fn receive_value(&mut self, val: AsyncResult<U::Value, U::Error>) {
        // Push the value onto the queue
        self.queue.push(val).ok()
            .expect("value queue should never run out of capacity");

        // Increment the number of values pending in the queue
        //
        // TODO: if the stream has failed, discard the value by popping from
        // the queue
        if self.acquire_produce_lock_or_inc_in_flight(Ordering::Acquire) {
            // Produce lock has been acquired
            self.send_value();
        }
    }

    fn send_value(&mut self) {
        let sender = self.sender.take().expect("expected sender to be sender");
        let value = self.queue.pop().expect("expected value to be in queue");

        match value {
            Ok(value) => {
                let mut inner = self.clone();

                sender.send(value).receive(move |res| {
                    match res {
                        Ok(sender) => {
                            // Save off the sender in case the lock is released
                            inner.sender = Some(sender);

                            if inner.release_lock_if_idle(Ordering::Release) {
                                // in-flight has been decremented, also, even though
                                // the previous memory fence is a release, the only
                                // memory that is accessed has been set previously in
                                // the current thread
                                inner.send_value();
                            }
                        }
                        Err(_) => {
                            // TODO: Transition to a canceled state and discard
                            // currently pending values
                        }
                    }
                });
            }
            Err(AsyncError::Failed(e)) => sender.fail(e),
            Err(AsyncError::Aborted) => sender.abort(),
        }

        // Will decrement the consumer in-flight and potentially start
        // processing another value
        self.maybe_process_next(true);
    }

    fn try_acquire_consume_lock(&self, dec_in_flight: bool, order: Ordering) -> bool {
        let mut old = self.consume_state.load(Ordering::Relaxed);

        loop {
            // If the consume lock has already been acquired and in-flight is
            // not being decremented, then the state does not need to
            // transition. Nothing else to do, the lock has not been acquired.
            if (old.has_lock() || old.in_flight() == self.max) && !dec_in_flight {
                return false;
            }

            let new = if old.has_lock() {
                debug_assert!(dec_in_flight, "state transition bug");
                // The lock coul dnot be acquired, but the num in-flight must
                // be decremented
                old.dec_in_flight()
            } else {
                debug_assert!(old.in_flight() < self.max || dec_in_flight, "state transition bug");
                if dec_in_flight {
                    old.with_lock()
                } else {
                    old.inc_in_flight().with_lock()
                }
            };

            let act = self.consume_state.compare_and_swap(old, new, order);

            if act == old {
                return !old.has_lock();
            }

            old = act;
        }
    }

    fn try_acquire_produce_lock(&self, order: Ordering) -> bool {
        let mut old = self.produce_state.load(order);

        loop {
            if old.has_lock() {
                return false;
            }

            let act = self.produce_state.compare_and_swap(old, old.with_lock(), order);

            if act == old {
                return true;
            }

            old = act
        }
    }

    // Increment the in-flight counter and attempt to acquire the produce lock
    fn acquire_produce_lock_or_inc_in_flight(&self, order: Ordering) -> bool {
        let mut old = self.produce_state.load(Ordering::Relaxed);

        loop {
            let new = if old.has_lock() {
                old.inc_in_flight()
            } else {
                old.with_lock()
            };

            let act = self.produce_state.compare_and_swap(old, new, order);

            if act == old {
                return !old.has_lock();
            }

            old = act;
        }
    }

    fn release_lock_if_idle(&self, order: Ordering) -> bool {
        let mut old = self.produce_state.load(Ordering::Relaxed);

        loop {
            let new = if old.in_flight() > 0 {
                old.dec_in_flight()
            } else {
                old.without_lock()
            };

            let act = self.produce_state.compare_and_swap(old, new, order);

            if act == old {
                return new.has_lock();
            }

            old = act;
        }
    }
}

impl<T: Send + 'static, U: Async, F> ops::Deref for Inner<T, U, F> {
    type Target = Core<T, U, F>;

    fn deref(&self) -> &Core<T, U, F> {
        unsafe { mem::transmute(self.0.get()) }
    }
}

impl<T: Send + 'static, U: Async, F> ops::DerefMut for Inner<T, U, F> {
    fn deref_mut(&mut self) -> &mut Core<T, U, F> {
        unsafe { mem::transmute(self.0.get()) }
    }
}

impl<T: Send + 'static, U: Async, F> Clone for Inner<T, U, F> {
    fn clone(&self) -> Inner<T, U, F> {
        Inner(self.0.clone())
    }
}

unsafe impl<T: Send, U: Async, F> Send for Inner<T, U, F> { }
unsafe impl<T: Send, U: Async, F> Sync for Inner<T, U, F> { }

const LOCK: usize = 1 << 31;

#[derive(Copy, Clone, Eq, PartialEq)]
struct State(usize);

impl State {
    fn in_flight(&self) -> usize {
        self.0 & MAX_IN_FLIGHT
    }

    fn inc_in_flight(&self) -> State {
        assert!(self.in_flight() < MAX_IN_FLIGHT);
        State(self.0 + 1)
    }

    fn dec_in_flight(&self) -> State {
        assert!(self.in_flight() > 0);
        State(self.0 - 1)
    }

    fn has_lock(&self) -> bool {
        self.0 & LOCK == LOCK
    }

    fn with_lock(&self) -> State {
        State(self.0 | LOCK)
    }

    fn without_lock(&self) -> State {
        State(!LOCK & self.0)
    }
}

struct AtomicState {
    atomic: AtomicUsize,
}

impl AtomicState {
    fn new() -> AtomicState {
        AtomicState { atomic: AtomicUsize::new(0) }
    }

    fn load(&self, order: Ordering) -> State {
        let val = self.atomic.load(order);
        State(val)
    }

    fn compare_and_swap(&self, old: State, new: State, order: Ordering) -> State {
        let val = self.atomic.compare_and_swap(old.0, new.0, order);
        State(val)
    }

    fn release_lock(&self, order: Ordering) {
        self.atomic.fetch_sub(LOCK, order);
    }
}
