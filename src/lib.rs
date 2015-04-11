//! Composable primitives for asynchronous computations
//!
//! The async module contains utilities for managing asynchronous computations.
//! These utilities are primarily based around `Future` and `Stream` types as
//! well as functions that allow composing computations on these types.
//!
//! ## Future
//!
//! A `Future` is a proxy representing the result of a computation which may
//! not be complete.  The computation may be running concurrently in another
//! thread or may be triggered upon completion of an asynchronous callback. One
//! way to think of a `Future` is as a `Result` where the value is
//! asynchronously computed.
//!
//! For example:
//!
//! ```
//! use eventual::*;
//!
//! // Run a computation in another thread
//! let future1 = Future::spawn(|| {
//!     // Represents an expensive computation, but for now just return a
//!     // number
//!     42
//! });
//!
//! // Run another computation
//! let future2 = Future::spawn(|| {
//!     // Another expensive computation
//!     18
//! });
//!
//! let res = join((
//!         future1.map(|v| v * 2),
//!         future2.map(|v| v + 5)))
//!     .and_then(|(v1, v2)| v1 - v2)
//!     .await().unwrap();
//!
//! assert_eq!(61, res);
//!
//! ```
//!
//! ## Stream
//!
//! A `Stream` is like a `Future`, except that instead of representing a single
//! value, it represents a sequence of values.
//!

#![crate_name = "eventual"]

extern crate syncbox;
extern crate clock_ticks;

#[macro_use]
extern crate log;

pub use self::future::{Future, Complete};
pub use self::stream::{Stream, StreamIter, Sender, BusySender};
pub use self::join::{join, Join};
pub use self::receipt::Receipt;
pub use self::select::{select, Select};
pub use self::sequence::sequence;

use std::fmt;

// ## TODO
//
// * Switch generics to where clauses
//   - rust-lang/rust#20300 (T::Foo resolution)
//
// * Allow Async::or & Async::or_else to change the error type
//
// * Improve performance / reduce allocations

mod core;
mod future;
mod join;
mod process;
mod receipt;
mod select;
mod sequence;
mod stream;

/// A value representing an asynchronous computation
pub trait Async : Send + Sized {
    type Value: Send;
    type Error: Send;
    type Cancel: Cancel<Self>;

    /// Returns true if `expect` will succeed.
    fn is_ready(&self) -> bool;

    /// Returns true if the async value is ready and has failed
    fn is_err(&self) -> bool;

    /// Get the underlying value if present
    fn poll(self) -> Result<AsyncResult<Self::Value, Self::Error>, Self>;

    /// Get the underlying value if present, panic otherwise
    fn expect(self) -> AsyncResult<Self::Value, Self::Error> {
        if let Ok(v) = self.poll() {
            return v;
        }

        panic!("the async value is not ready");
    }

    /// Invokes the given function when the Async instance is ready to be
    /// consumed.
    fn ready<F>(self, f: F) -> Self::Cancel where F: FnOnce(Self) + Send;

    /// Invoke the callback with the resolved `Async` result.
    fn receive<F>(self, f: F)
            where F: FnOnce(AsyncResult<Self::Value, Self::Error>) + Send {
        self.ready(move |async| {
            match async.poll() {
                Ok(res) => f(res),
                Err(_) => panic!("ready callback invoked but is not actually ready"),
            }
        });
    }

    /// Blocks the thread until the async value is complete and returns the
    /// result.
    fn await(self) -> AsyncResult<Self::Value, Self::Error> {
        use std::sync::mpsc::channel;

        let (tx, rx) = channel();

        self.receive(move |res| tx.send(res).ok().expect("receiver thread died"));
        rx.recv().ok().expect("async disappeared without a trace")
    }

    /// Trigger the computation without waiting for the result
    fn fire(self) {
        self.receive(drop)
    }

    /*
     *
     * ===== Computation Builders =====
     *
     */

