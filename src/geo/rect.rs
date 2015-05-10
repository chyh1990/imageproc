use std::fmt;
// use std::ops::{Add, Sub, Neg, Mul, Div};

use geo::Point;
use geo::types::GeoScalar;

#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
#[repr(C, packed)]
pub struct Rect<T: GeoScalar> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T
}

impl<T: GeoScalar> Rect<T> {
    pub fn new(x: T, y: T, w: T, h: T) -> Rect<T> {
        return Rect{
            x: x,
            y: y,
            width: w,
            height: h,
        }
    }

    pub fn area(&self) -> T {
        self.width * self.height
    }

    pub fn tl(&self) -> Point<T> {
        Point::new(self.x, self.y)
    }

    pub fn br(&self) -> Point<T> {
        Point::new(self.x + self.width, self.y + self.height)
    }

    pub fn contains(&self, p: &Point<T>) -> bool {
        if self.x <= p.x && p.x < self.x + self.width
            && self.y <= p.y && p.y < self.y + self.height {
                return true;
            } else {
                return false;
            }
    }
}

impl<T: GeoScalar> fmt::Display for Rect<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}, {}, {}]", self.x, self.y,
               self.width, self.height)
    }
}
