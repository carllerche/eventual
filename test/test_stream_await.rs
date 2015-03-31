use eventual::*;

#[test]
pub fn test_stream_fail_await() {
    let (tx, rx) = Stream::<u32, &'static str>::pair();

    tx.fail("nope");
    assert_eq!("nope", rx.await().unwrap_err().unwrap());
}

#[test]
pub fn test_stream_fail_second_iter_await() {
    let (tx, mut rx) = Stream::<u32, &'static str>::pair();

    tx.send(1)
        .and_then(|tx| tx.fail("nope"))
        .fire();

    match rx.await() {
        Ok(Some((v, rest))) => {
            assert_eq!(v, 1);
            rx = rest;
        }
        _ => panic!("unexpected value for head of stream"),
    }

    assert_eq!("nope", rx.await().unwrap_err().unwrap());
}
