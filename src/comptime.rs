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

// The whole U8 trait and implementations is so we can get a runtime
// representation of our compile time numbers. Recall 2 = S<S<0>>,
// but that S<S<Zero>> is a zero sized struct, it doesn't exist at
// runtime. Here we say, for any type of the form S<S<S<..<0>>>>
// you can call the function u8()->u8 to get an eight bit number
// representing the value that the type encodes. The impls of the methods
// will end up looking like:
// impl U8 for S<S<S<Zero>>> { fn u8() -> u8 {return 1 + 1 + 1 + 0; } }
// Of course rust lets us write this recursively.
pub trait U8 {
    fn u8() -> u8;
}

impl U8 for Zero {
    fn u8() -> u8 {
        0
    }
}

impl<P> U8 for S<P>
where
    P: U8,
{
    fn u8() -> u8 {
        P::u8() + 1
    }
}

impl<T> ToValue for T
where
    T: U8,
{
    type T = u8;
    fn to_value() -> u8 {
        T::u8()
    }
}

pub trait ToValue {
    type T;
    fn to_value() -> Self::T;
}

pub struct Nil;
pub struct Cons<H, R>(PhantomData<(H, R)>);

pub struct Memory<L, V, R>(PhantomData<(L, V, R)>);
pub struct Machine<Mem, Inp, Out>(PhantomData<(Mem, Inp, Out)>);

pub struct Incr<Next>(PhantomData<Next>);
pub struct Decr<Next>(PhantomData<Next>);
pub struct Right<Next>(PhantomData<Next>);
pub struct Left<Next>(PhantomData<Next>);
pub struct Loop<Body, Next>(PhantomData<(Body, Next)>);
pub struct Write<Next>(PhantomData<Next>);

pub trait PureStep {}
impl<T> PureStep for Incr<T> {}
impl<T> PureStep for Decr<T> {}
impl<T> PureStep for Right<T> {}
impl<T> PureStep for Left<T> {}
impl<T, U> PureStep for Loop<T, U> {}

pub trait Step<Prog> {
    type Next;
}

impl<T> Step<Nil> for T {
    type Next = T;
}

impl<Ml, Mv, Mr, Pn> Step<Incr<Pn>> for Memory<Ml, Mv, Mr>
where
    Memory<Ml, S<Mv>, Mr>: Step<Pn>,
{
    type Next = <Memory<Ml, S<Mv>, Mr> as Step<Pn>>::Next;
}

impl<Ml, Mv, Mr, Pn> Step<Decr<Pn>> for Memory<Ml, Mv, Mr>
where
    Mv: DecrNat,
    Memory<Ml, <Mv as DecrNat>::Result, Mr>: Step<Pn>,
{
    type Next = <Memory<Ml, <Mv as DecrNat>::Result, Mr> as Step<Pn>>::Next;
}

impl<Ml, Mv, Mr1, Mrr, Pn> Step<Right<Pn>> for Memory<Ml, Mv, Cons<Mr1, Mrr>>
where
    Memory<Cons<Mv, Ml>, Mr1, Mrr>: Step<Pn>,
{
    type Next = <Memory<Cons<Mv, Ml>, Mr1, Mrr> as Step<Pn>>::Next;
}

impl<Ml, Mv, Pn> Step<Right<Pn>> for Memory<Ml, Mv, Nil>
where
    Memory<Cons<Mv, Ml>, Zero, Nil>: Step<Pn>,
{
    type Next = <Memory<Cons<Mv, Ml>, Zero, Nil> as Step<Pn>>::Next;
}

impl<Mr, Mv, Ml1, Mlr, Pn> Step<Left<Pn>> for Memory<Cons<Ml1, Mlr>, Mv, Mr>
where
    Memory<Mlr, Ml1, Cons<Mv, Mr>>: Step<Pn>,
{
    type Next = <Memory<Mlr, Ml1, Cons<Mv, Mr>> as Step<Pn>>::Next;
}

