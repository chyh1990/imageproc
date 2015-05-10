use std::slice;
use num::NumCast;
use std::default::Default;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use traits::Primitive;

#[derive(Debug)]
pub enum ImageError {
    InvalidImage,
    OutOfMemoryError,
    UnknownImageFormat,
    UnknownError,
}

#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
#[repr(C, packed)]
pub struct Color<T: Primitive + Default> {
    b: T,
    g: T,
    r: T,
    a: T
}

impl<T: Primitive + Default> Color<T> {
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
pub trait Pixel: Copy + Clone {
    /// The underlying subpixel type.
    type Subpixel: Primitive + Default;

    fn zero() -> Self;

    fn has_alpha() -> bool;

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
pub struct $ident<T: Primitive + Default> { pub data: [T; $channels] }

#[allow(non_snake_case, dead_code)]
#[inline]
pub fn $ident<T: Primitive + Default>(data: [T; $channels]) -> $ident<T> {
        $ident {
                    data: data
        }
}

impl<T: Primitive + Default> Pixel for $ident<T> {
    type Subpixel = T;

    #[inline]
    fn zero() -> $ident<T> {
        $ident {
            data: [Default::default(); $channels]
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
    fn has_alpha() -> bool {
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

impl<T: Primitive + Default> Index<usize> for $ident<T> {
    type Output = T;

    #[inline]
    fn index(&self, _index: usize) -> &T {
        &self.data[_index]
    }
}

impl<T: Primitive + Default> IndexMut<usize> for $ident<T> {
    #[inline]
    fn index_mut(&mut self, _index: usize) -> &mut T {
        &mut self.data[_index]
    }
}

)* // END
    }
}

define_colors! {
    Bgr, 3, false, "BGR", #[doc = "RGB colors"];
    Gray, 1, false, "Y", #[doc = "Grayscale colors"];
    Bgra, 4, true, "BGRA", #[doc = "BGR colors + alpha channel"];
    Rgba, 4, true, "RGBA", #[doc = "RGB colors + alpha channel"];
}

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
    fn has_alpha(&self) -> bool {
        T::has_alpha()
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

    pub fn row_mut(&mut self, r: u32) -> &mut [T] {
        let start = r * self.stride;
        &mut self.data[start as usize .. (start + self.stride) as usize]
    }

    pub fn fill(&mut self, v: &T) {
        for p in self.data.iter_mut() {
            *p = *v;
        }
    }

    pub fn zero(&mut self) {
        self.fill(&T::zero())
    }

    pub fn pixel_at(&self, x: u32, y: u32) -> &T {
        let off = self.stride * y + x;
        &self.data[off as usize]
    }

    fn pixel_mut_at(&mut self, x: u32, y: u32) -> &mut T {
        let off = self.stride * y + x;
        &mut self.data[off as usize]
    }
}

impl<T: Pixel> Drop for Image<T> {
        fn drop(&mut self) {
            self.data.clear();
        }
}

pub type ImageBGRA = Image<Bgra<u8>>;
pub type ImageGray = Image<Gray<u8>>;
pub type ImageBGR = Image<Bgr<u8>>;
pub type ImageGrayf = Image<Gray<f32>>;

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
        let img = ImageBGRA::new(100, 200);
        assert_eq!(img.channels(), 4);
        assert_eq!(img.bits_per_pixel(), 4 * 8);
        assert_eq!(img.pixels().len(), 100 * 200);
        assert_eq!(img.raw().len(), 100 * 200 * 4);
        assert_eq!(img.pitch(), 100 * 4);
    }

}

