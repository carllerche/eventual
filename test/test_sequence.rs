use eventual::{self, Async, Future, Stream, AsyncError};
use std::sync::mpsc::{channel, Sender};

#[test]
pub fn test_sequencing_two_futures_on_demand() {
    let (c1, f1) = Future::<i32, ()>::pair();
    let (c2, f2) = Future::<i32, ()>::pair();

    let mut seq = eventual::sequence(vec![f1, f2]);

    c2.complete(123);

    match seq.await() {
        Ok(Some((v, rest))) => {
            assert_eq!(123, v);
            seq = rest;
        }
        v => panic!("unexpected value: {:?}", v),
    }

    c1.complete(234);

    match seq.await() {
        Ok(Some((v, rest))) => {
            assert_eq!(234, v);
            seq = rest;
        }
        v => panic!("unexpected value: {:?}", v),
    }

    assert!(seq.await().unwrap().is_none());
}

#[test]
pub fn test_sequencing_two_futures_up_front() {
    let (c1, f1) = Future::<i32, ()>::pair();
    let (c2, f2) = Future::<i32, ()>::pair();
    let (tx, rx) = channel();

    fn receive(stream: Stream<i32, ()>, tx: Sender<i32>) {
        stream.receive(move |res| {
            match res {
                Ok(Some((v, rest))) => {
                    tx.send(v).unwrap();
                    receive(rest, tx);
                }
                Ok(None) => tx.send(-1).unwrap(),
                Err(_) => panic!("nope"),
            }
        });
    }

    receive(eventual::sequence(vec![f1, f2]), tx);

    c2.complete(123);
    c1.complete(234);

    let vals: Vec<i32> = rx.iter().collect();
    assert_eq!(&[123, 234, -1], &vals[..]);
}

#[test]
pub fn test_sequencing_failed_future() {
    let (c1, f1) = Future::<i32, ()>::pair();
    let (c2, f2) = Future::<i32, ()>::pair();
    let (tx, rx) = channel();

    let seq = eventual::sequence(vec![f1, f2]);

    seq.receive(move |res| {
        match res {
            Err(AsyncError::Failed(_)) => tx.send("win").unwrap(),
            v => panic!("unexpected value {:?}", v),
        }
    });

    c2.fail(());
    c1.complete(234);

    assert_eq!("win", rx.recv().unwrap());
}
