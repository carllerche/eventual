use eventual::{Async, Timer};
use std::sync::mpsc::*;
use std::thread;
use time::{SteadyTime, Duration};

#[test]
pub fn test_timer_register_early() {
    let timer = Timer::new();
    let (tx, rx) = channel();

    let start = SteadyTime::now();

    timer.timeout_ms(300)
        .and_then(move |_| {
            assert!(SteadyTime::now() - start >= ms(300));
            tx.send("done").unwrap()
        })
        .fire();

    assert_eq!("done", rx.recv().unwrap());
}

#[test]
pub fn test_timer_register_late() {
    let timer = Timer::new();
    let (tx, rx) = channel();

    let timeout = timer.timeout_ms(300);

    thread::sleep_ms(600);

    let start = SteadyTime::now();

    timeout
        .and_then(move |_| {
            assert!(SteadyTime::now() - start < ms(100));
            tx.send("done").unwrap()
        })
        .fire();

    assert_eq!("done", rx.recv().unwrap());
}

fn ms(ms: u32) -> Duration {
    Duration::milliseconds(ms as i64)
}
