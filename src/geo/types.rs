use std::fmt::Display;
use std::ops::{Neg, Mul, Div};
use std::cmp::{min, max};
use num::{ Bounded, Num, NumCast };

pub trait NumMinMax {
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
}

pub trait GeoScalar: Copy + NumCast + Num 
    + PartialOrd<Self> + Clone + Bounded
    + Display + Neg + Mul + Div + NumMinMax {
}

macro_rules! define_scalar_int(
    ($t:ty) => (
impl GeoScalar for $t {}
impl NumMinMax for $t {
    fn min(self, other: Self) -> Self {
        min(self, other)
    }
    fn max(self, other: Self) -> Self {
        max(self, other)
    }
}
);
);

macro_rules! define_scalar_float(
    ($t:ty) => (
impl GeoScalar for $t {}
impl NumMinMax for $t {
    fn min(self, other: Self) -> Self {
        self.min(other)
    }
    fn max(self, other: Self) -> Self {
        self.max(other)
    }
}
);
);

define_scalar_int!(isize);
define_scalar_int!(i8);
define_scalar_int!(i16);
define_scalar_int!(i32);
define_scalar_int!(i64);
define_scalar_float!(f32);
define_scalar_float!(f64);

