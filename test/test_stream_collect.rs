use eventual::*;
use super::nums;

#[test]
pub fn test_stream_collect_async() {
    let s = nums::<()>(0, 5).collect();
    assert_eq!([0, 1, 2, 3, 4], s.await().unwrap());
}
