use {Async, Future, Stream, Sender};
use syncbox::ScheduledThreadPool;
use time::{SteadyTime, Duration};

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

    /// Return a `Stream` with values realized every `ms` milliseconds.
    pub fn interval_ms(&self, ms: u32) -> Stream<(), ()> {
        let (tx, rx) = Stream::pair();
        let pool = self.pool.clone();
        let interval = Duration::milliseconds(ms as i64);
        let next = SteadyTime::now() + interval;

        do_interval(pool, tx, next, interval);

        rx
    }
}

/// Processes the interval stream
fn do_interval<S>(pool: ScheduledThreadPool,
                  sender: S,
                  next: SteadyTime,
                  interval: Duration)
        where S: Async<Value=Sender<(), ()>> {

    sender.receive(move |res| {
        if let Ok(sender) = res {
            let now = SteadyTime::now();
            let delay = next - now;
            let next = next + interval;
            let pool2 = pool.clone();

            pool.schedule_ms(delay.num_milliseconds() as u32, move || {
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
