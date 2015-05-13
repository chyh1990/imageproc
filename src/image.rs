use std::slice;
use num::NumCast;
use num::traits::{Saturating, Bounded};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::ops::{Add, Sub, Mul};

use traits::Primitive;
use geo::Rect;

#[derive(Debug)]
pub enum ImageError {
    InvalidImage,
    OutOfRegion,
    OutOfMemoryError,
    UnknownImageFormat,
    UnknownError,
}

#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
#[repr(C, packed)]
pub struct Color<T: Primitive> {
    b: T,
    g: T,
    r: T,
    a: T
}

impl<T: Primitive> Color<T> {
    fn new(b: T, g: T, r: T, a: T) -> Color<T> {
        Color {
            b: b,
            g: g,
            r: r,
            a: a,
        }
    }

    fn from_gray(gray: T) -> Color<T> {
        Color {
            b: gray,
            g: gray,
            r: gray,
            a: T::max_value()
        }
    }

    fn is_gray(&self) -> bool {
        (self.b == self.g) && (self.r == self.g)
    }
}

/// A pixel object is usually not used standalone but as a view into an image buffer.
pub trait Pixel: Copy + Clone + Index<usize> {
    /// The underlying subpixel type.
    type Subpixel: Primitive;

    fn zero() -> Self;

    fn alpha() -> isize;

    /// Returns the number of channels of this pixel type.
    fn channels() -> u8;

    /// Returns the bits per pixel
    fn bits_per_pixel() -> u8;

    /// Returns the components as a slice.
    fn raw(&self) -> &[Self::Subpixel];

    /// Returns the components as a mutable slice
    fn raw_mut(&mut self) -> &mut [Self::Subpixel];

    fn blend(self, other: Self, alpha: f32) -> Self;
    fn blend4(self, b: Self, c: Self, d: Self, u: f32, v: f32) -> Self;
}

