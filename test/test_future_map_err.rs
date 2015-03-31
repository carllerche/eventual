use eventual::*;

#[test]
pub fn test_future_map_err_success() {
    let (tx, rx) = Future::<i32, ()>::pair();
    let rx = rx.map_err(|_| panic!("nope"));

    tx.complete(123);
    assert_eq!(123, rx.await().unwrap());
}

#[test]
pub fn test_future_map_err_completed() {
    let f = Future::<i32, ()>::of(123)
        .map_err(|_| unreachable!());

    assert_eq!(123, f.await().unwrap());
}

#[test]
pub fn test_future_map_err_failed() {
    let f = Future::<(), i32>::error(123)
        .map_err(|e| {
            assert_eq!(123, e);
            "win"
        });

    match f.await() {
        Err(AsyncError::Failed(e)) => assert_eq!("win", e),
        _ => panic!("unexpected value"),
    }
}

#[test]
pub fn test_future_map_err_fail() {
    let (tx, rx) = Future::<(), i32>::pair();
    let rx = rx.map_err(|e| {
        assert_eq!(123, e);
        "win"
    });

    tx.fail(123);

    match rx.await() {
        Err(AsyncError::Failed(e)) => assert_eq!("win", e),
        _ => panic!("unexpected value"),
    }
}

#[test]
pub fn test_future_map_err_abort() {
    let (tx, rx) = Future::<(), i32>::pair();
    let rx = rx.map_err(|_| unreachable!());

    drop(tx);

    match rx.await() {
        Err(AsyncError::Aborted) => {}
        _ => panic!("unexpected value"),
    }
}
