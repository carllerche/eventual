use super::{Async, Future, Complete, AsyncError};
use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::{self, AtomicIsize};
use std::sync::atomic::Ordering;

pub fn join<J: Join<T, E>, T: Send, E: Send>(asyncs: J) -> Future<T, E> {
    let (complete, future) = Future::pair();

    // Don't do any work until the consumer registers interest in the completed
    // value.
    complete.receive(move |res| {
        if let Ok(complete) = res {
            asyncs.join(complete);
        }
    });

    future
}

pub trait Join<T, E> : Send {
    fn join(self, complete: Complete<T, E>);
}

trait Partial<R> {
    fn consume(&mut self) -> R;
}

impl<T1, T2> Partial<(T1, T2)> for (Option<T1>, Option<T2>) {
    fn consume(&mut self) -> (T1, T2) {
        (self.0.take().unwrap(), self.1.take().unwrap())
    }
}

impl<T1, T2, T3> Partial<(T1, T2, T3)> for (Option<T1>, Option<T2>, Option<T3>) {
    fn consume(&mut self) -> (T1, T2, T3) {
        (self.0.take().unwrap(), self.1.take().unwrap(), self.2.take().unwrap())
    }
}

struct Progress<P: Partial<R>, R: Send, E: Send> {
    inner: Arc<UnsafeCell<ProgressInner<P, R, E>>>,
}

unsafe impl<P: Partial<R>, R: Send, E: Send> Sync for Progress<P, R, E> {}
unsafe impl<P: Partial<R>, R: Send, E: Send> Send for Progress<P, R, E> {}

impl<P: Partial<R>, R: Send, E: Send> Progress<P, R, E> {
    fn new(vals: P, complete: Complete<R, E>, remaining: isize) -> Progress<P, R, E> {
        let inner = Arc::new(UnsafeCell::new(ProgressInner {
            vals: vals,
            complete: Some(complete),
            remaining: AtomicIsize::new(remaining),
        }));

        Progress { inner: inner }
    }

    fn succeed(&self) {
        let complete = self.inner_mut().complete.take()
            .expect("complete already consumed");

        // Set an acquire fence to make sure that all values have been acquired
        atomic::fence(Ordering::Acquire);

        debug!("completing join");
        complete.complete(self.inner_mut().vals.consume());
    }

    fn fail(&self, err: AsyncError<E>) {
        if self.inner().remaining.swap(-1, Ordering::Relaxed) > 0 {
            let complete = self.inner_mut().complete.take()
                .expect("complete already consumed");

            // If not an execution error, it is a cancellation error, in which
            // case, our complete will go out of scope and propagate up a
            // cancellation.
            if let AsyncError::Failed(e) = err {
                complete.fail(e);
            }
        }
    }

    fn vals_mut<'a>(&'a self) -> &'a mut P {
        &mut self.inner_mut().vals
    }

    fn dec(&self) -> isize {
        self.inner().remaining.fetch_sub(1, Ordering::Release) - 1
    }

    fn inner(&self) -> &ProgressInner<P, R, E> {
        use std::mem;
        unsafe { mem::transmute(self.inner.get()) }
    }

    fn inner_mut(&self) -> &mut ProgressInner<P, R, E> {
        use std::mem;
        unsafe { mem::transmute(self.inner.get()) }
    }
}

impl<P: Partial<R>, R: Send, E: Send> Clone for Progress<P, R, E> {
    fn clone(&self) -> Progress<P, R, E> {
        Progress { inner: self.inner.clone() }
    }
}

struct ProgressInner<P: Partial<R>, R: Send, E: Send> {
    vals: P,
    complete: Option<Complete<R, E>>,
    remaining: AtomicIsize,
}

macro_rules! expr {
    ($e: expr) => { $e };
}

macro_rules! component {
    ($async:ident, $progress:ident, $id:tt) => {{
        let $progress = $progress.clone();

        $async.receive(move |res| {
            debug!(concat!("dependent future complete; id=", $id, "; success={}"), res.is_ok());

            // Get a pointer to the value staging area (Option<T>). Values will
            // be stored here until the join is complete
            let slot = expr!(&mut $progress.vals_mut().$id);

            match res {
                Ok(v) => {
                    // Set the value
                    *slot = Some(v);

                    // Track that the value has been received
                    if $progress.dec() == 0 {
                        debug!("last future completed -- completing join");
                        // If all values have been received, successfully
                        // complete the future
                        $progress.succeed();
                    }
                }
                Err(e) => {
                    $progress.fail(e);
                }
            }
        });
    }};
}

/*
 *
 * ===== Join for Tuples =====
 *
 */

impl<A1: Async<Error=E>, A2: Async<Error=E>, E> Join<(A1::Value, A2::Value), E> for (A1, A2)
        where E: Send,
              A1::Value: Send,
              A2::Value: Send {

    fn join(self, complete: Complete<(<A1 as Async>::Value, <A2 as Async>::Value), E>) {
        let (a1, a2) = self;
        let p = Progress::new((None, None), complete, 2);

        component!(a1, p, 0);
        component!(a2, p, 1);
    }
}

impl<A1: Async<Error=E>, A2: Async<Error=E>, A3: Async<Error=E>, E> Join<(A1::Value, A2::Value, A3::Value), E> for (A1, A2, A3)
        where E: Send,
              A1::Value: Send,
              A2::Value: Send,
              A3::Value: Send {

    fn join(self, complete: Complete<(A1::Value, A2::Value, A3::Value), E>) {
        let (a1, a2, a3) = self;
        let p = Progress::new((None, None, None), complete, 3);

        component!(a1, p, 0);
        component!(a2, p, 1);
        component!(a3, p, 2);
    }
}
