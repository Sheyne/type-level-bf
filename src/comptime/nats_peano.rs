pub use crate::peano::{DecrNat, One, Two, Five, Four, Six, Three, Add};

use crate::peano;

pub type Zero = peano::Zero;
pub struct True;
pub struct False;

pub trait IsNonZero {
    type Result;
}

impl IsNonZero for Zero {
    type Result = False;
}
impl<T> IsNonZero for peano::S<T> {
    type Result = True;
}

pub trait IncrNat {
    type Result;
}

impl<T> IncrNat for T {
    type Result = peano::S<T>;
}

pub trait NonZero {}

impl<T> NonZero for peano::S<T> {}

use super::Make;

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

impl<P> Make<u8> for crate::peano::S<P>
where
    P: Make<u8>,
{
    fn make() -> u8 {
        P::make() + 1
    }
}

impl<P> Make<u32> for crate::peano::S<P>
where
    P: Make<u32>,
{
    fn make() -> u32 {
        P::make() + 1
    }
}
