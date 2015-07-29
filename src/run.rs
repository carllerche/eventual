use super::{Async, Pair, AsyncError, Future};
use syncbox::Task;
use syncbox::TaskBox;
use syncbox::Run;
use std::sync::Arc;

/// This method defers a task onto a task runner until we can complete that call.
/// Currently we only support using a ThreadPool as the task runner itself.
pub fn defer<R: Run<Box<TaskBox>> + Send + Sync + 'static,
             A: Async + 'static>(task_runner: Arc<R>, future_in: A) -> Future<A::Value, A::Error> {
    let (complete, future_out) = Future::pair();
    complete.receive(|result_or_error| {
        if let Ok(complete) = result_or_error {
            future_in.receive(move | result_or_error | {
                match result_or_error {
                    Ok(val) => task_runner.run(Box::new(|| complete.complete(val))),
                    Err(AsyncError::Failed(err)) => complete.fail(err),
                    Err(AsyncError::Aborted) => complete.abort(),
                }
            });
        }
    });
    future_out
}

/// This method backgrounds a task onto a task runner waiting for complete to be called.
/// Currently we only support using a ThreadPool as the task runner itself.
pub fn background<R: Run<Box<TaskBox>> + Send + Sync + 'static, F: FnOnce() -> T  + Send + 'static,
                  T: Send>(task_runner: Arc<R>, closure: Box<F>) -> Future<T, ()> {
    let (complete, future) = Future::<(), ()>::pair();
    let res = defer(task_runner, future).and_then(move |()| {
        Ok(closure())
    });
    complete.complete(());
    res
}