impl<Mr, Mv, Pn> Step<Left<Pn>> for Memory<Nil, Mv, Mr>
where
    Memory<Nil, Zero, Cons<Mv, Mr>>: Step<Pn>,
{
    type Next = <Memory<Nil, Zero, Cons<Mv, Mr>> as Step<Pn>>::Next;
}

impl<Lbody, Lnext, Ml, Mr> Step<Loop<Lbody, Lnext>> for Memory<Ml, Zero, Mr>
where
    Self: Step<Lnext>,
{
    type Next = <Self as Step<Lnext>>::Next;
}

impl<Lbody, Lnext, Ml, Mr, N> Step<Loop<Lbody, Lnext>> for Memory<Ml, S<N>, Mr>
where
    Memory<Ml, S<N>, Mr>: Step<Lbody>,
    <Memory<Ml, S<N>, Mr> as Step<Lbody>>::Next: Step<Loop<Lbody, Lnext>>,
{
    type Next = <<Memory<Ml, S<N>, Mr> as Step<Lbody>>::Next as Step<Loop<Lbody, Lnext>>>::Next;
}

impl<Prog, Mem, In, Out> Step<Prog> for Machine<Mem, In, Out>
where
    Prog: PureStep,
    Mem: Step<Prog>,
{
    type Next = <Mem as Step<Prog>>::Next;
}

impl<Pn, Ml, Mv, Mr, In, Out> Step<Write<Pn>> for Machine<Memory<Ml, Mv, Mr>, In, Out>
where
    Machine<Memory<Ml, Mv, Mr>, In, Cons<Mv, Out>>: Step<Pn>,
{
    type Next = <Machine<Memory<Ml, Mv, Mr>, In, Cons<Mv, Out>> as Step<Pn>>::Next;
}

fn _test_incr_decr() {
    type StateInital = Memory<Nil, Zero, Nil>;
    type StateExpected1 = Memory<Nil, One, Nil>;
    type StateExpected2 = Memory<Nil, Two, Nil>;

    assert_type_eq_all!(<StateInital as Step<Incr<Nil>>>::Next, StateExpected1);
    assert_type_eq_all!(<StateInital as Step<Incr<Incr<Nil>>>>::Next, StateExpected2);
    assert_type_eq_all!(
        <StateInital as Step<Incr<Incr<Decr<Nil>>>>>::Next,
        StateExpected1
    );
}

fn _test_move_left_right() {
    type StateInital = Memory<Nil, One, Nil>;
    type StateExpected1 = Memory<Cons<One, Nil>, Zero, Nil>;
    type StateExpected2 = Memory<Nil, One, Cons<Zero, Nil>>;

    assert_type_eq_all!(<StateInital as Step<Right<Nil>>>::Next, StateExpected1);
    assert_type_eq_all!(
        <StateInital as Step<Right<Left<Nil>>>>::Next,
        StateExpected2
    );
}

fn _test_loop() {
    type StateInital = Memory<Nil, Two, Nil>;
    type StateExpected = Memory<Nil, One, Nil>;
    type Program = Loop<Decr<Nil>, Incr<Nil>>;

    assert_type_eq_all!(<StateInital as Step<Program>>::Next, StateExpected);
}


impl<H, R> ToValue for Cons<H, R>
where
    H: ToValue,
    R: ToValue,
{
    type T = (<H as ToValue>::T, <R as ToValue>::T);

    fn to_value() -> Self::T {
        return (<H as ToValue>::to_value(), <R as ToValue>::to_value());
    }
}

impl ToValue for Nil {
    type T = String;

    fn to_value() -> Self::T {
        "Nil".to_owned()
    }
}

impl<L, V, R> ToValue for Memory<L, V, R>
where
    L: ToValue,
    V: ToValue,
    R: ToValue,
{
    type T = (<L as ToValue>::T, <V as ToValue>::T, <R as ToValue>::T);

    fn to_value() -> Self::T {
        return (
            <L as ToValue>::to_value(),
            <V as ToValue>::to_value(),
            <R as ToValue>::to_value(),
        );
    }
}