// originally from https://github.com/PistonDevelopers/image
macro_rules! define_colors {
    {$(
        $ident:ident,
        $channels: expr,
        $alpha: expr,
        $interpretation: expr,
        #[$doc:meta];
    )*} => {

$( // START Structure definitions

#[$doc]
#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
#[repr(C, packed)]
#[allow(missing_docs)]
pub struct $ident<T: Primitive> { pub data: [T; $channels] }

#[allow(non_snake_case, dead_code)]
#[inline]
pub fn $ident<T: Primitive>(data: [T; $channels]) -> $ident<T> {
        $ident {
                    data: data
        }
}

impl<T: Primitive> Pixel for $ident<T> {
    type Subpixel = T;

    #[inline]
    fn zero() -> $ident<T> {
        $ident {
            data: [T::zero(); $channels]
        }
    }

    #[inline]
    fn channels() -> u8 {
        $channels
    }

    #[inline]
    fn bits_per_pixel() -> u8 {
        8 * $channels
    }

    #[inline]
    fn alpha() -> isize {
        $alpha
    }


    #[inline]
    fn raw(&self) -> &[T] {
        &self.data
    }

    #[inline]
    fn raw_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    #[inline]
    fn blend(self, other: Self, alpha: f32) -> Self {
        // OPTIMIZE
        let mut t = self.data;
        for i in 0..$channels {
            t[i] = NumCast::from(t[i].to_f32().unwrap() * alpha
                + other.data[i].to_f32().unwrap() * (1f32 - alpha)).unwrap();
        }
        $ident(t)
    }

    #[inline]
    fn blend4(self, b: Self, c: Self, d: Self, u: f32, v: f32) -> Self {
        let a0 = u * v;
        let a1 = (1.0 - u) * v;
        let a2 = u * (1.0 - v);
        let a3 = (1.0 - u) * (1.0 - v);

        let mut a = self.data;
        for i in 0..$channels {
            a[i] = NumCast::from(
                a[i].to_f32().unwrap() * a0
                + b[i].to_f32().unwrap() * a1
                + c[i].to_f32().unwrap() * a2
                + d[i].to_f32().unwrap() * a3
                ).unwrap();
        }
        $ident(a)
    }
}

impl<T: Primitive> Index<usize> for $ident<T> {
    type Output = T;

    #[inline]
    fn index(&self, _index: usize) -> &T {
        &self.data[_index]
    }
}

impl<T: Primitive> IndexMut<usize> for $ident<T> {
    #[inline]
    fn index_mut(&mut self, _index: usize) -> &mut T {
        &mut self.data[_index]
    }
}

impl<T: Primitive> Add for $ident<T> {
    type Output = $ident<T>;

    #[inline]
    fn add(self, other: $ident<T>) -> $ident<T> {
        let mut m = [T::zero(); $channels];
        for i in 0..$channels { m[i] = self.data[i] + other.data[i]; }
        $ident(m)
    }
}

impl<T: Primitive> Sub for $ident<T> {
    type Output = $ident<T>;

    #[inline]
    fn sub(self, other: $ident<T>) -> $ident<T> {
        let mut m = [T::zero(); $channels];
        for i in 0..$channels { m[i] = self.data[i] - other.data[i]; }
        $ident(m)
    }
}

impl<T: Primitive> Mul for $ident<T> {
    type Output = $ident<T>;

    #[inline]
    fn mul(self, other: $ident<T>) -> $ident<T> {
        let mut m = [T::zero(); $channels];
        for i in 0..$channels { m[i] = self.data[i] * other.data[i]; }
        $ident(m)
    }
}

// Pixel * scalar
impl<T: Primitive, U: Primitive> Add<U> for $ident<T> {
    type Output = $ident<U>;

    #[inline]
    fn add(self, other: U) -> $ident<U> {
        let mut m = [U::zero(); $channels];
        for i in 0..$channels {
            m[i] = U::from(self.data[i]).unwrap() + other;
        }
        $ident(m)
    }
}

impl<T: Primitive, U: Primitive> Sub<U> for $ident<T> {
    type Output = $ident<U>;

    #[inline]
    fn sub(self, other: U) -> $ident<U> {
        let mut m = [U::zero(); $channels];
        for i in 0..$channels {
            m[i] = U::from(self.data[i]).unwrap() - other;
        }
        $ident(m)
    }
}

impl<T: Primitive, U: Primitive> Mul<U> for $ident<T> {
    type Output = $ident<U>;

    #[inline]
    fn mul(self, other: U) -> $ident<U> {
        let mut m = [U::zero(); $channels];
        for i in 0..$channels {
            m[i] = U::from(self.data[i]).unwrap() * other;
        }
        $ident(m)
    }
}

)* // END
    }
}

define_colors! {
    Bgr, 3, -1, "BGR", #[doc = "RGB colors"];
    Gray, 1, -1, "Y", #[doc = "Grayscale colors"];
    Bgra, 4, 3, "BGRA", #[doc = "BGR colors + alpha channel"];
    Rgba, 4, 3, "RGBA", #[doc = "RGB colors + alpha channel"];
}

macro_rules! define_saturating(
    ($t:ident) => (
impl<T: Primitive + Saturating> Saturating for $t<T> {
    #[inline]
    fn saturating_add(self, other: $t<T>) -> $t<T> {
        let mut m = self.data;
        for i in 0..$t::<T>::channels() as usize {
            m[i] = m[i].saturating_add(other.data[i]); }
        $t(m)
    }
    #[inline]
    fn saturating_sub(self, other: $t<T>) -> $t<T> {
        let mut m = self.data;
        for i in 0..$t::<T>::channels() as usize {
            m[i] = m[i].saturating_sub(other.data[i]); }
        $t(m)
    }
}
);
);

define_saturating!(Gray);
define_saturating!(Bgr);
define_saturating!(Bgra);

pub trait GenericImage {
    type Pixel: Pixel;
}

#[derive(Debug)]
pub struct Image<T: Pixel> {
    w: u32,
    h: u32,
    stride: u32, //stride in sizeof(T)
    data: Vec<T>
}

impl<T: Pixel> GenericImage for Image<T> {
    type Pixel = T;
}

impl<T: Pixel> Image<T> {
    pub fn new(width: u32, height: u32) -> Image<T> {
        // fast allocation without initization
        let len = (width as usize) * (height as usize);
        let mut data: Vec<T> = Vec::with_capacity(len);
        unsafe { data.set_len(len); }
        Image {
            w: width,
            h: height,
            stride: width,
            data: data
        }
    }

