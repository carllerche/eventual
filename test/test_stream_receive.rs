use super::{spawn, sleep_ms};
use eventual::*;
use std::sync::mpsc::{self, channel};

#[test]
pub fn test_one_shot_stream_async() {
}

#[test]
pub fn test_one_shot_stream_await() {
    let (gen, stream) = Stream::<&'static str, ()>::pair();
    let (tx, rx) = channel();

    spawn(move || {
        debug!(" ~~ awaiting on stream ~~");
        match stream.await() {
            Ok(Some((h, _))) => tx.send(h).unwrap(),
            _ => panic!("nope"),
        }
    });

    sleep_ms(50);

    debug!(" ~~ Sender::send(\"hello\") ~~");
    gen.send("hello");
    assert_eq!("hello", rx.recv().unwrap());
}

#[test]
pub fn test_one_shot_stream_done() {
    let (sender, mut stream) = Stream::<&'static str, ()>::pair();
    let (tx, rx) = channel();

    spawn(move || {
        while let Ok(Some((v, rest))) = stream.await() {
            tx.send(v).unwrap();
            stream = rest;
        }
    });

    sender.send("hello")
        .await().unwrap();

    let vals: Vec<&'static str> = rx.iter().collect();
    assert_eq!(["hello"], vals);
}

#[test]
pub fn test_stream_receive_before_generate_interest_async() {
    let (sender, stream) = Stream::pair();
    let (tx, rx) = channel();
    let (eof_t, eof_r) = channel();

    fn do_receive(stream: Stream<&'static str, ()>,
                  tx: mpsc::Sender<&'static str>,
                  eof: mpsc::Sender<bool>) {
        stream.receive(move |res| {
            match res {
                Ok(Some((h, rest))) => {
                    tx.send(h).unwrap();
                    do_receive(rest, tx, eof);
                }
                Ok(None) => eof.send(true).unwrap(),
                _ => panic!("unexpected error"),
            }
        });
    }

    do_receive(stream, tx, eof_t);

    // Transfer the producer out
    let (txp, rxp) = channel();

    debug!(" ~~ Sender::receive ~~");
    sender
        .and_then(move |sender| sender.send("hello"))
        .and_then(move |sender| txp.send(sender).unwrap())
        .await().unwrap();

    // Receive the first message
    assert_eq!("hello", rx.recv().unwrap());

    // New channel to transfer the producer out
    let (txp, rxp2) = channel();

    // Get the producer, wait for readiness, and write another message
    debug!(" ~~ Sender::receive ~~");
    rxp.recv().unwrap()
        .and_then(move |sender| sender.send("world"))
        .and_then(move |sender| txp.send(sender).unwrap())
        .await().unwrap();

    debug!(" ~~ WAITING ON WORLD ~~");

    // Receive the second message
    assert_eq!("world", rx.recv().unwrap());

    debug!(" ~~ Sender::receive ~~");
    rxp2.recv().unwrap().receive(move |p| {
        drop(p.unwrap());
    });

    debug!(" ~~ Waiting on None ~~");
    assert!(eof_r.recv().unwrap());
    assert!(rx.recv().is_err());
}

