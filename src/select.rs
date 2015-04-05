#![allow(unused_assignments)]

use super::{Async, Future, Complete, Cancel, AsyncError};
use syncbox::atomic::{self, AtomicU64, Ordering};
use std::cell::UnsafeCell;
use std::sync::Arc;
use std::{fmt, u32};

pub fn select<S: Select<E>, E: Send>(asyncs: S) -> Future<(u32, S), E> {
    let (complete, res) = Future::pair();

    // Don't do any work until the consumer registers interest in the completed
    // value.
    complete.receive(move |res| {
        if let Ok(complete) = res {
            asyncs.select(complete);
        }
    });

    res
}


pub trait Select<E: Send> : Send {
    fn select(self, complete: Complete<(u32, Self), E>);
}

// ## Flow
//
// * - Start in initial state, in a loop register a ready callback on each async.
//      - Place Cancel into slot
//      - Attempt to inc callbacks registered
//          - If fail, remove Cancel & Cancel callback
//              - Success: Place async in val storage
//
// - Ready callback will:
//      - Attempt to transition from Init state -> Won state.
//          - If success, become responsible for canceling callbacks registered
//            according to state val (place successfully canceled vals into
//            async store)
//      - Place async in vals storage
//
//
// ## TODO
//
// - On selection failure, cancel all other pending futures. This will only
//   matter if / when interest cancellation gets proxied up.
//
// - Switch to associated types over Index & Error
//     Blocked: rust-lang/rust#21664

trait Values<S: Select<E>, E> {
    type Tokens: Send;

    fn consume(&mut self) -> S;

    fn cancel_callbacks(&mut self, selected: u32, up_to: u32, tokens: &mut Self::Tokens) -> u32;
}

struct Selection<V: Values<S, E>, S: Select<E>, E: Send> {
    core: Arc<UnsafeCell<Core<V, S, E>>>,
}

unsafe impl<V: Values<S, E>, S: Select<E>, E: Send> Sync for Selection<V, S, E> {}
unsafe impl<V: Values<S, E>, S: Select<E>, E: Send> Send for Selection<V, S, E> {}

// This implementation is very unsafe
impl<V: Values<S, E>, S: Select<E>, E: Send> Selection<V, S, E> {

    /// Create a new Selection instance
    fn new(vals: V,
           tokens: V::Tokens,
           remaining: u32,
           complete: Complete<(u32, S), E>) -> Selection<V, S, E> {

        let core = Arc::new(UnsafeCell::new(Core {
            vals: vals,
            tokens: tokens,
            complete: Some(complete),
            state: AtomicState::new(remaining),
        }));

        Selection { core: core }
    }

    fn async_ready<A: Async<Error=E>>(&self, async: A, index: u32, slot: &mut Option<A>) {
        let mut handled = 1;

        debug!("selection async ready; index={}; is_err={}", index, async.is_err());

        // Attempt to "win" the selection process
        let (win, prev, curr) = self.core().state.try_win(index, async.is_err());

        if win {
            // Deregister all callbacks that have been registered up to now
            handled += self.core_mut().vals.cancel_callbacks(
                index, prev.callbacks_registered(), &mut self.core_mut().tokens);

            debug!("async val won select; handled={}", handled);

            if async.is_err() {
                debug!("first realized async val is error");

                let complete = self.core_mut().complete.take()
                    .expect("result future previously completed");

                if let Err(AsyncError::Failed(e)) = async.expect() {
                    debug!("execution error");
                    complete.fail(e);
                }

                // If cancellation error, don't do anything
                return;
            }
        }

        // The selection has already errored out, there is no need to track
        // anything further
        if curr.is_err() {
            return;
        }

        // Stash async value
        *slot = Some(async);
        self.dec_remaining(handled, curr);
    }

