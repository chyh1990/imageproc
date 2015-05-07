use std::iter::repeat;
use std::slice;
use std::default::Default;
use std::marker::PhantomData;

use traits::Primitive;

#[derive(Debug)]
pub enum ImageError {
    InvalidImage,
    OutOfMemoryError,
    UnknownImageFormat
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
pub fn $ident<T: Primitive + Default>(data: [T; $channels]) -> $ident<T> {
        $ident {
                    data: data
        }
}

impl<T: Primitive + Default> Pixel for $ident<T> {
    type Subpixel = T;

    #[inline(always)]
    fn zero() -> $ident<T> {
        $ident {
            data: [Default::default(); $channels]
        }
    }

    #[inline(always)]
    fn channels() -> u8 {
        $channels
    }

    #[inline(always)]
    fn bits_per_pixel() -> u8 {
        8 * $channels
    }

    #[inline(always)]
    fn has_alpha() -> bool {
        $alpha
    }

    #[inline(always)]
    fn raw(&self) -> &[T] {
        &self.data
    }

    #[inline(always)]
    fn raw_mut(&mut self) -> &mut [T] {
        &mut self.data
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


#[derive(Debug)]
pub struct Image<T: Pixel> {
    w: u32,
    h: u32,
    stride: u32, //stride in sizeof(T)
    data: Vec<T>
}

impl<T: Pixel> Image<T> {
    fn new(width: u32, height: u32) -> Image<T> {
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

    fn from_raw(data: &[u8], width: u32, height: u32, stride: u32)
        -> Result<Image<T>, ImageError> {
            if stride < width {
                return Err(ImageError::OutOfMemoryError);
            }
            match (height * stride * (T::bits_per_pixel() as u32 / 8) ) as usize <= data.len() {
                true => {
                let data: Vec<T> = Vec::with_capacity((height * width) as usize);
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

    #[inline(always)]
    fn pixels(&self) -> &Vec<T> {
        &self.data
    }

    #[inline(always)]
    fn pixels_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    fn raw(&self) -> &[T::Subpixel] {
        let raw_len = self.bytes_per_row() * self.h as usize;
        unsafe { slice::from_raw_parts(self.data.as_ptr() as *const T::Subpixel, raw_len) }
    }

    fn raw_mut(&mut self) -> &mut [T::Subpixel] {
        let raw_len = self.bytes_per_row() * self.h as usize;
        unsafe { slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T::Subpixel, raw_len) }
    }

    #[inline(always)]
    fn channels(&self) -> u8 {
        T::channels()
    }

    #[inline(always)]
    fn bits_per_pixel(&self) -> u8 {
        T::bits_per_pixel()
    }

    #[inline(always)]
    fn bytes_per_row(&self) -> usize {
        (self.stride as usize) * (T::bits_per_pixel() / 8) as usize
    }

    fn fill(&mut self, v: &T) {
        for p in self.data.iter_mut() {
            *p = *v;
        }
    }

    fn zero(&mut self) {
        self.fill(&T::zero())
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
    }
}

