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

pub struct True;
pub struct False;

pub trait IsNonZero {
    type Result;
}

impl IsNonZero for Zero {
    type Result = False;
}
impl<T> IsNonZero for S<T> {
    type Result = True;
}

pub trait IncrNat {
    type Result;
}

impl<T> IncrNat for T {
    type Result = S<T>;
}

pub trait NonZero {}

impl<T> NonZero for S<T> {}

use crate::comptime::Make;

impl Make<u8> for Zero {
    fn make() -> u8 {
        0
    }
}

impl Make<u32> for Zero {
    fn make() -> u32 {
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

impl<P> Make<u32> for S<P>
where
    P: Make<u32>,
{
    fn make() -> u32 {
        P::make() + 1
    }
}