    // Track that a callback was just registered on an async value.
    fn track_callback<A: Async>(&self,
                                cancel: A::Cancel,
                                aref: &mut Option<A>,
                                cref: &mut Option<A::Cancel>) -> bool {

        // Store the cancel token
        *cref = Some(cancel);

        // Increment the registered callback count, if success, then there
        // isn't anything else to do.
        let (success, curr) = self.core().state.inc_callbacks_registered();

        if !success {
            // Select is done, cancel the callback that was just
            // registered
            let cancel = cref.take().expect("cancel token not present");

            // Attempt to cancel the callback
            if let Some(async) = cancel.cancel() {
                // Store off the async
                *aref = Some(async);

                self.dec_remaining(1, curr);

                return false;
            }
        }

        // Successfully tracked the callback
        return true;
    }

    // Track async values that have moved to the ready state
    fn dec_remaining(&self, count: u32, mut curr: State) {
        if count == 0 {
            debug!("dec_remaining -- nothing to do");
            return;
        }

        // Decrement the remaining count by the number of successfully
        // canceled callbacks
        curr = self.core().state.dec_remaining(count, curr);

        debug!("dec_remaining -- performed dec; state={:?}", curr);

        // TODO: This should not be in the win condition
        if curr.remaining() == 0 {
            // Last remaining async acquired, complete future
            self.complete(curr);
        }
    }

    fn complete(&self, curr: State) {
        // Use an Acquire fence to ensure that all async values are loaded into
        // memory
        atomic::fence(Ordering::Acquire);

        let core = self.core_mut();

        // Get the completer
        let complete = core.complete.take().expect("result future previously completed");

        // And complete the select
        complete.complete((curr.selected(), core.vals.consume()));
    }

    fn core(&self) -> &Core<V, S, E> {
        use std::mem;
        unsafe { mem::transmute(self.core.get()) }
    }

    fn core_mut(&self) -> &mut Core<V, S, E> {
        use std::mem;
        unsafe { mem::transmute(self.core.get()) }
    }
}

impl<V: Values<S, E>, S: Select<E>, E: Send> Selection<V, S, E> {
    fn clone(&self) -> Selection<V, S, E> {
        Selection { core: self.core.clone() }
    }
}

struct Core<V: Values<S, E>, S: Select<E>, E: Send> {
    vals: V,
    tokens: V::Tokens,
    complete: Option<Complete<(u32, S), E>>,
    state: AtomicState,
}

/*
 *
 * ===== Macro helpers =====
 *
 */

macro_rules! expr {
    ($e: expr) => { $e };
}

macro_rules! components {
    ($selection:ident, $pending:ident, $handled:ident, ($async:ident, $id:tt)) => {{
        if $pending {
            let s = $selection.clone();
            let c = $async.ready(move |a| {
                s.async_ready(a, expr!($id), expr!(&mut s.core_mut().vals.$id))
            });

            let core = $selection.core_mut();

            $pending = $selection.track_callback(
                c, expr!(&mut core.vals.$id), expr!(&mut core.tokens.$id));
        } else {
            expr!($selection.core_mut().vals.$id) = Some($async);
            $handled += 1;
        }
    }};

    ($selection:ident, $pending:ident, $handled:ident, ($async:ident, $id:tt), $($rest:tt),+) => {{
        components!($selection, $pending, $handled, ($async, $id));
        components!($selection, $pending, $handled, $($rest),*);
    }};
}

macro_rules! tuple_select {
    ($count:expr, $complete:ident, $vals:expr, $tokens:expr, $(($async:ident, $id:tt)),+) => {{
        // Create the selection
        let selection = Selection::new($vals, $tokens, $count, $complete);

        let mut pending = true;
        let mut handled = 0;

        components!(selection, pending, handled, $(($async, $id)),*);

        // Components

        if handled > 0 {
            selection.dec_remaining(handled, selection.core().state.load(Ordering::Relaxed));
        }
    }}
}

