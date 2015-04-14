use {Async, Future};
use syncbox::ScheduledThreadPool;
use time::SteadyTime;

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
        let now = SteadyTime::now();

        tx.receive(move |res| {
            if let Ok(tx) = res {
                let elapsed = (SteadyTime::now() - now).num_milliseconds() as u32;

                if elapsed >= ms {
                    tx.complete(());
                    return;
                }

                pool.schedule_ms(ms - elapsed, move || {
                    tx.complete(());
                });
            }
        });

        rx
    }
}

impl Clone for Timer {
    fn clone(&self) -> Timer {
        Timer { pool: self.pool.clone() }
    }
}
