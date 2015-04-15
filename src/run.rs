use super::{
	Async,
		Pair,
		AsyncError,
		Future
};

use syncbox::Task;
use syncbox::TaskBox;
use syncbox::Run;

pub fn defer< R: Run<Box<TaskBox>>, A: Async + 'static>(r: R, a: A) -> Future<A::Value, A::Error> {
	let (tx, rx) = Future::pair();
	r.run(Box::new(move || {
		a.receive(move |res| { 
			match res {
				Ok(val) => tx.complete(val),
				Err(AsyncError::Failed(err)) => tx.fail(err),
				Err(AsyncError::Aborted) => {
					tx.abort();
				}
					// Otherwise, do nothing. A thread panicked before completing the value, doing
					// nothing will proxy the panic (represented as an AsyncError::Aborted)
			}
		})
	}));
	rx
}

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
	assert_eq!(Ok(7+ 5), res.await());
}
