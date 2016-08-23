use {Async, Future, Stream, Sender};
use syncbox::ScheduledThreadPool;
use std::time::{Instant, Duration};

/// Provides timeouts as a `Future` and periodic ticks as a `Stream`.
pub struct Timer {
    pool: ScheduledThreadPool,
}

impl Timer {
    /// Creates a new timer backed by a thread pool with 5 threads.
    pub fn new() -> Timer {
        Timer::with_capacity(5)
    }

    /// Creates a new timer backed by a thread pool with `capacity` threads.
    pub fn with_capacity(capacity: u32) -> Timer {
        Timer {
            pool: ScheduledThreadPool::fixed_size(capacity),
        }
    }

    /// Returns a `Future` that will be completed in `ms` milliseconds
    pub fn timeout_ms(&self, ms: u32) -> Future<(), ()> {
        let (tx, rx) = Future::pair();
        let pool = self.pool.clone();
        let now = Instant::now();

        tx.receive(move |res| {
            if let Ok(tx) = res {
                let elapsed = now.elapsed();
                let elapsed_ms = elapsed.as_secs() as u32 * 1000 + elapsed.subsec_nanos() / 1000_000;

                if elapsed_ms >= ms {
                    tx.complete(());
                    return;
                }

                pool.schedule_ms(ms - elapsed_ms, move || {
                    tx.complete(());
                });
            }
        });

        rx
    }

    /// Return a `Stream` with values realized every `ms` milliseconds.
    pub fn interval_ms(&self, ms: u32) -> Stream<(), ()> {
        let (tx, rx) = Stream::pair();
        let pool = self.pool.clone();
        let interval = Duration::from_millis(ms as u64);
        let next = Instant::now() + interval;

        do_interval(pool, tx, next, interval);

        rx
    }
}

/// Processes the interval stream
fn do_interval<S>(pool: ScheduledThreadPool,
                  sender: S,
                  next: Instant,
                  interval: Duration)
        where S: Async<Value=Sender<(), ()>> {

    sender.receive(move |res| {
        if let Ok(sender) = res {
            let now = Instant::now();
            let delay = next - now;
            let next = next + interval;
            let pool2 = pool.clone();
            let delay_ms = delay.as_secs() as u32 * 1000 + delay.subsec_nanos() / 1000_000;

            pool.schedule_ms(delay_ms, move || {
                let busy = sender.send(());
                do_interval(pool2, busy, next, interval);
            });
        }
    });
}

impl Clone for Timer {
    fn clone(&self) -> Timer {
        Timer { pool: self.pool.clone() }
    }
}
