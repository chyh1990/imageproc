use std::fmt;
use std::ops::{Add, Sub, Neg, Mul, Div};

use geo::types::GeoScalar;

#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
#[repr(C, packed)]
pub struct Point<T: GeoScalar> {
    pub x: T,
    pub y: T
}

impl<T: GeoScalar> Point<T> {
    pub fn new(x: T, y: T) -> Point<T> {
        return Point{
            x: x,
            y: y
        }
    }
}

impl<T: GeoScalar> fmt::Display for Point<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T: GeoScalar> Add for Point<T> {
    type Output = Point<T>;

    fn add(self, other: Point<T>) -> Point<T> {
        Point {x: self.x + other.x, y: self.y + other.y}
    }
}

impl<T: GeoScalar> Sub for Point<T> {
    type Output = Point<T>;

    fn sub(self, other: Point<T>) -> Point<T> {
        Point {x: self.x - other.x, y: self.y - other.y}
    }
}

impl<T: GeoScalar> Neg for Point<T> {
    type Output = Point<T>;

    fn neg(self) -> Point<T> {
        Point {x: T::zero() - self.x, y: T::zero() - self.y }
    }
}

impl<T: GeoScalar> Mul<T> for Point<T> {
    type Output = Point<T>;

    fn mul(self, _rhs: T) -> Point<T> {
        Point {x: self.x * _rhs, y: self.y * _rhs }
    }
}


impl<T: GeoScalar> Div<T> for Point<T> {
    type Output = Point<T>;

    fn div(self, _rhs: T) -> Point<T> {
        Point {x: self.x / _rhs, y: self.y / _rhs }
    }
}



#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_ops() {
        let p1 = Point::new(1,4);
        let p2 = Point::new(2,3);
        let out = p1 + p2;
        assert_eq!(out.x, 3);
        assert_eq!(out.y, 7);
    }
}

