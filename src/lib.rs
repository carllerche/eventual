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
//!     .and_then(|(v1, v2)| Ok(v1 - v2))
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
#![deny(warnings)]

extern crate syncbox;
extern crate time;

#[macro_use]
extern crate log;

pub use self::future::{Future, Complete};
pub use self::join::{join, Join};
pub use self::receipt::Receipt;
pub use self::run::{background, defer};
pub use self::select::{select, Select};
pub use self::sequence::sequence;
pub use self::stream::{Stream, StreamIter, Sender, BusySender};
pub use self::timer::Timer;

use std::error::Error;
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
mod run;
mod select;
mod sequence;
mod stream;
mod timer;

/// A value representing an asynchronous computation
pub trait Async : Send + 'static + Sized {
    type Value: Send + 'static;
    type Error: Send + 'static;
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
    fn ready<F>(self, f: F) -> Self::Cancel where F: FnOnce(Self) + Send + 'static;

    /// Invoke the callback with the resolved `Async` result.
    fn receive<F>(self, f: F)
            where F: FnOnce(AsyncResult<Self::Value, Self::Error>) + Send + 'static {
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

    /// This method returns a Future whose completion value depends on the
    /// ready value of the original future.
    ///
    /// ```
    /// use eventual::*;
    /// #[derive(Debug, PartialEq)]
    /// struct Move(u8);
    ///
    /// let m = Move(10);
    ///
    /// let f = Future::<&'static str, ()>::of("hello");
    /// let computed = f.when(move |res| {
    ///     match res {
    ///         Ok(res) => Ok((res, m)),
    ///         Err(e) => Err((e, m))
    ///     }
    /// }).await();
    ///
    /// assert_eq!(computed, Ok(("hello", Move(10))));
    ///
    /// ```
    fn when<F, U: Async>(self, f: F) -> Future<U::Value, U::Error>
            where F: FnOnce(Result<Self::Value, Self::Error>) -> U + Send + 'static,
                  U::Value: Send + 'static,
                  U::Error: Send + 'static {
        let (complete, ret) = Future::pair();

        complete.receive(move |c| {
            if let Ok(complete) = c {
                self.receive(move |res| {
                    let res = match res {
                        Ok(v) => Ok(v),
                        Err(AsyncError::Failed(e)) => Err(e),
                        Err(AsyncError::Aborted) => return
                    };
                    f(res).receive(move |res| {
                        match res {
                            Ok(u) => complete.complete(u),
                            Err(AsyncError::Failed(e)) => complete.fail(e),
                            _ => {}
                        }
                    })
                });
            }
        });

        ret
    }

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
            where F: FnOnce(Self::Value) -> U + Send + 'static,
                  U::Value: Send + 'static {
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
            where F: FnOnce(Self::Error) -> A + Send + 'static,
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

pub trait Cancel<A: Send + 'static> : Send + 'static {
    fn cancel(self) -> Option<A>;
}

/*
 *
 * ===== Async implementations =====
 *
 */

impl<T: Send + 'static, E: Send + 'static> Async for Result<T, E> {
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

    fn ready<F: FnOnce(Result<T, E>) + Send + 'static>(self, f: F) -> Option<Result<T, E>> {
        f(self);
        None
    }

    fn await(self) -> AsyncResult<T, E> {
        self.map_err(|e| AsyncError::Failed(e))
    }
}

impl<A: Send + 'static> Cancel<A> for Option<A> {
    fn cancel(self) -> Option<A> {
        self
    }
}

/// Convenience implementation for (), to ease use of side-effecting functions returning unit
impl Async for () {
    type Value  = ();
    type Error  = ();
    type Cancel = Option<()>;

    fn is_ready(&self) -> bool {
        true
    }

    fn is_err(&self) -> bool {
        false
    }

    fn poll(self) -> Result<AsyncResult<(), ()>, ()> {
        Ok(Ok(self))
    }

    fn ready<F: FnOnce(()) + Send + 'static>(self, f: F) -> Option<()> {
        f(self);
        None
    }

    fn await(self) -> AsyncResult<(), ()> {
         Ok(self)
    }
}

/*
 *
 * ===== AsyncResult =====
 *
 */

pub type AsyncResult<T, E> = Result<T, AsyncError<E>>;

#[derive(Eq, PartialEq)]
pub enum AsyncError<E: Send + 'static> {
    Failed(E),
    Aborted,
}

impl<E: Send + 'static> AsyncError<E> {
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

impl<E: Send + Error + 'static> Error for AsyncError<E> {
    fn description(&self) -> &str {
        match *self {
            AsyncError::Failed(ref e) => e.description(),
            AsyncError::Aborted => "aborted",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            AsyncError::Failed(ref e) => e.cause(),
            AsyncError::Aborted => None,
        }
    }
}

impl<E: Send + 'static + fmt::Debug> fmt::Debug for AsyncError<E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AsyncError::Failed(ref e) => write!(fmt, "AsyncError::Failed({:?})", e),
            AsyncError::Aborted => write!(fmt, "AsyncError::Aborted"),
        }
    }
}

impl<E: Send + 'static + fmt::Display> fmt::Display for AsyncError<E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AsyncError::Failed(ref e) => write!(fmt, "{}", e),
            AsyncError::Aborted => write!(fmt, "[aborted]"),
        }
    }
}

/*
 *
 * ===== BoxedReceive =====
 *
 */

// Needed to allow virtual dispatch to Receive
trait BoxedReceive<T> : Send + 'static {
    fn receive_boxed(self: Box<Self>, val: T);
}

impl<F: FnOnce(T) + Send + 'static, T> BoxedReceive<T> for F {
    fn receive_boxed(self: Box<F>, val: T) {
        (*self)(val)
    }
}
