use core::marker::PhantomData;
use static_assertions::assert_type_eq_all;

#[cfg(feature = "use-typenum")]
mod nats_typenum;
#[cfg(feature = "use-typenum")]
pub use nats_typenum::*;

#[cfg(not(feature = "use-typenum"))]
pub use crate::peano::*;

// Here we define a type-level linked list.
// we can make lists like the following:
// Cons<u8, Cons<u16, Nil>>
pub struct Nil;
pub struct Cons<H, R>(PhantomData<(H, R)>);

// Define a struct to hold the memory of the turing machine.
// This is formatted as a zipper list which means that we're
// representing the list and the index into it in a single
// data structure.
// Here is a diagram of [1, 2, 3, 4, 5] where index 2 (holding a
// 3) is the current element.
// ([2, 1], 3, [4, 5])
pub struct Memory<L, V, R>(PhantomData<(L, V, R)>);
// Represent the full state of a machine including the tape (Memory)
// and I/O. Inp is a cons-list of yet to be read input data
// and Out is a cons-list of output data where the head of the list
// represents the most recently written thing
pub struct Machine<Mem, Inp, Out>(PhantomData<(Mem, Inp, Out)>);

/// The fundemental operations for out VM
pub mod ops {
    use core::marker::PhantomData;
    /// Increment the current head
    pub struct Incr;
    /// Decrement the current head
    pub struct Decr;
    /// Move the head to the right
    pub struct Right;
    /// Move the head to the left
    pub struct Left;
    /// As long as the current head is non-zero, repeat body
    pub struct Loop<Body>(PhantomData<Body>);
    /// Write the current head to Out
    pub struct Write;
    /// Pop the input and write it to the current head
    pub struct Read;
}

/// A type level function MemoryOp : (Memory, Instruction) -> Memory
pub trait MemoryOp<Mem> {
    type Next;
}

/// Increment the head of the tape
/// Memory<L, V, R> -> Memory<L, V+1, R>
impl<Ml, Mv: IncrNat, Mr> MemoryOp<ops::Incr> for Memory<Ml, Mv, Mr> {
    type Next = Memory<Ml, <Mv as IncrNat>::Result, Mr>;
}

/// Decrement the head of the tape
/// Memory<L, V, R> -> Memory<L, V-1, R>
impl<Ml, Mv: DecrNat, Mr> MemoryOp<ops::Decr> for Memory<Ml, Mv, Mr> {
    type Next = Memory<Ml, <Mv as DecrNat>::Result, Mr>;
}

/// Move the tape to the right
/// Memory<L, V, Mr1::Mrr> -> Memory<V::L, Mr1, Mrr>
impl<Ml, Mv, Mr1, Mrr> MemoryOp<ops::Right> for Memory<Ml, Mv, Cons<Mr1, Mrr>> {
    type Next = Memory<Cons<Mv, Ml>, Mr1, Mrr>;
}

/// Move the tape to the right
/// (if we hit the end of the tape, define it as a zero)
/// Memory<L, V, Nil> -> Memory<V::L, 0, Nil>
impl<Ml, Mv> MemoryOp<ops::Right> for Memory<Ml, Mv, Nil> {
    type Next = Memory<Cons<Mv, Ml>, Zero, Nil>;
}

/// Move the tape to the left
/// Memory<Ml1::Mlr, V, R> -> Memory<Mlr, Ml1, V::R>
impl<Mr, Mv, Ml1, Mlr> MemoryOp<ops::Left> for Memory<Cons<Ml1, Mlr>, Mv, Mr> {
    type Next = Memory<Mlr, Ml1, Cons<Mv, Mr>>;
}

/// Move the tape to the left
/// Memory<Nil, V, R> -> Memory<Nil, 0, V::R>
impl<Mr, Mv> RunMachine<ops::Left> for Memory<Nil, Mv, Mr> {
    type Next = Memory<Nil, Zero, Cons<Mv, Mr>>;
}

/// Helper function for Loops
/// (False, Body, Rest, Machine) -> Machine.RunMachine(Rest)
/// (True, Body, Rest, Machine) -> Machine.RunMachine(Body).RunMachine( Loop(Body)::Rest )
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
    <M as RunMachine<Body>>::Next: RunMachine<Cons<ops::Loop<Body>, Rest>>,
    M: RunMachine<Body>,
{
    type Result = <<M as RunMachine<Body>>::Next as RunMachine<Cons<ops::Loop<Body>, Rest>>>::Next;
}

/// Machine.RunMachine(Loop(Body)::Rest) -> RunMachineIntermediate(IsHeadNonZero?, Body, Rest, Machine)
impl<Body, Rest, Ml, Mr, In, Out, N> RunMachine<Cons<ops::Loop<Body>, Rest>>
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

/// Run the Program Prog on the implementing machine
pub trait RunMachine<Prog> {
    type Next;
}