    pub fn from_raw(data: &[u8], width: u32, height: u32, stride: u32)
        -> Result<Image<T>, ImageError> {
            if stride < width {
                return Err(ImageError::OutOfMemoryError);
            }
            match (height * stride * (T::bits_per_pixel() as u32 / 8) ) as usize <= data.len() {
                true => {
                let data: Vec<T> = Vec::with_capacity((height * width) as usize);
                // TODO(chenyh): copy data
                Ok(Image {
                    w: width,
                    h: height,
                    stride: width,
                    data: data
                })
                },
                false => Err(ImageError::OutOfMemoryError)
            }
    }

    #[inline]
    pub fn width(&self) -> u32 { self.w }

    #[inline]
    pub fn height(&self) -> u32 { self.h }

    #[inline]
    pub fn size(&self) -> (u32, u32) { (self.w, self.h) }

    #[inline]
    pub fn stride(&self) -> u32 {self.stride }

    #[inline]
    pub fn pitch(&self) -> u32 { self.stride * (self.bits_per_pixel() / 8) as u32 }

    #[inline]
    pub fn pixels(&self) -> &[T] {
        &self.data
    }

    #[inline]
    pub fn pixels_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    pub fn raw(&self) -> &[T::Subpixel] {
        let raw_len = self.bytes_per_row() * self.h as usize;
        unsafe { slice::from_raw_parts(self.data.as_ptr() as *const T::Subpixel, raw_len) }
    }

    pub fn raw_mut(&mut self) -> &mut [T::Subpixel] {
        let raw_len = self.bytes_per_row() * self.h as usize;
        unsafe { slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T::Subpixel, raw_len) }
    }

    #[inline]
    pub fn channels(&self) -> u8 {
        T::channels()
    }

    #[inline]
    pub fn bits_per_pixel(&self) -> u8 {
        T::bits_per_pixel()
    }

    #[inline]
    fn alpha(&self) -> isize {
        T::alpha()
    }

    #[inline]
    fn has_alpha() -> bool {
        T::alpha() >= 0
    }

    #[inline]
    pub fn bytes_per_row(&self) -> usize {
        (self.stride as usize) * (T::bits_per_pixel() / 8) as usize
    }

    #[inline]
    pub fn row(&self, r: u32) -> &[T] {
        let start = r * self.stride;
        &self.data[start as usize .. (start + self.stride) as usize]
    }

    #[inline]
    pub fn row_mut(&mut self, r: u32) -> &mut [T] {
        let start = r * self.stride;
        &mut self.data[start as usize .. (start + self.stride) as usize]
    }

    pub fn fill(&mut self, v: &T) {
        for p in self.data.iter_mut() {
            *p = *v;
        }
    }

    pub fn fill_channel(&mut self, ch_idx: usize, v: T::Subpixel) {
        assert!(ch_idx < T::channels() as usize);
        for p in self.data.iter_mut() {
            p.raw_mut()[ch_idx] = v;
        }
    }

    pub fn set_alpha(&mut self) {
        if Image::<T>::has_alpha() {
            self.fill_channel(T::alpha() as usize,
                T::Subpixel::max_value());
        }
    }

    pub fn zero(&mut self) {
        self.fill(&T::zero())
    }

    pub fn iter(&self) -> ImageIterator<T> {
        ImageIterator {
            image: &self,
            row: self.row(0),
            y: 0,
            x: 0
        }
    }

    pub fn iter_mut(&mut self) -> ImageMutIterator<T> {
        ImageMutIterator {
            image: self,
            y: 0,
            x: 0
        }
    }
    //pub fn crop(&self, rect: &Rect) -> Result<Image<T>, ImageError> {
    //}
}

impl<T: Pixel> Drop for Image<T> {
        fn drop(&mut self) {
            self.data.clear();
        }
}

impl<T: Pixel> Clone for Image<T> {
    fn clone(&self) -> Image<T> {
        Image {
            w: self.w,
            h: self.h,
            stride: self.stride,
            data: self.data.clone(),
        }
    }
}

