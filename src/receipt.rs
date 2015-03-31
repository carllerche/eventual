use super::Async;
use super::core::Core;
use std::marker::PhantomData;

pub struct Receipt<A: Async> {
    core: Option<Core<A::Value, A::Error>>,
    count: u64,
    marker: PhantomData<A>,
}

unsafe impl<A: Async> Send for Receipt<A> { }

pub fn new<A, T: Send, E: Send>(core: Core<T, E>, count: u64) -> Receipt<A>
        where A: Async<Value=T, Error=E> {
    Receipt {
        core: Some(core),
        count: count,
        marker: PhantomData,
    }
}

pub fn none<A: Async>() -> Receipt<A> {
    Receipt {
        core: None,
        count: 0,
        marker: PhantomData,
    }
}

pub fn parts<A, T: Send, E: Send>(receipt: Receipt<A>) -> (Option<Core<T, E>>, u64)
        where A: Async<Value=T, Error=E> {
    (receipt.core, receipt.count)
}