#[test]
pub fn test_stream_produce_interest_before_receive_async() {
    let (sender, mut stream) = Stream::<&'static str, ()>::pair();
    let (txp, rxp) = channel();

    debug!(" ~~ Sender::receive #1 ~~");
    sender.receive(move |res| {
        debug!("  ~~~ Sender interest received, sending message");
        let sender = res.unwrap();
        sender.send("hello").receive(move |res| {
            debug!(" ~~~ Sender ready again ~~");
            txp.send(res.unwrap()).unwrap();
        });
    });

    let (tx, rx) = channel();

    debug!(" ~~ Stream::receive #1 ~~");
    stream.receive(move |res| {
        debug!(" ~~ Stream receive ~~");
        tx.send(res).unwrap()
    });

    match rx.recv().unwrap() {
        Ok(Some((v, rest))) => {
            debug!(" ~~ Stream received over channel ~~");
            assert_eq!(v, "hello");
            stream = rest;
        }
        _ => panic!("nope"),
    }

    let (tx, rx) = channel();

    debug!(" ~~ Stream::receive #2 ~~");
    stream.receive(move |res| tx.send(res).unwrap());

    let (txp2, rxp2) = channel();

    debug!(" ~~ Sender::receive #2 ~~");
    rxp.recv().unwrap().receive(move |res| {
        let sender = res.unwrap();
        sender.send("world").receive(move |res| {
            txp2.send(res.unwrap()).unwrap();
        });
    });

    debug!("WAAAAAAAAAT");
    match rx.recv().unwrap() {
        Ok(Some((v, rest))) => {
            debug!(" ~~ Stream received over channel TWO ~~");
            assert_eq!(v, "world");
            stream = rest;
        }
        _ => panic!("nope"),
    }

    let (tx, rx) = channel();

    debug!(" ~~ Stream::receive #3 ~~");
    stream.receive(move |res| tx.send(res).unwrap());

    rxp2.recv().unwrap().receive(move |p| drop(p.unwrap()));

    match rx.recv().unwrap() {
        Ok(None) => {}
        _ => panic!("nope"),
    }
}

#[test]
pub fn test_stream_produce_before_receive_async() {
}

#[test]
pub fn test_stream_send_then_done_before_receive() {
    // Currently, Sender::done() cannot be called before the previous
    // value is consumed. It would be ideal to be able to signal that the
    // stream is done without having to do another producer callback
    // iteration.
}

#[test]
pub fn test_recursive_receive() {
    let (gen, s) = Stream::<i32, ()>::pair();
    let (tx, rx) = channel();

    fn consume(s: Stream<i32, ()>, tx: mpsc::Sender<i32>) {
        debug!(" ~~~~ CONSUME ENTER ~~~~~ ");
        s.receive(move |res| {
            debug!(" ~~~ CONSUME CALLBACK ENTER ~~~~~ ");
            if let Some((v, rest)) = res.unwrap() {
                tx.send(v).unwrap();
                consume(rest, tx);
            }
            debug!(" ~~~ CONSUME CALLBACK EXIT ~~~~~ ");
        });
        debug!(" ~~~~ CONSUME EXIT ~~~~~ ");
    }

    fn produce(p: Sender<i32, ()>, n: i32) {
        debug!(" ~~~~ PRODUCE ENTER ~~~~~ ");
        if n > 20_000 {
            return;
        }

        p.send(n).receive(move |res| {
            debug!("~~~~~~ SEND COMPLETE ~~~~~~~~");
            produce(res.unwrap(), n + 1)
        });
        debug!(" ~~~~ PRODUCE EXIT ~~~~~ ");
    }

    consume(s, tx);
    produce(gen, 1);

    debug!(" ~~~~~ BLOCKING ~~~~~~");
    let vals: Vec<i32> = rx.iter().collect();
    assert_eq!(vals.len(), 20_000);
}

#[test]
pub fn test_complete_during_consumer_receive() {
    let (sender, stream) = Stream::<i32, ()>::pair();
    let (tx, rx) = channel();

    sender.send(1).receive(|sender| {
        sender.unwrap().send(2);
    }).fire();

    stream.receive(move |res| {
        match res.unwrap() {
            Some((v, rest)) => {
                assert_eq!(v, 1);

                rest.receive(move |res| {
                    match res.unwrap() {
                        Some((v, _)) => {
                            assert_eq!(v, 2);
                            tx.send("done").unwrap();
                        }
                        _ => panic!("unexpected value"),
                    }
                });
            }
            _ => panic!("unexpected value"),
        }
    });

    assert_eq!("done", rx.recv().unwrap());
}

#[test]
#[ignore]
pub fn test_recurse_and_cancel() {
    // unimplemented
}