    /// This method returns a future whose completion value depends on the
    /// completion value of the original future.
    ///
    /// If the original future completes with an error, the future returned by
    /// this method completes with that error.
    ///
    /// If the original future completes successfully, the future returned by
    /// this method completes with the completion value of `next`.
    fn and<U: Async<Error=Self::Error>>(self, next: U) -> Future<U::Value, Self::Error> {
        self.and_then(move |_| next)
    }

    /// This method returns a future whose completion value depends on the
    /// completion value of the original future.
    ///
    /// If the original future completes with an error, the future returned by
    /// this method completes with that error.
    ///
    /// If the original future completes successfully, the callback to this
    /// method is called with the value, and the callback returns a new future.
    /// The future returned by this method then completes with the completion
    /// value of that returned future.
    ///
    /// ```
    /// use eventual::*;
    ///
    /// let f = Future::of(1337);
    ///
    /// f.and_then(|v| {
    ///     assert_eq!(v, 1337);
    ///     Ok(1007)
    /// }).and_then(|v| {
    ///     assert_eq!(v, 1007)
    /// }).await();
    ///
    /// let e = Future::<(), &'static str>::error("failed");
    ///
    /// e.and_then(|v| {
    ///     panic!("unreachable");
    ///     Ok(())
    /// }).await();
    /// ```
    fn and_then<F, U: Async<Error=Self::Error>>(self, f: F) -> Future<U::Value, Self::Error>
            where F: FnOnce(Self::Value) -> U + Send,
                  U::Value: Send {
        let (complete, ret) = Future::pair();

        complete.receive(move |c| {
            if let Ok(complete) = c {
                self.receive(move |res| {
                    match res {
                        Ok(v) => {
                            f(v).receive(move |res| {
                                match res {
                                    Ok(u) => complete.complete(u),
                                    Err(AsyncError::Failed(e)) => complete.fail(e),
                                    _ => {}
                                }
                            });
                        }
                        Err(AsyncError::Failed(e)) => complete.fail(e),
                        _ => {}
                    }
                });
            }
        });

        ret
    }

    /// This method returns a future whose completion value depends on the
    /// completion value of the original future.
    ///
    /// If the original future completes successfully, the future returned by
    /// this method will complete with that value.
    ///
    /// If the original future completes with an error, the future returned by
    /// this method will complete with the completion value of the `alt` future
    /// passed in. That can be either a success or error.
    fn or<A>(self, alt: A) -> Future<Self::Value, A::Error>
            where A: Async<Value=Self::Value> {
        self.or_else(move |_| alt)
    }

    /// This method returns a future whose completion value depends on the
    /// completion value of the original future.
    ///
    /// If the original future completes successfully, the future returned by
    /// this method will complete with that value.
    ///
    /// If the original future completes with an error, this method will invoke
    /// the callback passed to the method, which should return a future. The
    /// future returned by this method will complete with the completion value
    /// of that future. That can be either a success or error.
    fn or_else<F, A>(self, f: F) -> Future<Self::Value, A::Error>
            where F: FnOnce(Self::Error) -> A + Send,
                  A: Async<Value=Self::Value> {

        let (complete, ret) = Future::pair();

        complete.receive(move |c| {
            if let Ok(complete) = c {
                self.receive(move |res| {
                    match res {
                        Ok(v) => complete.complete(v),
                        Err(AsyncError::Failed(e)) => {
                            f(e).receive(move |res| {
                                match res {
                                    Ok(v) => complete.complete(v),
                                    Err(AsyncError::Failed(e)) => complete.fail(e),
                                    _ => {}
                                }
                            });
                        }
                        Err(AsyncError::Aborted) => drop(complete),
                    }
                });
            }
        });

        ret
    }
}

pub trait Pair {
    type Tx;

    fn pair() -> (Self::Tx, Self);
}

pub trait Cancel<A: Send> : Send {
    fn cancel(self) -> Option<A>;
}

/*
 *
 * ===== Async implementations =====
 *
 */

