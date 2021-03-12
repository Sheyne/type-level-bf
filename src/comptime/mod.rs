use core::marker::PhantomData;
use static_assertions::assert_type_eq_all;

#[cfg(feature = "use-typenum")]
mod nats_typenum;
#[cfg(feature = "use-typenum")]
pub use nats_typenum::*;

#[cfg(not(feature = "use-typenum"))]
mod nats_peano;
#[cfg(not(feature = "use-typenum"))]
pub use nats_peano::*;



pub struct Nil;
pub struct Cons<H, R>(PhantomData<(H, R)>);

pub struct Memory<L, V, R>(PhantomData<(L, V, R)>);
pub struct Machine<Mem, Inp, Out>(PhantomData<(Mem, Inp, Out)>);

pub struct Incr;
pub struct Decr;
pub struct Right;
pub struct Left;
pub struct Loop<Body>(PhantomData<Body>);
pub struct Write;
pub struct Read;

pub trait MemoryOp<Mem> {
    type Next;
}

impl<Ml, Mv, Mr> MemoryOp<Incr> for Memory<Ml, Mv, Mr>
where
    Mv: IncrNat,
{
    type Next = Memory<Ml, <Mv as IncrNat>::Result, Mr>;
}

impl<Ml, Mv, Mr> MemoryOp<Decr> for Memory<Ml, Mv, Mr>
where
    Mv: DecrNat,
{
    type Next = Memory<Ml, <Mv as DecrNat>::Result, Mr>;
}

impl<Ml, Mv, Mr1, Mrr> MemoryOp<Right> for Memory<Ml, Mv, Cons<Mr1, Mrr>> {
    type Next = Memory<Cons<Mv, Ml>, Mr1, Mrr>;
}

impl<Ml, Mv> MemoryOp<Right> for Memory<Ml, Mv, Nil> {
    type Next = Memory<Cons<Mv, Ml>, Zero, Nil>;
}

impl<Mr, Mv, Ml1, Mlr> MemoryOp<Left> for Memory<Cons<Ml1, Mlr>, Mv, Mr> {
    type Next = Memory<Mlr, Ml1, Cons<Mv, Mr>>;
}

impl<Mr, Mv> RunMachine<Left> for Memory<Nil, Mv, Mr> {
    type Next = Memory<Nil, Zero, Cons<Mv, Mr>>;
}

pub trait RunMachineIntermediate<Body, Rest, T> {
    type Result;
}

impl<Body, Rest, M> RunMachineIntermediate<Body, Rest, M> for False
where
    M: RunMachine<Rest>,
{
    type Result = <M as RunMachine<Rest>>::Next;
}

impl<Body, Rest, M> RunMachineIntermediate<Body, Rest, M> for True
where
    <M as RunMachine<Body>>::Next: RunMachine<Cons<Loop<Body>, Rest>>,
    M: RunMachine<Body>,
{
    type Result = <<M as RunMachine<Body>>::Next as RunMachine<Cons<Loop<Body>, Rest>>>::Next;
}

impl<Body, Rest, Ml, Mr, In, Out, N> RunMachine<Cons<Loop<Body>, Rest>>
    for Machine<Memory<Ml, N, Mr>, In, Out>
where
    N: IsNonZero,
    <N as IsNonZero>::Result:
        RunMachineIntermediate<Body, Rest, Machine<Memory<Ml, N, Mr>, In, Out>>,
{
    type Next = <<N as IsNonZero>::Result as RunMachineIntermediate<
        Body,
        Rest,
        Machine<Memory<Ml, N, Mr>, In, Out>,
    >>::Result;
}

pub trait RunMachine<Prog> {
    type Next;
}

impl<T> RunMachine<Nil> for T {
    type Next = T;
}

impl<Action, Rest, Mem, In, Out> RunMachine<Cons<Action, Rest>> for Machine<Mem, In, Out>
where
    Mem: MemoryOp<Action>,
    Machine<<Mem as MemoryOp<Action>>::Next, In, Out>: RunMachine<Rest>,
{
    type Next = <Machine<<Mem as MemoryOp<Action>>::Next, In, Out> as RunMachine<Rest>>::Next;
}

