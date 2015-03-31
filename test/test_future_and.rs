use eventual::*;
use std::sync::mpsc::channel;

#[test]
pub fn test_and_success_async() {
    let (c1, f1) = Future::<&'static str, ()>::pair();
    let (c2, f2) = Future::<i32, ()>::pair();
    let (tx1, rx) = channel();
    let tx2 = tx1.clone();

    c1.receive(move |c| {
        tx1.send("first").unwrap();
        c.unwrap().complete("zomg");
    });

    c2.receive(move |c| {
        tx2.send("second").unwrap();
        c.unwrap().complete(123);
    });

    let and = f1.and(f2);

    // No interest registered yet
    assert!(rx.try_recv().is_err());

    let res = and.await().unwrap();
    assert_eq!(res, 123);

    assert_eq!("first", rx.recv().unwrap());
    assert_eq!("second", rx.recv().unwrap());
}
