use super::{futures, nums};
use eventual::Async;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;

#[test]
pub fn test_process_sync_result() {
    let (tx, rx) = channel();
    let counter = Counter::new(0);

    {
        let counter = counter.clone();

        nums::<()>(0, 4).process(3, move |val| {
            assert_eq!(val, counter.inc());
            Ok(2 * val)
        }).each(move |val| {
            tx.send(val).unwrap();
        }).fire();
    }

    assert_eq!(4, counter.val());

    let vals: Vec<usize> = rx.iter().collect();
    assert_eq!([0, 2, 4, 6], &vals[..]);
}

#[test]
pub fn test_process_async_result_less_than_buffer() {
    let (mut completes, futures) = futures(2);
    let (tx, rx) = channel();
    let counter = Counter::new(0);

    {
        let counter = counter.clone();

        nums::<()>(0, 2).process(2, move |val| {
            assert_eq!(val, counter.inc());
            futures.try_recv().unwrap()
        }).each(move |val| {
            tx.send(val).unwrap();
        }).fire();
    }

    assert_eq!(2, counter.val());

    completes.remove(1).complete(123);
    completes.remove(0).complete(234);

    let vals: Vec<usize> = rx.iter().collect();
    assert_eq!([123, 234], &vals[..]);
}

#[test]
pub fn test_process_async_result_more_than_buffer() {
    let (mut completes, futures) = futures(4);
    let (tx, rx) = channel();
    let counter = Counter::new(0);

    {
        let counter = counter.clone();

        nums::<()>(0, 4)
            .process(2, move |val| {
                assert_eq!(val, counter.inc());
                futures.try_recv().unwrap()
            }).each(move |val| {
                tx.send(val).unwrap();
            }).fire();
    }

    assert_eq!(2, counter.val());

    completes.remove(2).complete(12);

    // Should not receive a value yet
    assert!(rx.try_recv().is_err(), "expected no values yet");

    completes.remove(1).complete(23);
    completes.remove(0).complete(34);
    completes.remove(0).complete(45);

    let vals: Vec<usize> = rx.iter().collect();
    assert_eq!([23, 12, 34, 45], &vals[..]);
}

#[test]
pub fn test_process_drop_stream_before_second_val_complete() {
    let (mut completes, futures) = futures(4);
    let counter = Counter::new(0);

    let stream = {
        let counter = counter.clone();

        nums::<()>(0, 4).process(2, move |val| {
            assert_eq!(val, counter.inc());
            futures.try_recv().unwrap()
        })
    };

    completes.remove(1).complete(123);

    // Take the first value
    match stream.await() {
        Ok(Some((v, _))) => {
            assert_eq!(v, 123);
        }
        _ => panic!("unexpected stream head"),
    }

    assert_eq!(3, counter.val());
}

#[test]
pub fn test_process_drop_stream_after_second_val_complete() {
    let (mut completes, futures) = futures(4);
    let counter = Counter::new(0);

    let stream = {
        let counter = counter.clone();

        nums::<()>(0, 4).process(2, move |val| {
            assert_eq!(val, counter.inc());
            futures.try_recv().unwrap()
        })
    };

    completes.remove(1).complete(123);
    completes.remove(0).complete(123);

    // Take the first value
    match stream.await() {
        Ok(Some((v, _))) => {
            assert_eq!(v, 123);
        }
        _ => panic!("unexpected stream head"),
    }

    assert_eq!(3, counter.val());
}

#[test]
pub fn test_process_failed_future() {
    let (mut completes, futures) = futures(4);
    let (tx1, rx) = channel();
    let tx2 = tx1.clone();
    let counter = Counter::new(0);

    {
        let counter = counter.clone();

        nums(0, 4).process(2, move |val| {
            assert_eq!(val, counter.inc());
            futures.try_recv().unwrap()
        }).each(move |val| {
            tx1.send(Ok(val)).unwrap();
        }).or_else(move |err| {
            tx2.send(Err(err)).unwrap();
            Ok::<(), ()>(())
        })
        .fire();
    }

    completes.remove(1).complete(123);
    completes.remove(0).fail("nope");

    assert_eq!(Ok(123), rx.recv().unwrap());
    assert_eq!(Err("nope"), rx.recv().unwrap());
    // TODO: For now this is 4, but ideally it would be 3 (no further values
    // are processed when the stream fails even if the buffer has space. This
    // is not currently implemented though.
    assert_eq!(4, counter.val());
    assert!(rx.recv().is_err());
}

struct Counter {
    atomic: Arc<AtomicUsize>,
}

impl Counter {
    fn new(val: usize) -> Counter {
        Counter { atomic: Arc::new(AtomicUsize::new(val)) }
    }

    fn val(&self) -> usize {
        self.atomic.load(Ordering::Relaxed)
    }

    fn inc(&self) -> usize {
        self.atomic.fetch_add(1, Ordering::Relaxed)
    }
}

impl Clone for Counter {
    fn clone(&self) -> Counter {
        Counter { atomic: self.atomic.clone() }
    }
}
