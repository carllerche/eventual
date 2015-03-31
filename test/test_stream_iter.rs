use eventual::*;

#[test]
pub fn test_stream_iter_async_producer() {
    let (tx, rx) = Stream::pair();

    fn gen(tx: Sender<i32, ()>, i: i32) {
        tx.receive(move |res| {
            let tx = res.unwrap();

            if i < 5 {
                tx.send(i).receive(move |res| {
                    gen(res.unwrap(), i + 1);
                });
            }
        })
    }

    gen(tx, 0);

    let vals: Vec<i32> = rx.iter().collect();
    assert_eq!([0, 1, 2, 3, 4], vals);
}
