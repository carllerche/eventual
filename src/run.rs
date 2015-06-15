use super::{Async, Pair, AsyncError, Future};

use syncbox::Task;
use syncbox::TaskBox;
use syncbox::Run;
use std::boxed::FnBox;

pub fn defer<R: Run<Box<TaskBox>> + Send + 'static + Sync + Clone,
             A: Async + 'static>(task_runner: &R, future_in: A) -> Future<A::Value, A::Error> {
    let (complete, future_out) = Future::pair();
    let task_runner_copy = task_runner.clone();
    complete.receive(|result_or_error| {
        if let Ok(complete) = result_or_error {
            future_in.receive(move | result_or_error | {
                match result_or_error {
                    Ok(val) => task_runner_copy.run(Box::new(|| complete.complete(val))),
                    Err(AsyncError::Failed(err)) => complete.fail(err),
                    Err(AsyncError::Aborted) => complete.abort(),
                }
            });
        }
    });
    future_out
}

pub fn background<R: Run<Box<TaskBox>> + Send + 'static + Sync + Clone,
                  F: FnBox() -> T + Send + 'static,
                  T: Send >(task_runner: &R, closure: Box<F>)
                                      -> Future<T, ()> {
  	
    let (complete, future) = Future::<(),()>::pair();
    let res = defer(task_runner, future).and_then( |() | {    
    	Ok(closure.call_box(()))
    });
    complete.complete(());
    res 
}	