use eventual::{self, Async, Future};
use syncbox::ThreadPool;
use std::thread;

#[test]
fn test_defer_runs_on_thread_pool() {
    ::env_logger::init().unwrap();

    // Set thread local
    let pool = ThreadPool::single_thread();
    let (complete, future) = Future::<i32, ()>::pair();
    let res = eventual::defer(pool.clone(), future).and_then(|v: i32| {
        // Assert thread local is not present here
        Ok(v + 5)
    });

    complete.complete(7);

    thread::sleep_ms(300);

    assert_eq!(Ok(7+ 5), res.await());
}