macro_rules! cancel_callbacks {
    ($s:ident, $selected:ident, $up_to:ident, $tokens:ident, $ret:ident, $i:tt) => {{
        {
            let i = expr!($i);
            if $selected != i && $up_to >= i+1 {
                let cancel = expr!($tokens.$i.take().expect("cancel token missing"));

                if let Some(async) = cancel.cancel() {
                    expr!($s.$i) = Some(async);
                    $ret += 1;
                }
            }
        }
    }};

    ($s:ident, $selected:ident, $up_to:ident, $tokens:ident, $ret:ident, $i:tt, $($rest:tt),+) => {{
        cancel_callbacks!($s, $selected, $up_to, $tokens, $ret, $i);
        cancel_callbacks!($s, $selected, $up_to, $tokens, $ret, $($rest),*);
    }};
}

/*
 *
 * ===== State =====
 *
 */

struct AtomicState {
    atomic: AtomicU64,
}

impl AtomicState {
    fn new(remaining: u32) -> AtomicState {
        let init = State::new(remaining);
        AtomicState { atomic: AtomicU64::new(init.as_u64()) }
    }

    fn load(&self, order: Ordering) -> State {
        State::load(self.atomic.load(order))
    }

    fn compare_and_swap(&self, old: State, new: State, order: Ordering) -> State {
        let actual = self.atomic.compare_and_swap(old.as_u64(), new.as_u64(), order);
        State::load(actual)
    }

    fn try_win(&self, index: u32, err: bool) -> (bool, State, State) {
        let mut curr = self.load(Ordering::Relaxed);

        loop {
            if curr.is_won() {
                return (false, curr, curr);
            }

            let next = if err {
                curr.as_err()
            } else {
                curr.as_won(index)
            };

            let actual = self.compare_and_swap(curr, next, Ordering::Acquire);

            if actual == curr {
                return (true, curr, next);
            }

            curr = actual;
        }
    }

    fn inc_callbacks_registered(&self) -> (bool, State) {
        let mut curr = self.load(Ordering::Relaxed);

        loop {
            if curr.is_won() {
                return (false, curr);
            }

            let next = curr.inc_callbacks_registered();
            let actual = self.compare_and_swap(curr, next, Ordering::Release);

            if actual == curr {
                return (true, next);
            }

            curr = actual;
        }
    }

    fn dec_remaining(&self, count: u32, mut curr: State) -> State {
        loop {
            assert!(curr.remaining() >= count, "curr={}; count={}", curr.remaining(), count);

            let next = curr.dec_remaining(count);
            let actual = self.compare_and_swap(curr, next, Ordering::Release);

            if actual == curr {
                debug!("dec_remaining -- transitioned state; from={:?}; to={:?}", curr, next);
                return next;
            }

            curr = actual;
        }
    }
}

/*
 * == State u64 ==
 *
 * 1 bit: Won (0: init, 1: won)
 * 1 bit: Error
 * 31 bits:
 *      - Remaining Async vals being waited on
 *
 * 31 bits:
 *      - init: Callbacks registered
 *      - Won: Index that won
 */
#[derive(Copy, Clone, PartialEq, Eq)]
struct State {
    val: u64,
}

impl fmt::Debug for State {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.is_won() {
            write!(fmt, "State[won={}; err={}; selected={}; remaining={}]",
                   true, self.is_err(), self.selected(), self.remaining())
        } else {
            write!(fmt, "State[won={}; err={}; callbacks={}; remaining={}]",
                   false, false, self.callbacks_registered(), self.remaining())
        }
    }
}

const WON_MASK: u64 = 1;
const ERR_MASK: u64 = 1 << 1;
const REM_MASK: u64 = (1 << 31) - 1;

impl State {
    fn new(remaining: u32) -> State {
        assert!(remaining <= (u32::MAX >> 1));
        State { val: (remaining as u64) << 2 }
    }

    fn load(val: u64) -> State {
        State { val: val }
    }

