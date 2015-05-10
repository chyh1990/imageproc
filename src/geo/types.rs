use std::fmt::Display;
use std::ops::{Neg, Mul, Div};
use num::{ Bounded, Num, NumCast };

pub trait GeoScalar: Copy + NumCast + Num 
    + PartialOrd<Self> + Clone + Bounded
    + Display + Neg + Mul + Div {
}

impl GeoScalar for isize {
}
impl GeoScalar for i8 {
}
impl GeoScalar for i16 {
}
impl GeoScalar for i32 {
}
impl GeoScalar for i64 {
}
impl GeoScalar for f32 {
}
impl GeoScalar for f64 {
}
