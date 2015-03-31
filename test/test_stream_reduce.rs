use eventual::*;
use super::nums;

#[test]
pub fn test_stream_reduce_async() {
    let s = nums::<()>(0, 5).reduce(10, move |sum, v| sum + v);
    assert_eq!(20, s.await().unwrap());
}

#[test]
pub fn test_stream_reduce_fail() {
    let (tx, rx) = Stream::pair();
    tx.send(1)
        .and_then(|tx| tx.send(2))
        .and_then(|tx| tx.send(3))
        .and_then(|tx| tx.fail(()))
        .fire();

    let reduced = rx.reduce(0, move |sum, v| sum + v);
    assert_eq!(Err(AsyncError::Failed(())), reduced.await());
}

#[test]
#[ignore]
pub fn test_stream_reduce_async_success() {
    // Successfully reduce a stream with async computations
}

#[test]
#[ignore]
pub fn test_stream_reduce_async_fail() {
    // An async computation returned from a reduction fails
}
