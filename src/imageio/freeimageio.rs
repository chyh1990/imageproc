extern crate libc;

use std::ffi::CString;
use std::path::Path;
use std::ptr;
use self::libc::{c_int, c_void, c_char, c_uchar};

use imageio::ImageIO;
use image::{ImageBGRA, ImageError};

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone)]
enum ImageFormat {
	FIF_UNKNOWN = -1,
	FIF_BMP		= 0,
	FIF_ICO		= 1,
	FIF_JPEG	= 2,
	FIF_JNG		= 3,
	FIF_KOALA	= 4,
	FIF_LBM		= 5,
	FIF_MNG		= 6,
	FIF_PBM		= 7,
	FIF_PBMRAW	= 8,
	FIF_PCD		= 9,
	FIF_PCX		= 10,
	FIF_PGM		= 11,
	FIF_PGMRAW	= 12,
	FIF_PNG		= 13,
	FIF_PPM		= 14,
	FIF_PPMRAW	= 15,
	FIF_RAS		= 16,
	FIF_TARGA	= 17,
	FIF_TIFF	= 18,
	FIF_WBMP	= 19,
	FIF_PSD		= 20,
	FIF_CUT		= 21,
	FIF_XBM		= 22,
	FIF_XPM		= 23,
	FIF_DDS		= 24,
	FIF_GIF     = 25,
	FIF_HDR		= 26,
	FIF_FAXG3	= 27,
	FIF_SGI		= 28,
	FIF_EXR		= 29,
	FIF_J2K		= 30,
	FIF_JP2		= 31,
	FIF_PFM		= 32,
	FIF_PICT	= 33,
	FIF_RAW		= 34,
	FIF_WEBP	= 35,
	FIF_JXR		= 36
}

const JPEG_EXIFROTATE: c_int = 0x0008;

#[link(name = "freeimage-3.17.0")]
extern {
    fn FreeImage_Load(fif :ImageFormat, filename: *const c_char, flag: c_int) -> *mut c_void;
    fn FreeImage_Unload(dib: *mut c_void);

    fn FreeImage_GetFileType(filename: *const c_char, size: c_int) -> ImageFormat;

    fn FreeImage_GetWidth(dib: *mut c_void) -> u32;
    fn FreeImage_GetHeight(dib: *mut c_void) -> u32;
    fn FreeImage_GetBPP(dib: *mut c_void) -> u32;
    fn FreeImage_GetPitch(dib: *mut c_void) -> u32;
    fn FreeImage_GetBits(dib: *mut c_void) -> *mut c_uchar;

    fn FreeImage_ConvertToGreyscale(dib: *mut c_void) -> *mut c_void;
    fn FreeImage_ConvertTo32Bits(dib: *mut c_void) -> *mut c_void;
    fn FreeImage_Clone(dib: *mut c_void) -> *mut c_void;
}

pub struct FreeImageIO;

impl ImageIO<ImageBGRA> for FreeImageIO {
    fn from_path(path: &Path) -> Result<ImageBGRA, ImageError> {
        let c_path = CString::new(path.to_str().unwrap()).unwrap();
        let format = unsafe { FreeImage_GetFileType(c_path.as_ptr(), 0) };
        if format == ImageFormat::FIF_UNKNOWN {
            return Err(ImageError::UnknownImageFormat);
        }
        let mut flags: c_int = 0;
        if format == ImageFormat::FIF_JPEG {
            flags |= JPEG_EXIFROTATE;
        }
        let p = unsafe { FreeImage_Load(format, c_path.as_ptr(), flags) };
        if p.is_null() {
            Err(ImageError::InvalidImage)
        } else {
            let np;
            let old_bpp = unsafe{ FreeImage_GetBPP(p) };
            if old_bpp != 32 {
                np = unsafe { FreeImage_ConvertTo32Bits(p) };
                unsafe { FreeImage_Unload(p) };
            } else {
                np = p;
            }

            let w = unsafe { FreeImage_GetWidth(np) };
            let h = unsafe { FreeImage_GetHeight(np) };
            let pitch = unsafe { FreeImage_GetPitch(np) };
            let mut image = ImageBGRA::new(w, h);

            {
                let stride_dst = image.pitch();
                let mut dst = image.raw_mut();
                let pdst = dst.as_mut_ptr();
                let sptr = unsafe { FreeImage_GetBits(np) };
                if sptr.is_null() {
                    panic!("No image data!");
                }

                unsafe {
                    let sptr_end =  sptr.offset((pitch * (h - 1)) as isize);
                    for y in 0..h {
                        // freeimage save image reversely
                        ptr::copy(sptr_end.offset(-((y * pitch) as isize)),
                            pdst.offset((y * stride_dst) as isize),
                            stride_dst as usize);
                    }
                }

            }

            unsafe { FreeImage_Unload(np) };
            Ok(image)
        }

    }
}

#[cfg(test)]
mod test {
    use std::env;
    use std::path::Path;
    use imageio::ImageIO;
    use imageio::FreeImageIO;

    #[test]
    fn test_load() {
        let path = Path::new("./tests/cat.jpg");
        let img = FreeImageIO::from_path(&path).unwrap();
        assert_eq!(img.width(), 150);
        assert_eq!(img.height(), 120);
        assert_eq!(img.bits_per_pixel(), 32);
        assert_eq!(img.channels(), 4);
    }
}

