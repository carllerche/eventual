use eventual::*;

#[test]
pub fn test_stream_map_err_success() {
    let (tx, rx) = Stream::<i32, ()>::pair();
    let rx = rx.map_err(|_| panic!("nope"));

    tx.send(123)
        .and_then(|tx| tx.send(234))
        .and_then(|tx| tx.send(345))
        .fire();

    let vals: Vec<i32> = rx.iter().collect();
    assert_eq!(&[123, 234, 345], &vals);
}

#[test]
pub fn test_stream_map_err_fail() {
    let (tx, rx) = Stream::pair();

    tx.send(123)
        .and_then(|tx| tx.send(234))
        .and_then(|tx| tx.fail("win"))
        .fire();

    let mut rx = rx.map_err(|e| {
        assert_eq!("win", e);
        123
    });

    match rx.await() {
        Ok(Some((v, rest))) => {
            assert_eq!(123, v);
            rx = rest;
        }
        _ => panic!("unexpected value"),
    }

    match rx.await() {
        Ok(Some((v, _))) => assert_eq!(234, v),
        _ => panic!("unexpected value"),
    }
}
