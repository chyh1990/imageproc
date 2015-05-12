use traits::Primitive;

#[inline(always)]
pub fn clipped_round(x: f32, min: i32, max: i32) -> i32 {
    let t = x.round() as i32;
    if t < min {
        return min;
    }
    if t > max {
        return max;
    }
    t
}

#[inline(always)]
pub fn clip<T: Primitive>(x: T, min: T, max: T) -> T {
    if x < min {
        return min;
    }
    if x > max {
        return max;
    }
    x
}

