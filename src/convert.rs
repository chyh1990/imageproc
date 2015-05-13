use std::ops::{Index, IndexMut};
use num::traits::ToPrimitive;
use image::*;
use traits::Primitive;

pub trait ColorMapper {
    type SrcType: Pixel;
    type DstType: Pixel;

    fn to(src: &Self::SrcType) -> Self::DstType;
}

pub struct MapBgraGray;
impl ColorMapper for MapBgraGray {
    type SrcType = Bgra<u8>;
    type DstType = Gray<u8>;

    #[inline(always)]
    fn to(src: &Self::SrcType) -> Self::DstType {
        let d = ((src[0] as u32 * 28
                + src[1] as u32 * 151
                + src[2] as u32 * 77) >> 8) as u8;
        Gray([d])
    }
}

pub struct MapGrayBgra;
impl ColorMapper for MapGrayBgra {
    type SrcType = Gray<u8>;
    type DstType = Bgra<u8>;

    #[inline(always)]
    fn to(src: &Self::SrcType) -> Self::DstType {
        let v = src[0];
        Bgra([v, v, v, 255])
    }
}

pub struct MapGrayBgr;
impl ColorMapper for MapGrayBgr {
    type SrcType = Gray<u8>;
    type DstType = Bgr<u8>;

    #[inline(always)]
    fn to(src: &Self::SrcType) -> Self::DstType {
        let v = src[0];
        Bgr([v, v, v])
    }
}

pub struct MapBgrGray;
impl ColorMapper for MapBgrGray {
    type SrcType = Bgr<u8>;
    type DstType = Gray<u8>;

    #[inline(always)]
    fn to(src: &Self::SrcType) -> Self::DstType {
        let d = ((src[0] as u32 * 28
                + src[1] as u32 * 151
                + src[2] as u32 * 77) >> 8) as u8;
        Gray([d])
    }
}

pub struct MapGrayGrayf;
impl ColorMapper for MapGrayGrayf {
    type SrcType = Gray<u8>;
    type DstType = Gray<f32>;

    #[inline(always)]
    fn to(src: &Self::SrcType) -> Self::DstType {
        Gray([src[0].to_f32().unwrap()])
    }
}

pub fn convert<M>(src: &Image<M::SrcType>) -> Image<M::DstType> 
    where M: ColorMapper {
    let mut dst = Image::new(src.width(), src.height());
    for h in 0..src.height() {
        let psrc = src.row(h);
        let pdst = dst.row_mut(h);
        for w in 0..src.width() as usize {
            pdst[w] = M::to(&psrc[w]);
        }
    }
    dst
}

pub fn split<T, U>(src: &Image<T>) -> Vec<Image<Gray<U>>> 
    where T: Pixel,
          U: Primitive,
          T: Index<usize, Output=U>
{
    let mut out = Vec::with_capacity(src.channels() as usize);
    for _ in 0..src.channels() {
        out.push(Image::<Gray<U>>::new(src.width(), src.height()));
    }
    // not the most efficiet way, but saving all out row ptr
    // will violate the borrow checker
    for y in 0..src.height() {
        let psrc = src.row(y);
        for c in 0..src.channels() as usize {
            let pdst = out[c].row_mut(y);
            for x in 0..src.width() as usize {
                pdst[x][0] = psrc[x][c];
            }
        }
    }
    out
}

pub fn merge<P, T>(src: &Vec<Image<Gray<P>>>) -> Image<T>
    where P: Primitive,
          T: Pixel,
          T: IndexMut<usize, Output=P>
{
    assert_eq!(T::channels() as usize, src.len());
    let src0 = &src[0];
    for i in src.iter() {
        assert_eq!(i.width(), src0.width());
        assert_eq!(i.height(), src0.height());
    }
    let mut out = Image::<T>::new(src0.width(), src0.height());
    for y in 0..src0.height() {
        let pdst = out.row_mut(y);
        for c in 0..T::channels() as usize {
            let psrc = src[c].row(y);
            for x in 0..src0.width() as usize {
                pdst[x][c] = psrc[x][0];
            }
        }
    }
    out
}

#[cfg(test)]
mod test {
    use image::*;
    use super::*;

    #[test]
    fn test_bgra_to_gray() {
        let mut src = ImageBgra::new(4, 1);
        {
            let r = src.row_mut(0);
            r[0] = Bgra([255, 0, 0, 255]);
            r[1] = Bgra([0, 255, 0, 255]);
            r[2] = Bgra([0, 0, 255, 255]);
            r[3] = Bgra([255, 255, 255, 255]);
        }
        let dst = convert::<MapBgraGray>(&src);
        assert_eq!(dst[(0, 0)], Gray([27]));
        assert_eq!(dst[(1, 0)], Gray([150]));
        assert_eq!(dst[(2, 0)], Gray([76]));
        assert_eq!(dst[(3, 0)], Gray([255]));
    }

    #[test]
    fn test_convert() {
        let mut src = ImageBgra::new(2000,1000);
        src.fill(&Bgra([100,100,100,255]));
        let out = convert::<MapBgraGray>(&src);
    }

    #[test]
    fn test_split_and_merge() {
        let mut src = ImageBgra::new(20,10);
        src.fill(&Bgra([100,100,100,255]));
        let t = split(&src);
        let out: ImageBgra = merge(&t);
        for (x, y, p) in out.iter() {
            assert_eq!(*p, src[(x, y)]);
        }
    }
}

