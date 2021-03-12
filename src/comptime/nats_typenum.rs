pub use typenum::{False, True};

pub trait IsNonZero {
    type Result;
}

impl<T> IsNonZero for T
where
    T: typenum::IsNotEqual<Zero>,
{
    type Result = <T as typenum::IsNotEqual<Zero>>::Output;
}

pub type Zero = typenum::U0;
pub type One = typenum::U1;
pub type Two = typenum::U2;
pub type Three = typenum::U3;
pub type Five = typenum::U5;

pub trait Add<T> {
    type Sum;
}

impl<T, U> Add<T> for U
where
    U: core::ops::Add<T>,
{
    type Sum = <U as core::ops::Add<T>>::Output;
}

pub trait IncrNat {
    type Result;
}

impl<T> IncrNat for T
where
    T: std::ops::Add<One>,
{
    type Result = <T as std::ops::Add<One>>::Output;
}
pub trait DecrNat {
    type Result;
}

impl<T> DecrNat for T
where
    T: std::ops::Sub<One>,
{
    type Result = <T as std::ops::Sub<One>>::Output;
}

use super::Make;

impl<T> Make<i8> for T
where
    T: typenum::Integer,
{
    fn make() -> i8 {
        T::to_i8()
    }
}

impl<T> Make<i32> for T
where
    T: typenum::Integer,
{
    fn make() -> i32 {
        T::to_i32()
    }
}

impl<T> Make<u8> for T
where
    T: typenum::Unsigned,
{
    fn make() -> u8 {
        T::to_u8()
    }
}

impl<T> Make<u32> for T
where
    T: typenum::Unsigned,
{
    fn make() -> u32 {
        T::to_u32()
    }
}