impl<Pn, Ml, Mv, Mr, In, Out> RunMachine<Cons<Write, Pn>> for Machine<Memory<Ml, Mv, Mr>, In, Out>
where
    Machine<Memory<Ml, Mv, Mr>, In, Cons<Mv, Out>>: RunMachine<Pn>,
{
    type Next = <Machine<Memory<Ml, Mv, Mr>, In, Cons<Mv, Out>> as RunMachine<Pn>>::Next;
}

impl<Pn, Ml, Mv, Mr, Out> RunMachine<Cons<Read, Pn>> for Machine<Memory<Ml, Mv, Mr>, Nil, Out>
where
    Machine<Memory<Ml, Zero, Mr>, Nil, Out>: RunMachine<Pn>,
{
    type Next = <Machine<Memory<Ml, Zero, Mr>, Nil, Out> as RunMachine<Pn>>::Next;
}

impl<Pn, Ml, Mv, Mr, Inv, Inr, Out> RunMachine<Cons<Read, Pn>>
    for Machine<Memory<Ml, Mv, Mr>, Cons<Inv, Inr>, Out>
where
    Machine<Memory<Ml, Inv, Mr>, Inr, Out>: RunMachine<Pn>,
{
    type Next = <Machine<Memory<Ml, Inv, Mr>, Inr, Out> as RunMachine<Pn>>::Next;
}

pub type MakeMachine<Input> = Machine<Memory<Nil, Zero, Nil>, Input, Nil>;

fn _test_incr_write() {
    type StateInital = MakeMachine<Nil>;
    type StateExpected = Machine<Memory<Nil, One, Nil>, Nil, Cons<One, Nil>>;
    type Program = Cons<Incr, Cons<Write, Nil>>;

    assert_type_eq_all!(<StateInital as RunMachine<Program>>::Next, StateExpected);
}

fn _test_incr_decr() {
    type StateInital = Machine<Memory<Nil, Zero, Nil>, Nil, Nil>;
    type StateExpected1 = Machine<Memory<Nil, One, Nil>, Nil, Nil>;
    type StateExpected2 = Machine<Memory<Nil, Two, Nil>, Nil, Nil>;

    type Program1 = Cons<Incr, Nil>;
    type Program2 = Cons<Incr, Cons<Incr, Nil>>;
    type Program3 = Cons<Incr, Cons<Incr, Cons<Decr, Nil>>>;

    assert_type_eq_all!(<StateInital as RunMachine<Program1>>::Next, StateExpected1);
    assert_type_eq_all!(<StateInital as RunMachine<Program2>>::Next, StateExpected2);
    assert_type_eq_all!(<StateInital as RunMachine<Program3>>::Next, StateExpected1);
}

fn _test_move_left_right() {
    type StateInital = Machine<Memory<Nil, One, Nil>, Nil, Nil>;
    type StateExpected1 = Machine<Memory<Cons<One, Nil>, Zero, Nil>, Nil, Nil>;
    type StateExpected2 = Machine<Memory<Nil, One, Cons<Zero, Nil>>, Nil, Nil>;

    type Program1 = Cons<Right, Nil>;
    type Program2 = Cons<Right, Cons<Left, Nil>>;

    assert_type_eq_all!(<StateInital as RunMachine<Program1>>::Next, StateExpected1);
    assert_type_eq_all!(<StateInital as RunMachine<Program2>>::Next, StateExpected2);
}

fn _test_loop() {
    type StateInital = Machine<Memory<Nil, Two, Nil>, Nil, Nil>;
    type StateExpected = Machine<Memory<Nil, One, Nil>, Nil, Nil>;
    type Program = Cons<Loop<Cons<Decr, Nil>>, Cons<Incr, Nil>>;

    assert_type_eq_all!(<StateInital as RunMachine<Program>>::Next, StateExpected);
}

pub trait GetOutput {
    type Output;
}

impl<Mem, In, Out> GetOutput for Machine<Mem, In, Out> {
    type Output = Out;
}

pub trait Make<T> {
    fn make() -> T;
}

impl<T> Make<Vec<T>> for Nil {
    fn make() -> Vec<T> {
        vec![]
    }
}

impl<H, Tail, T> Make<Vec<T>> for Cons<H, Tail>
where
    H: Make<T>,
    Tail: Make<Vec<T>>,
{
    fn make() -> Vec<T> {
        let mut res = Tail::make();
        res.push(H::make());
        res
    }
}