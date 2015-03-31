use super::{spawn, sleep_ms};
use eventual::*;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

#[test]
pub fn test_complete_before_await() {
    let (c, f) = Future::<&'static str, ()>::pair();
    let (tx, rx) = channel();

    spawn(move || {
        c.complete("zomg");
        tx.send("done").unwrap()
    });

    sleep_ms(50);
    assert_eq!(f.await().unwrap(), "zomg");
    assert_eq!(rx.recv().unwrap(), "done");
}

#[test]
pub fn test_complete_after_await() {
    let (c, f) = Future::<&'static str, ()>::pair();
    let (tx, rx) = channel();

    spawn(move || {
        sleep_ms(50);
        c.complete("zomg");
        tx.send("done").unwrap();
    });

    assert_eq!(f.await().unwrap(), "zomg");
    assert_eq!(rx.recv().unwrap(), "done");
}

#[test]
pub fn test_receive_complete_before_await() {
    let (c, f) = Future::<&'static str, ()>::pair();
    let w1 = Arc::new(AtomicBool::new(false));
    let w2 = w1.clone();

    c.receive(move |c| {
        assert!(w2.load(Relaxed));
        c.unwrap().complete("zomg");
    });

    w1.store(true, Relaxed);
    assert_eq!(f.await().unwrap(), "zomg");
}

#[test]
pub fn test_receive_complete_after_await() {
    let (c, f) = Future::<&'static str, ()>::pair();
    let w1 = Arc::new(AtomicBool::new(false));
    let w2 = w1.clone();

    spawn(move || {
        sleep_ms(50);
        c.receive(move |c| {
            assert!(w2.load(Relaxed));
            c.unwrap().complete("zomg")
        });
    });

    w1.store(true, Relaxed);
    assert_eq!(f.await().unwrap(), "zomg");
}

#[test]
pub fn test_await_complete_before_consumer_await() {
    let (c, f) = Future::<&'static str, ()>::pair();

    spawn(move || {
        c.await().unwrap().complete("zomg")
    });

    sleep_ms(50);

    assert_eq!(f.await().unwrap(), "zomg");
}

#[test]
pub fn test_await_complete_after_consumer_await() {
    let (c, f) = Future::<&'static str, ()>::pair();

    spawn(move || {
        sleep_ms(50);
        c.await().unwrap().complete("zomg");
    });

    assert_eq!("zomg", f.await().unwrap());
}

#[test]
pub fn test_producer_await_when_consumer_await() {
    let (c, f) = Future::<&'static str, ()>::pair();

    spawn(move || {
        c.await().unwrap()
            .await().unwrap()
            .await().unwrap().complete("zomg");
    });

    sleep_ms(50);
    assert_eq!(f.await().unwrap(), "zomg");
}

#[test]
pub fn test_producer_fail_before_consumer_await() {
    let (c, f) = Future::<i32, &'static str>::pair();

    c.fail("nope");

    let err = f.await().unwrap_err();
    assert!(err.is_failed());
    assert_eq!(err.unwrap(), "nope");
}

#[test]
pub fn test_producer_drops_before_consumer_await() {
    let (c, f) = Future::<i32, ()>::pair();

    drop(c);

    let err = f.await().unwrap_err();
    assert!(err.is_aborted());
}

#[test]
pub fn test_producer_drops_after_consumer_await() {
    let (c, f) = Future::<i32, ()>::pair();

    spawn(move || {
        sleep_ms(50);
        drop(c);
    });

    let err = f.await().unwrap_err();
    assert!(err.is_aborted());
}