impl<T: Send, E: Send> Async for Result<T, E> {
    type Value = T;
    type Error = E;
    type Cancel = Option<Result<T, E>>;

    fn is_ready(&self) -> bool {
        true
    }

    fn is_err(&self) -> bool {
        self.is_err()
    }

    fn poll(self) -> Result<AsyncResult<T, E>, Result<T, E>> {
        Ok(self.await())
    }

    fn ready<F: FnOnce(Result<T, E>) + Send>(self, f: F) -> Option<Result<T, E>> {
        f(self);
        None
    }

    fn await(self) -> AsyncResult<T, E> {
        self.map_err(|e| AsyncError::Failed(e))
    }
}

impl<A: Send> Cancel<A> for Option<A> {
    fn cancel(self) -> Option<A> {
        self
    }
}

/*
 *
 * ===== Async implementations =====
 *
 */

macro_rules! async_impl_body {
    ($ty:ty) => (
        fn is_ready(&self) -> bool {
            true
        }

        fn is_err(&self) -> bool {
            false
        }

        fn poll(self) -> Result<AsyncResult<$ty, ()>, $ty> {
            Ok(Ok(self))
        }

        fn ready<F: FnOnce($ty) + Send>(self, f: F) -> Option<$ty> {
            f(self);
            None
        }

        fn await(self) -> AsyncResult<$ty, ()> {
            Ok(self)
        }
    );
}

macro_rules! async_impl {
    ($ty:ty) => (
        impl Async for $ty {
            type Value  = $ty;
            type Error  = ();
            type Cancel = Option<$ty>;

            async_impl_body!($ty);
        }
    );

    ($ty:ty, $($rest:tt),*) => (
        async_impl!($ty);
        async_impl!($($rest),*);
    );
}

// For now, implement on as many concrete types as possible. One day, Rust will
// support specialization (hopefully) and this won't be necessary.
async_impl!(
    (), bool, String,
    u8, u16, u32, u64, usize,
    i8, i16, i32, i64, isize);

impl<E: Send> Async for Vec<E> {
    type Value = Vec<E>;
    type Error = ();
    type Cancel = Option<Vec<E>>;

    async_impl_body!(Vec<E>);
}

/*
 *
 * ===== AsyncResult =====
 *
 */

pub type AsyncResult<T, E> = Result<T, AsyncError<E>>;

#[derive(Eq, PartialEq)]
pub enum AsyncError<E: Send> {
    Failed(E),
    Aborted,
}

impl<E: Send> AsyncError<E> {
    pub fn failed(err: E) -> AsyncError<E> {
        AsyncError::Failed(err)
    }

    pub fn aborted() -> AsyncError<E> {
        AsyncError::Aborted
    }

    pub fn is_aborted(&self) -> bool {
        match *self {
            AsyncError::Aborted => true,
            _ => false,
        }
    }

    pub fn is_failed(&self) -> bool {
        match *self {
            AsyncError::Failed(..) => true,
            _ => false,
        }
    }

    pub fn unwrap(self) -> E {
        match self {
            AsyncError::Failed(err) => err,
            AsyncError::Aborted => panic!("unwrapping a cancellation error"),
        }
    }

    pub fn take(self) -> Option<E> {
        match self {
            AsyncError::Failed(err) => Some(err),
            _ => None,
        }
    }
}

impl<E: Send + fmt::Debug> fmt::Debug for AsyncError<E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AsyncError::Failed(ref e) => write!(fmt, "AsyncError::Failed({:?})", e),
            AsyncError::Aborted => write!(fmt, "AsyncError::Aborted"),
        }
    }
}

/*
 *
 * ===== BoxedReceive =====
 *
 */

// Needed to allow virtual dispatch to Receive
trait BoxedReceive<T> : Send {
    fn receive_boxed(self: Box<Self>, val: T);
}

impl<F: FnOnce(T) + Send, T> BoxedReceive<T> for F {
    fn receive_boxed(self: Box<F>, val: T) {
        (*self)(val)
    }
}