/// Running a machine with no more instructions to execute is our base case, no-op
impl<T> RunMachine<Nil> for T {
    type Next = T;
}

/// If the current instruction deals purely with memory then run the memory op
impl<Action, Rest, Mem, In, Out> RunMachine<Cons<Action, Rest>> for Machine<Mem, In, Out>
where
    Mem: MemoryOp<Action>,
    Machine<<Mem as MemoryOp<Action>>::Next, In, Out>: RunMachine<Rest>,
{
    type Next = <Machine<<Mem as MemoryOp<Action>>::Next, In, Out> as RunMachine<Rest>>::Next;
}

/// Write the current head to Out
impl<Pn, Ml, Mv, Mr, In, Out> RunMachine<Cons<ops::Write, Pn>> for Machine<Memory<Ml, Mv, Mr>, In, Out>
where
    Machine<Memory<Ml, Mv, Mr>, In, Cons<Mv, Out>>: RunMachine<Pn>,
{
    type Next = <Machine<Memory<Ml, Mv, Mr>, In, Cons<Mv, Out>> as RunMachine<Pn>>::Next;
}

/// Nothing to read writes 0 to the memory head
impl<Pn, Ml, Mv, Mr, Out> RunMachine<Cons<ops::Read, Pn>> for Machine<Memory<Ml, Mv, Mr>, Nil, Out>
where
    Machine<Memory<Ml, Zero, Mr>, Nil, Out>: RunMachine<Pn>,
{
    type Next = <Machine<Memory<Ml, Zero, Mr>, Nil, Out> as RunMachine<Pn>>::Next;
}

/// Pop the top off of Inp and write it to the memory head
impl<Pn, Ml, Mv, Mr, Inv, Inr, Out> RunMachine<Cons<ops::Read, Pn>>
    for Machine<Memory<Ml, Mv, Mr>, Cons<Inv, Inr>, Out>
where
    Machine<Memory<Ml, Inv, Mr>, Inr, Out>: RunMachine<Pn>,
{
    type Next = <Machine<Memory<Ml, Inv, Mr>, Inr, Out> as RunMachine<Pn>>::Next;
}

/// Create a new machine with the given input list and empty memory and output
pub type MakeMachine<Input> = Machine<Memory<Nil, Zero, Nil>, Input, Nil>;

fn _test_incr_write() {
    type StateInital = MakeMachine<Nil>;
    type StateExpected = Machine<Memory<Nil, One, Nil>, Nil, Cons<One, Nil>>;
    type Program = Cons<ops::Incr, Cons<ops::Write, Nil>>;

    assert_type_eq_all!(<StateInital as RunMachine<Program>>::Next, StateExpected);
}

fn _test_incr_decr() {
    type StateInital = Machine<Memory<Nil, Zero, Nil>, Nil, Nil>;
    type StateExpected1 = Machine<Memory<Nil, One, Nil>, Nil, Nil>;
    type StateExpected2 = Machine<Memory<Nil, Two, Nil>, Nil, Nil>;

    type Program1 = Cons<ops::Incr, Nil>;
    type Program2 = Cons<ops::Incr, Cons<ops::Incr, Nil>>;
    type Program3 = Cons<ops::Incr, Cons<ops::Incr, Cons<ops::Decr, Nil>>>;

    assert_type_eq_all!(<StateInital as RunMachine<Program1>>::Next, StateExpected1);
    assert_type_eq_all!(<StateInital as RunMachine<Program2>>::Next, StateExpected2);
    assert_type_eq_all!(<StateInital as RunMachine<Program3>>::Next, StateExpected1);
}

fn _test_move_left_right() {
    type StateInital = Machine<Memory<Nil, One, Nil>, Nil, Nil>;
    type StateExpected1 = Machine<Memory<Cons<One, Nil>, Zero, Nil>, Nil, Nil>;
    type StateExpected2 = Machine<Memory<Nil, One, Cons<Zero, Nil>>, Nil, Nil>;

    type Program1 = Cons<ops::Right, Nil>;
    type Program2 = Cons<ops::Right, Cons<ops::Left, Nil>>;

    assert_type_eq_all!(<StateInital as RunMachine<Program1>>::Next, StateExpected1);
    assert_type_eq_all!(<StateInital as RunMachine<Program2>>::Next, StateExpected2);
}

fn _test_loop() {
    type StateInital = Machine<Memory<Nil, Two, Nil>, Nil, Nil>;
    type StateExpected = Machine<Memory<Nil, One, Nil>, Nil, Nil>;
    type Program = Cons<ops::Loop<Cons<ops::Decr, Nil>>, Cons<ops::Incr, Nil>>;

    assert_type_eq_all!(<StateInital as RunMachine<Program>>::Next, StateExpected);
}

pub trait GetOutput {
    type Output;
}

impl<Mem, In, Out> GetOutput for Machine<Mem, In, Out> {
    type Output = Out;
}

/// Construct the runtime value that T encodes
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