    fn is_won(&self) -> bool {
        self.val & WON_MASK == WON_MASK
    }

    fn as_won(&self, index: u32) -> State {
        assert!(index <= (u32::MAX >> 1));

        let val = WON_MASK | ((index as u64) << 33)
                | (self.val & (REM_MASK << 2))
                ;

        State { val: val }
    }

    fn is_err(&self) -> bool {
        self.val & ERR_MASK == ERR_MASK
    }

    fn as_err(&self) -> State {
        let val = WON_MASK | ERR_MASK
                | (self.val & (REM_MASK << 2))
                ;

        State { val: val }
    }

    fn callbacks_registered(&self) -> u32 {
        assert!(!self.is_won());
        self.most_significant_u31()
    }

    fn inc_callbacks_registered(&self) -> State {
        assert!(!self.is_won());
        State { val: self.val + (1 << 33) }
    }

    fn selected(&self) -> u32 {
        assert!(self.is_won());
        self.most_significant_u31()
    }

    fn remaining(&self) -> u32 {
        ((self.val >> 2) & REM_MASK) as u32
    }

    fn dec_remaining(&self, count: u32) -> State {
        State { val: self.val - ((count as u64) << 2) }
    }

    fn most_significant_u31(&self) -> u32 {
        (self.val >> 33) as u32
    }

    fn as_u64(&self) -> u64 {
        self.val
    }
}

/*
 *
 * ===== Select for Tuples =====
 *
 */

impl<A1: Async<Error=E>, A2: Async<Error=E>, E: Send> Select<E> for (A1, A2) {
    fn select(self, complete: Complete<(u32, (A1, A2)), E>) {
        let (a1, a2) = self;

        tuple_select!(
            2, complete,
            (None, None),
            (None, None),
            (a1, 0), (a2, 1));
    }
}

impl<A1: Async<Error=E>, A2: Async<Error=E>, E> Values<(A1, A2), E> for (Option<A1>, Option<A2>)
        where E: Send {

    type Tokens = (Option<A1::Cancel>, Option<A2::Cancel>);

    fn consume(&mut self) -> (A1, A2) {
        (self.0.take().unwrap(), self.1.take().unwrap())
    }

    fn cancel_callbacks(&mut self,
                        selected: u32,
                        up_to: u32,
                        tokens: &mut (Option<A1::Cancel>, Option<A2::Cancel>)) -> u32 {

        let mut ret = 0;

        cancel_callbacks!(
            self, selected, up_to, tokens, ret,
            0, 1);

        ret
    }
}

impl<A1: Async<Error=E>, A2: Async<Error=E>, A3: Async<Error=E>, E: Send> Select<E> for (A1, A2, A3) {
    fn select(self, complete: Complete<(u32, (A1, A2, A3)), E>) {
        let (a1, a2, a3) = self;

        tuple_select!(
            3, complete,
            (None, None, None),
            (None, None, None),
            (a1, 0), (a2, 1), (a3, 2));
    }
}

impl<A1: Async<Error=E>, A2: Async<Error=E>, A3: Async<Error=E>, E> Values<(A1, A2, A3), E> for (Option<A1>, Option<A2>, Option<A3>)
        where E: Send {

    type Tokens = (Option<A1::Cancel>, Option<A2::Cancel>, Option<A3::Cancel>);

    fn consume(&mut self) -> (A1, A2, A3) {
        (self.0.take().unwrap(), self.1.take().unwrap(), self.2.take().unwrap())
    }

    fn cancel_callbacks(&mut self,
                        selected: u32,
                        up_to: u32,
                        tokens: &mut (Option<A1::Cancel>, Option<A2::Cancel>, Option<A3::Cancel>)) -> u32 {

        let mut ret = 0;

        cancel_callbacks!(
            self, selected, up_to, tokens, ret,
            0, 1, 2);

        ret
    }
}
