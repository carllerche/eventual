use eventual::*;
use std::sync::mpsc::channel;

#[test]
pub fn test_or_first_success_async() {
    let (c1, f1) = Future::<&'static str, ()>::pair();
    let (c2, f2) = Future::<&'static str, ()>::pair();
    let (tx1, rx) = channel();
    let tx2 = tx1.clone();

    c1.receive(move |c| {
        tx1.send("first").unwrap();
        c.unwrap().complete("zomg");
    });

    c2.receive(move |res| {
        if let Err(AsyncError::Aborted) = res {
            tx2.send("winning").unwrap();
        }
    });

    let or = f1.or(f2);

    // No interest registered yet
    assert!(rx.try_recv().is_err());

    let res = or.await().unwrap();
    assert_eq!(res, "zomg");

    assert_eq!("first", rx.recv().unwrap());
    assert_eq!("winning", rx.recv().unwrap());
}

#[test]
pub fn test_or_else_complete_before_receive() {
    let (c, f) = Future::<&'static str, i32>::pair();
    let (tx, rx) = channel();

    f.or_else(move |e| {
        assert_eq!(123, e);
        Ok::<&'static str, ()>("caught")
    }).receive(move |res| {
        tx.send(res.unwrap()).unwrap();
    });

    c.fail(123);

    assert_eq!(rx.recv().unwrap(), "caught");
}
