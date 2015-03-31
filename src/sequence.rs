use {Async, AsyncError, Stream, Sender};
use std::{mem, ops};
use std::cell::UnsafeCell;
use std::iter::IntoIterator;
use std::sync::Arc;
use std::sync::atomic::{self, AtomicUsize, Ordering};

/// Returns a `Stream` consisting of the completion of the supplied async
/// values in the order that they are completed.
pub fn sequence<I, A>(asyncs: I) -> Stream<A::Value, A::Error>
        where I: IntoIterator<Item=A> + Send,
              A: Async {

    // Create a stream pair
    let (tx, rx) = Stream::pair();

    tx.receive(move |res| {
        debug!("sequence() - sequence consumer ready; res={:?}", res);
        if let Ok(tx) = res {
            setup(asyncs, tx);
        }
    });

    rx
}

// == !!! Warning !!! ==
//
// The code below uses an UnsafeCell to by-pass rust's memory model with
// respect to concurrency. In order to implement `sequence` in a lock-free
// fashion, it is required to be able to access a shared data structure from
// multiple threads. However, assuming no bugs, the cross thread memory access
// should be safe as it is coordinated via atomic variables and fences.

fn setup<I, A>(asyncs: I, sender: Sender<A::Value, A::Error>)
        where I: IntoIterator<Item=A>,
              A: Async {

    // Collect async values into a vec, the vec will be used later for storage
    let vec: Vec<Option<A>> = asyncs.into_iter()
        .map(|a| Some(a))
        .collect();

    let inner = Inner::new(vec, sender);

    // Register callbacks
    for i in 0..inner.queue.len() {
        let mut inner = inner.clone();
        let async = inner.queue[i].take()
            .expect("expected a value to be present");

        // Because a vec of Option<Async> is used with "safe" APIs, when
        // putting / taking from this vec across threads, a consistent snapshot
        // of the value must be seen. Aka, the fact that we called take() must
        // be visible when the ready callback is invoked.
        //
        // At some point, it may be worth to migrate this code to use unsafe
        // access to the vec such that writes to a memory slot don't "drop" the
        // previous value. This would reduce the required number of memory
        // fences.
        atomic::fence(Ordering::Release);

        debug!("setup() - async.ready callback; i={}", i);

        // Wait for the async computation to complete.
        async.ready(move |async| {
            debug!("setup() -  async is ready; i={}", i);
            inner.ready(async);
        });
    }
}

const IDLE: usize = 0; // Consumer is ready, but nothing is currently happening
const BUSY: usize = 1; // Consumer is busy, cannot send another value yet
const SEND: usize = 2; // A thread is currently sending a value
const FAIL: usize = 3; // The sender has failed
const DROP: usize = 4; // The consumer is no longer interested in values

struct Core<A: Async> {
    queue: Vec<Option<A>>,
    next: AtomicUsize,    // Next index to send to consumer
    ready: AtomicUsize,   // The number of ready async values
    state: AtomicUsize,   // The sender state
    enqueue: AtomicUsize, // The position at which to enqueue the value
    sender: Option<Sender<A::Value, A::Error>>,
}

struct Inner<A: Async>(Arc<UnsafeCell<Core<A>>>);

impl<A: Async> Inner<A> {
    fn new(queue: Vec<Option<A>>, sender: Sender<A::Value, A::Error>) -> Inner<A> {
        let core = Core {
            queue: queue,
            next: AtomicUsize::new(0),
            ready: AtomicUsize::new(0),
            state: AtomicUsize::new(IDLE),
            enqueue: AtomicUsize::new(0),
            sender: Some(sender),
        };

        Inner(Arc::new(UnsafeCell::new(core)))
    }

    fn ready(&mut self, async: A) {
        // First, enqueue the value
        self.enqueue(async);

        // Attempt to acquire the "send" lock. A relaxed ordering can be used
        // since we will only ever attempt to read the value that was just
        // written by the current thread or values written before, so the
        // previous Acquire fence is sufficient.
        let curr = self.state.compare_and_swap(IDLE, SEND, Ordering::Relaxed);

        debug!("Inner::ready() - current-state={}", curr);

        if IDLE == curr {
            self.send();
        }
    }

    fn send(&mut self) {
        debug!("Inner::send(); sending value");

        // The lock has been acquired, now send the next value
        let sender = self.sender.take().expect("expected the stream sender to be present");

        let i = self.next.fetch_add(1, Ordering::Acquire);
        let async = self.queue[i].take().expect("expected an async value to be present");

        match async.expect() {
            Ok(val) => {
                let mut inner = self.clone();

                self.state.swap(BUSY, Ordering::Release);
                sender.send(val).receive(move |res| {
                    debug!("Inner::send() - sender ready; res={:?}; self={:p}", res, self);
                    match res {
                        Ok(sender) => {
                            inner.send_ready(sender, i);
                        }
                        Err(_) => {
                            inner.state.swap(DROP, Ordering::Relaxed);
                        }
                    }
                });
            }
            Err(e) => {
                self.state.swap(FAIL, Ordering::Release);

                // Annoying but currently needed
                match e {
                    AsyncError::Failed(e) => sender.fail(e),
                    _ => sender.abort(),
                }
            }
        }
    }

    fn send_ready(&mut self, sender: Sender<A::Value, A::Error>, prev: usize) {
        self.sender = Some(sender);

        // Changing the state to IDLE must happen here in order to prevent a
        // race condition with another thread enqueuing an async value,
        // checking the state and seeing BUSY but the current thread has left
        // the critical section.
        self.state.swap(IDLE, Ordering::Release);

        let ready = self.ready.load(Ordering::Relaxed);

        debug!("Inner::send_ready; prev={}; ready={}", prev, ready);

        if prev + 1 < ready {
            if IDLE == self.state.compare_and_swap(IDLE, SEND, Ordering::Relaxed) {
                self.send();
            }
        }
    }

    fn enqueue(&mut self, async: A) {
        let i = self.enqueue.fetch_add(1, Ordering::Acquire);

        debug!("Inner::enqueue(); i={}", i);

        self.queue[i] = Some(async);

        // Busy loop until any concurrent enqueues catch up
        loop {
            let j = self.ready.load(Ordering::Relaxed);

            if j != i {
                continue;
            }

            if j == self.ready.compare_and_swap(j, j + 1, Ordering::Release) {
                break;
            }
        }
    }
}

impl<A: Async> ops::Deref for Inner<A> {
    type Target = Core<A>;

    fn deref(&self) -> &Core<A> {
        unsafe { mem::transmute(self.0.get()) }
    }
}

impl<A: Async> ops::DerefMut for Inner<A> {
    fn deref_mut(&mut self) -> &mut Core<A> {
        unsafe { mem::transmute(self.0.get()) }
    }
}

impl<A: Async> Clone for Inner<A> {
    fn clone(&self) -> Inner<A> {
        Inner(self.0.clone())
    }
}

unsafe impl<A> Send for Inner<A> { }
unsafe impl<A> Sync for Inner<A> { }
