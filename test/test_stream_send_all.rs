use super::nums;
use eventual::*;
use std::sync::mpsc::channel;

#[test]
pub fn test_stream_successfully_sending_all_values() {
    let (tx, rx) = Stream::<usize, ()>::pair();

    tx.send(1)
        .and_then(|tx| {
            tx.send_all(nums::<()>(2, 5))
                .map_err(|_| unreachable!())
        })
        .and_then(|tx| tx.send(5))
        .fire();

    let vals: Vec<usize> = rx.iter().collect();
    assert_eq!(&[1, 2, 3, 4, 5], &vals);
}

#[test]
pub fn test_stream_producer_fails() {
    let (tx, src) = Stream::<i32, &'static str>::pair();

    // Populate the source
    tx.and_then(|tx| tx.send(2))
        .and_then(|tx| tx.send(3))
        .and_then(|tx| tx.fail("nope"))
        .fire();

    let (tx, dst) = Stream::<i32, &'static str>::pair();

    tx.send(1)
        .and_then(move |tx| {
            tx.send_all(src)
                .or_else(|(err, tx)| {
                    assert_eq!("nope", err);
                    tx.send(10)
                })
        })
        .fire();

    let vals: Vec<i32> = dst.iter().collect();
    assert_eq!(&[1, 2, 3, 10], &vals);
}

#[test]
pub fn test_stream_consumer_loses_interest() {
    let (seen, rx) = channel();
    let (tx, dst) = Stream::<usize, ()>::pair();

    let src = nums::<()>(1, 5).map(move |v| {
        seen.send(v).unwrap();
        v
    });

    let _ = tx.send_all(src);

    let vals: Vec<usize> = dst.take(3).iter().collect();
    assert_eq!(&[1, 2, 3], &vals);
    assert_eq!(vals, rx.iter().collect());
}

#[test]
pub fn test_future_send_success() {
    let (tx, rx) = Stream::<usize, ()>::pair();

    tx.send(1)
        .and_then(|tx| {
            tx.send_all(Future::<usize, ()>::of(2))
                .map_err(|_| unreachable!())
        })
        .and_then(|tx| tx.send(3))
        .fire();

    let vals: Vec<usize> = rx.iter().collect();
    assert_eq!(&[1, 2, 3], &vals);
}

#[test]
pub fn test_future_send_fail() {
    let (tx, rx) = Stream::<usize, &'static str>::pair();

    tx.send(1)
        .and_then(|tx| {
            tx.send_all(Future::error("nope"))
                .or_else(|(err, tx)| {
                    assert_eq!("nope", err);
                    tx.send(10)
                })
        })
        .and_then(|tx| tx.send(3))
        .fire();

    let vals: Vec<usize> = rx.iter().collect();
    assert_eq!(&[1, 10, 3], &vals);
}
