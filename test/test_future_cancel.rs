use eventual::*;
use std::sync::mpsc::channel;

#[test]
pub fn test_future_cancel_ready_before_complete() {
    let (complete, future) = Future::<i32, ()>::pair();

    let cancel = future.ready(move |_| panic!("nope"));
    let future = cancel.cancel().expect("cancel failed");

    complete.complete(123);
    assert_eq!(123, future.expect().unwrap());
}

#[test]
pub fn test_future_cancel_after_complete() {
    let (complete, future) = Future::<i32, ()>::pair();
    let (tx, rx) = channel();

    let cancel = future.ready(move |f| {
        tx.send(f.expect().unwrap()).unwrap();
    });

    complete.complete(123);
    assert!(cancel.cancel().is_none());
    assert_eq!(123, rx.recv().unwrap());
}

// TODO:
// - Test blocking & cancel
