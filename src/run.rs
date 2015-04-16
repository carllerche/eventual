use super::{
    Async,
    Pair,
    AsyncError,
    Future
};

use syncbox::Task;
use syncbox::TaskBox;
use syncbox::Run;

pub fn defer<R: Run<Box<TaskBox>> + Send + 'static, A: Async + 'static>(r: R, a: A) -> Future<A::Value, A::Error> {
    let (tx, rx) = Future::pair();

    tx.receive(move |res| {
        if let Ok(tx) = res {
            a.receive(move |res| {
                match res {
                    Ok(val) => {
                        println!("SCHEDULING COMPLETION");
                        r.run(Box::new(move || {
                            println!("COMPLETING VALUE");
                            tx.complete(val);
                        }))
                    },
                    Err(AsyncError::Failed(err)) => tx.fail(err),
                    Err(AsyncError::Aborted) => tx.abort(),
                }
            });
        }
    });

    rx
}
