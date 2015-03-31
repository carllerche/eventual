use eventual::{self, Async, Future};
use std::sync::mpsc::channel;

#[test]
pub fn test_selecting_two_futures_async_success() {
    let (c1, f1) = Future::<i32, ()>::pair();
    let (c2, f2) = Future::<i32, ()>::pair();
    let (tx, rx) = channel();

    let sel = eventual::select((f1, f2));

    assert!(!sel.is_ready());
    assert!(!c1.is_ready());
    assert!(!c2.is_ready());
    assert!(rx.try_recv().is_err());

    sel.receive(move |res| {
        let (i, (f1, f2)) = res.unwrap();

        assert_eq!(i, 0);
        assert!(!f2.is_ready());

        let val = f1.expect().unwrap();

        tx.send((val, f2)).unwrap();
    });

    assert!(c1.is_ready());
    assert!(c2.is_ready());

    c1.complete(123);

    let (val, f2) = rx.recv().unwrap();

    assert_eq!(123, val);
    assert!(!f2.is_ready());

    c2.complete(234);
    assert_eq!(234, f2.expect().unwrap());
}

#[test]
pub fn test_selecting_two_futures_async_error() {
    let (c1, f1) = Future::<(), i32>::pair();
    let (_c, f2) = Future::<(), i32>::pair();
    let (tx, rx) = channel();

    let sel = eventual::select((f1, f2));

    sel.receive(move |res| {
        debug!("receiving value");
        tx.send(res.unwrap_err()).unwrap();
    });

    c1.fail(123);
    assert_eq!(123, rx.recv().unwrap().unwrap());
}

#[test]
pub fn test_selecting_two_completed_futures_async() {
    let f1 = Future::<i32, ()>::of(123);
    let f2 = Future::of(234);
    let (tx, rx) = channel();

    let sel = eventual::select((f1, f2));

    assert!(!sel.is_ready());

    sel.receive(move |res| {
        tx.send(res.unwrap()).unwrap();
    });

    let (i, (f1, f2)) = rx.recv().unwrap();

    assert_eq!(0, i);
    assert_eq!(123, f1.expect().unwrap());
    assert_eq!(234, f2.expect().unwrap());
}
