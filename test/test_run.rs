extern crate syncbox;
use eventual::{defer, Future, Async};

// TODO figure out how to get rid of unused import error here
use syncbox::ThreadPool;
#[test]
fn test_defer_runs_on_thread_pool() {
    // Set thread local
    let pool = ThreadPool::single_thread();
    let (complete, future) = Future::<i32, ()>::pair();
    let res = defer(&pool, future).and_then(|v: i32| {
        // Assert thread local is not present here
        Ok(v + 5)
    });

    complete.complete(7);
    assert_eq!(Ok(7 + 5), res.await());
}