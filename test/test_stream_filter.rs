use eventual::*;
use std::sync::mpsc::channel;
use super::{nums};

#[test]
pub fn test_stream_map_filter() {
    let s = nums::<()>(0, 5).filter(move |i| i % 2 == 0);
    let (tx, rx) = channel();

    s.each(move |i| tx.send(i).unwrap()).fire();

    let vals: Vec<usize> = rx.iter().collect();
    assert_eq!([0, 2, 4], vals);
}
