mod types;
pub mod point;
pub mod rect;

pub use self::point::Point;
pub use self::rect::Rect;

pub type Pointi = Point<i32>;
pub type Pointf = Point<f32>;
pub type Recti = Rect<i32>;
pub type Rectf = Rect<f32>;
