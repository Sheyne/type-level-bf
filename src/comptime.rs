use core::marker::PhantomData;
use static_assertions::assert_type_eq_all;

// define a zero-sized struct called Zero,
// note that this struct has no fields and thus takes up
// no space at runtime. This means that it only *exists*
// as a compile time concept.
pub struct Zero;

// likewise, define a struct S<P> that represents a
// nature number. The number is the *S*uccessor of
// P. So we can encode 1 = S<0>, 2 = S<1> = S<S<0>>
// P.S., PhantomData is an artifact of Rust. Rust
// is not happy if you have an unused type parameter.
// i.e. you can't write: struct S<P>; like we could for
// Zero.
pub struct S<P>(PhantomData<P>);

// These type aliases aren't necessary. We can always
// write S<S<S<Zero>>> to mean Three, but this is more
// convenient. None of our axioms will need these definitions
// but the test code at the bottom will use them
pub type One = S<Zero>;
pub type Two = S<One>;
pub type Three = S<Two>;
pub type Four = S<Three>;
pub type Five = S<Four>;
pub type Six = S<Five>;

// Define a trait representing the operation of addition.
// Conceptually we're going to have a bunch of implementations
// of the trait that look like:
// impl Add<Three> for Four {
//    type Sum = Seven;
// }
// you get your result by checking the associated type
// Sum. The trick is to convince rust not to make use write
// all the additions tables
pub trait Add<A> {
    type Sum;
}

// Zero is easy, 0 + A = A
impl<A> Add<Zero> for A {
    type Sum = A;
}

// A+B is harder. We're going to use the fact that we can rewrite
// A+B as A+1+B' where B = B' + 1. Thus
// A+S(B) = S(A) + B. We can keep subtracting 1 from B and adding 1
// to A until we have A + 0 = A
// Effectively the below trait represents the rule:
// A + S(B) = S(A) + B
impl<A, B> Add<S<B>> for A
where
    S<A>: Add<B>,
{
    type Sum = <S<A> as Add<B>>::Sum;
}

pub trait Mul<A> {
    type Prod;
}

impl<A> Mul<Zero> for A {
    type Prod = Zero;
}

impl<A, B> Mul<S<B>> for A
where
    A: Mul<B>,
    <A as Mul<B>>::Prod: Add<A>,
{
    // A*B' = A * (B'-1+1)
    //      = A + A * (B'-1)
    // ->
    // A*S(B) = A * B + A
    type Prod = <<A as Mul<B>>::Prod as Add<A>>::Sum;
}

assert_type_eq_all!(<Two as Mul<Three>>::Prod, Six);

pub trait DecrNat {
    type Result;
}

impl DecrNat for Zero {
    type Result = Zero;
}

impl<A> DecrNat for S<A> {
    type Result = A;
}

assert_type_eq_all!(<Five as DecrNat>::Result, Four);

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

impl<Ml, Mv, Mr> MemoryOp<Incr> for Memory<Ml, Mv, Mr> {
    type Next = Memory<Ml, S<Mv>, Mr>;
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

impl<Body, Rest, Ml, Mr, In, Out> RunMachine<Cons<Loop<Body>, Rest>>
    for Machine<Memory<Ml, Zero, Mr>, In, Out>
where
    Self: RunMachine<Rest>,
{
    type Next = <Self as RunMachine<Rest>>::Next;
}

impl<Body, Rest, Ml, Mr, In, Out, N> RunMachine<Cons<Loop<Body>, Rest>>
    for Machine<Memory<Ml, S<N>, Mr>, In, Out>
where
    Self: RunMachine<Body>,
    <Self as RunMachine<Body>>::Next: RunMachine<Cons<Loop<Body>, Rest>>,
{
    type Next = <<Self as RunMachine<Body>>::Next as RunMachine<Cons<Loop<Body>, Rest>>>::Next;
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

impl Make<u8> for Zero {
    fn make() -> u8 {
        0
    }
}

impl<P> Make<u8> for S<P>
where
    P: Make<u8>,
{
    fn make() -> u8 {
        P::make() + 1
    }
}

impl Make<u32> for Zero {
    fn make() -> u32 {
        0
    }
}

impl<P> Make<u32> for S<P>
where
    P: Make<u32>,
{
    fn make() -> u32 {
        P::make() + 1
    }
}
