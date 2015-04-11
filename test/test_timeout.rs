use eventual::*;

#[test]
fn test_timeout() {
    // timeout for 400ms
    let timeout = Future::timeout(400);
    assert!(!timeout.is_ready());

    let wait = timeout.await();
    assert!(wait.is_ok());
}

#[test]
fn test_interval() {
    // triggers every 100ms
    let interval = Stream::interval(100);
    let collected = interval.take(4).collect().await();

    assert!(collected.is_ok());
    let collected = collected.unwrap();
    assert!(collected.len() == 4);
}
