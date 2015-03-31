use eventual::*;
use std::sync::mpsc::channel;

#[test]
pub fn test_stream_cancel_before_send() {
    let (generate, stream) = Stream::<i32, ()>::pair();

    let cancel = stream.ready(move |_| panic!("nope"));
    let stream = cancel.cancel().expect("cancel failed");

    generate.send(123);

    match stream.expect().unwrap() {
        Some((v, _)) => assert_eq!(123, v),
        _ => panic!("nope"),
    }
}

#[test]
pub fn test_stream_cancel_after_send() {
    let (generate, stream) = Stream::<i32, ()>::pair();
    let (tx, rx) = channel();

    let cancel = stream.ready(move |s| {
        tx.send(s.expect().unwrap()).unwrap()
    });

    generate.send(123);

    assert!(cancel.cancel().is_none());

    match rx.recv().unwrap() {
        Some((v, _)) => assert_eq!(123, v),
        _ => panic!("nope"),
    }
}

#[test]
pub fn test_stream_cancel_with_value() {
    let (tx, rx) = Stream::<i32, ()>::pair();

    tx.send(123);
    drop(rx);
}
