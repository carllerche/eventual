extern crate syncbox;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use eventual::{background, defer, Future, Async};

// TODO figure out how to get rid of unused import error here
use syncbox::ThreadPool;
#[test]
fn test_defer_runs_on_thread_pool() {
    // Set thread local
    let pool = ThreadPool::single_thread();
    let (complete, future) = Future::<i32, ()>::pair();
    let res = defer(pool, future).and_then(|v: i32| {
        // Assert thread local is not present here
        Ok(v + 5)
    });
    complete.complete(7);
    assert_eq!(Ok(7 + 5), res.await());
}

#[test]
fn test_threadpool_background() {
    // Set thread local
    let pool = ThreadPool::single_thread();
    let flag = Arc::new(AtomicBool::new(false));
    let f = flag.clone();
    let result = background(pool, Box::new(move || {
        assert!(f.load(Ordering::Relaxed));
        5
    }));
    // Wait for a bit to make sure that the background task hasn't run

    thread::sleep(Duration::from_millis(100));
    // Set the flag
    flag.store(true, Ordering::Relaxed);
    assert_eq!(Ok(5), result.await());
}
