//! # Futures & Streams
//!
//! Contains Future and Stream types as well as functions to operate on them.
//!
//! ## Future
//!
//! A future represents a value that will be provided sometime in the future.
//! The value may be computed concurrently in another thread or may be provided
//! upon completion of an asynchronous callback. The abstraction allows
//! describing computations to perform on the value once it is realized as well
//! as how to handle errors.
//!
//! One way to think of a Future is as a Result where the value is
//! asynchronously computed.
//!
//! # Stream

extern crate eventual;
extern crate time;
extern crate syncbox;

#[macro_use]
extern crate log;

/*
 * Last ported test: test_producer_fail_before_consumer_take
 */

// == Future tests ==
mod test_future_and;
mod test_future_await;
mod test_future_cancel;
mod test_future_map;
mod test_future_map_err;
mod test_future_or;
mod test_future_receive;

// == Join tests ==
mod test_join;
mod test_run;

// == Select tests ==
mod test_select;

// == Sequence tests ==
mod test_sequence;

// == Stream tests ==
mod test_stream_await;
mod test_stream_cancel;
mod test_stream_collect;
mod test_stream_each;
mod test_stream_filter;
mod test_stream_iter;
mod test_stream_map;
mod test_stream_map_err;
mod test_stream_process;
mod test_stream_receive;
mod test_stream_reduce;
mod test_stream_send_all;
mod test_stream_take;

// == Timer tests ==
mod test_timer;

/*
 *
 * ===== Helpers =====
 *
 */

use eventual::*;
use std::sync::mpsc::{Receiver, channel};

fn nums<E: Send>(from: usize, to: usize) -> Stream<usize, E> {
    Future::lazy(move || {
        debug!("range tick; from={}", from);

        if from < to {
            Ok(Some((from, nums(from + 1, to))))
        } else {
            Ok(None)
        }
    }).to_stream()
}

fn futures<T: Send, E: Send>(n: u32) -> (Vec<Complete<T, E>>, Receiver<Future<T, E>>) {
    let mut v = vec![];
    let (tx, rx) = channel();

    for _ in 0..n {
        let (complete, future) = Future::pair();
        tx.send(future).unwrap();
        v.push(complete);
    }

    (v, rx)
}

fn spawn<F: FnOnce() + Send + 'static>(f: F) {
    use std::thread;
    thread::spawn(f);
}

fn sleep_ms(ms: usize) {
    use std::thread;
    use std::time::Duration;
    thread::sleep(Duration::from_millis(ms as u64));
}
