use eventual::{Async, Timer};
use std::sync::mpsc::*;
use std::thread;
use std::time::{Instant, Duration};

#[test]
pub fn test_timer_register_early() {
    let timer = Timer::new();
    let (tx, rx) = channel();

    let start = Instant::now();

    timer.timeout_ms(300)
        .and_then(move |_| {
            assert!(start.elapsed() >= ms(300));
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

    thread::sleep(Duration::from_millis(600));

    let start = Instant::now();

    timeout
        .and_then(move |_| {
            assert!(start.elapsed() < ms(100));
            tx.send("done").unwrap()
        })
        .fire();

    assert_eq!("done", rx.recv().unwrap());
}

#[test]
pub fn test_timer_interval() {
    let timer = Timer::new();

    let mut prev = Instant::now();
    let ticks = timer.interval_ms(200).iter().take(10);

    for _ in ticks {
        let diff = prev.elapsed();
        assert!(diff >= ms(180), "actual={:?}", diff);
        prev = Instant::now()
    }
}

fn ms(ms: u64) -> Duration {
    Duration::from_millis(ms)
}
