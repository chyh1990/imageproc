//! This module provides usefull traits that where deprecated in rust

// Note copied from the stdlib under MIT license

use std::fmt::Display;
use num::{ Bounded, Num, NumCast };


/// Primitive trait from old stdlib, added max_value
pub trait Primitive: Copy + NumCast + Num
    + PartialOrd<Self> + Clone
    + Bounded + Display {
}

impl Primitive for usize {
}
impl Primitive for u8 {
}
impl Primitive for u16 {
}
impl Primitive for u32 {
}
impl Primitive for u64 {
}
impl Primitive for isize {
}
impl Primitive for i8 {
}
impl Primitive for i16 {
}
impl Primitive for i32 {
}
impl Primitive for i64 {
}
impl Primitive for f32 {
}
impl Primitive for f64 {
}