impl<T: Pixel> Index<(u32, u32)> for Image<T> {
    type Output = T;

    #[inline]
    fn index(&self, _index: (u32, u32)) -> &T {
        let (x, y) = _index;
        let off = self.stride * y + x;
        &self.data[off as usize]
    }
}

impl<T: Pixel> IndexMut<(u32, u32)> for Image<T> {
    #[inline]
    fn index_mut(&mut self, _index: (u32, u32)) -> &mut T {
        let (x, y) = _index;
        let off = self.stride * y + x;
        &mut self.data[off as usize]
    }
}


pub type ImageGray = Image<Gray<u8>>;
pub type ImageBgr = Image<Bgr<u8>>;
pub type ImageBgra = Image<Bgra<u8>>;

pub type ImageGrayf = Image<Gray<f32>>;
pub type ImageBgrf = Image<Bgr<f32>>;
pub type ImageBgraf = Image<Bgra<f32>>;

pub struct ImageIterator<'a, P>
    where P: Pixel + 'a,
          <P as Index<usize>>::Output: 'a,
          P::Subpixel: 'a
{
    image: &'a Image<P>,
    row: &'a [P],
    y: u32,
    x: u32
}

impl<'a, P> Iterator for ImageIterator<'a, P> 
    where P: Pixel + 'a,
          <P as Index<usize>>::Output: 'a,
          P::Subpixel: 'a,
{
    type Item = (u32, u32, &'a P);
    #[inline]
    fn next(&mut self) -> Option<(u32, u32, &'a P)> {
        if self.x >= self.image.width() {
            self.y += 1;
            if self.y >= self.image.height() {
                return None;
            } else {
                self.row = self.image.row(self.y);
            }
            self.x  = 0;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;
        Some((x, y, &self.row[x as usize]))
    }
}

pub struct ImageMutIterator<'a, P>
    where P: Pixel + 'a,
          <P as Index<usize>>::Output: 'a,
          P::Subpixel: 'a
{
    image: &'a mut Image<P>,
    y: u32,
    x: u32
}

impl<'a, P> Iterator for ImageMutIterator<'a, P> 
    where P: Pixel + 'a,
          <P as Index<usize>>::Output: 'a,
          P::Subpixel: 'a,
{
    type Item = (u32, u32, &'a mut P);
    #[inline]
    fn next(&mut self) -> Option<(u32, u32, &'a mut P)> {
        if self.x >= self.image.width() {
            self.y += 1;
            self.x  = 0;
        }
        if self.y >= self.image.height() {
            return None;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;
        // TODO: implement this without `unsafe'
        unsafe {
            let t: *mut P = &mut self.image.row_mut(y)[x as usize];
            Some((x, y, &mut *t))
        }
    }
}



#[cfg(test)]
mod test {
    use std::mem;
    use super::*;

    #[test]
    fn test_pixel_size() {
        assert_eq!(mem::size_of::<Bgra<u8>>(), 4);
        assert_eq!(mem::size_of::<Bgr<u8>>(), 3);
        assert_eq!(mem::size_of::<Gray<u8>>(), 1);
    }

    #[test]
    fn test_alloc() {
        let img = ImageBgra::new(100, 200);
        assert_eq!(img.channels(), 4);
        assert_eq!(img.bits_per_pixel(), 4 * 8);
        assert_eq!(img.pixels().len(), 100 * 200);
        assert_eq!(img.raw().len(), 100 * 200 * 4);
        assert_eq!(img.pitch(), 100 * 4);
    }

    #[test]
    fn test_iter() {
        let mut img = ImageBgra::new(10, 5);
        for (x, y, p) in img.iter_mut() {
            *p = Bgra::<u8>([128, 128, 0, 0]);
        }

        for (x, y, p) in img.iter() {
            assert_eq!(*p, Bgra::<u8>([128, 128, 0, 0]));
        }
    }

    #[test]
    fn test_traits() {
        let mut img = ImageBgra::new(10, 5);
        for (x, y, p) in img.iter_mut() {
            *p = *p + Bgra::<u8>([128, 128, 0, 0]);
        }
    }

}

