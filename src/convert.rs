#![allow(non_camel_case_types)]
use image::*;

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
        assert_eq!(*dst.pixel_at(0, 0), Gray([27]));
        assert_eq!(*dst.pixel_at(1, 0), Gray([150]));
        assert_eq!(*dst.pixel_at(2, 0), Gray([76]));
        assert_eq!(*dst.pixel_at(3, 0), Gray([255]));
    }

    #[test]
    fn test_convert() {
        let mut src = ImageBgra::new(2000,1000);
        src.fill(&Bgra([100,100,100,255]));
        let out = convert::<MapBgraGray>(&src);
    }
}

