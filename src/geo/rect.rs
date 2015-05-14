use std::fmt;

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

    pub fn intersect(&self, r: &Rect<T>) -> Rect<T> {
        let x0 = self.x.max(r.x);
        let y0 = self.y.max(r.y);
        let x1 = self.br().x.min(r.br().x);
        let y1 = self.br().y.min(r.br().y);
        if x0 > x1 || y0 > y1 {
            Rect::new(x0, y0, T::zero(), T::zero())
        } else {
            Rect::new(x0, y0, x1 - x0, y1 - y0)
        }
    }
}

impl<T: GeoScalar> fmt::Display for Rect<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}, {}, {}]", self.x, self.y,
               self.width, self.height)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ops() {
        let r1 = Rect::new(1f32, 1f32, 5f32, 6f32);
        let r2 = Rect::new(2f32, 3f32, 7f32, 7f32);
        let out = r1.intersect(&r2);
        assert_eq!(out, Rect::new(2f32, 3f32, 4f32, 4f32));
    }
}

