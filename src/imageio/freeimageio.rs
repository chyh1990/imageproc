extern crate libc;

use std::ffi::CString;
use std::path::Path;
use std::ptr;
use self::libc::{c_int, c_uint, c_void, c_char, c_uchar};
use std::sync::{Once, ONCE_INIT};

use imageio::ImageIO;
use image::{ImageBgra, ImageBgr, ImageGray,
    ImageError, Image, Pixel};

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

#[link(name = "freeimage", kind = "static")]
extern {
    fn FreeImage_Initialise(load_local_only: c_int);
    fn FreeImage_DeInitialise();
    fn FreeImage_Allocate(width: c_int, height: c_int, bpp: c_int, red_mask: c_uint, green_mask: c_uint, blue_mask: c_uint) -> *mut c_void;
    fn FreeImage_Load(fif: ImageFormat, filename: *const c_char, flag: c_int) -> *mut c_void;
    fn FreeImage_Save(fif: ImageFormat, dib: *mut c_void, filename: *const c_char, flags: c_int) -> c_int;
    fn FreeImage_Unload(dib: *mut c_void);

    fn FreeImage_GetFileType(filename: *const c_char, size: c_int) -> ImageFormat;
    fn FreeImage_GetFIFFromFilename(filename: *const c_char) -> ImageFormat;

    fn FreeImage_GetWidth(dib: *mut c_void) -> u32;
    fn FreeImage_GetHeight(dib: *mut c_void) -> u32;
    fn FreeImage_GetBPP(dib: *mut c_void) -> u32;
    fn FreeImage_GetPitch(dib: *mut c_void) -> u32;
    fn FreeImage_GetBits(dib: *mut c_void) -> *mut c_uchar;

    fn FreeImage_ConvertToGreyscale(dib: *mut c_void) -> *mut c_void;
    fn FreeImage_ConvertTo32Bits(dib: *mut c_void) -> *mut c_void;
    fn FreeImage_ConvertTo24Bits(dib: *mut c_void) -> *mut c_void;
    fn FreeImage_Clone(dib: *mut c_void) -> *mut c_void;
}

fn init() {
    static LIBSTART: Once = ONCE_INIT;
    LIBSTART.call_once(|| {
        // XXX not unloaded
        unsafe { FreeImage_Initialise(0); }
    });
}

unsafe fn from_raw<T: Pixel>(np: *mut c_void) -> Image<T> {
    let w = FreeImage_GetWidth(np);
    let h = FreeImage_GetHeight(np);
    let pitch = FreeImage_GetPitch(np);
    let mut image = Image::new(w, h);
    {
        let stride_dst = image.pitch();
        let mut dst = image.raw_mut();
        let pdst = dst.as_mut_ptr() as *mut u8;
        let sptr = FreeImage_GetBits(np);
        if sptr.is_null() {
            panic!("No image data!");
        }

        let sptr_end =  sptr.offset((pitch * (h - 1)) as isize);
        for y in 0..h {
            // freeimage save image reversely
            ptr::copy(sptr_end.offset(-((y * pitch) as isize)),
            pdst.offset((y * stride_dst) as isize),
            stride_dst as usize);
        }

    }
    FreeImage_Unload(np);
    image
}

unsafe fn try_load_from_file(path: &Path, bits :u8) -> *mut c_void {
    let c_path = CString::new(path.to_str().unwrap()).unwrap();
    let format = FreeImage_GetFileType(c_path.as_ptr(), 0);
    if format == ImageFormat::FIF_UNKNOWN {
        return ptr::null_mut();
    }
    let mut flags: c_int = 0;
    if format == ImageFormat::FIF_JPEG {
        flags |= JPEG_EXIFROTATE;
    }
    let p = FreeImage_Load(format, c_path.as_ptr(), flags);
    let np;
    let old_bpp = FreeImage_GetBPP(p);
    if old_bpp != bits as u32 {
        match bits {
            8  => np = FreeImage_ConvertToGreyscale(p),
            24 => np = FreeImage_ConvertTo24Bits(p),
            32 => np = FreeImage_ConvertTo32Bits(p),
            _ => return ptr::null_mut()
        }
        FreeImage_Unload(p);
    } else {
        np = p;
    }
    np
}

unsafe fn to_raw<T: Pixel>(image: &Image<T>) -> *mut c_void {
    let src_bits = image.bits_per_pixel() as i32;
    assert!(src_bits == 8 || src_bits == 24 || src_bits == 32);
    // XXX opt me
    let p = FreeImage_Allocate(image.width() as i32,
    image.height() as i32, src_bits, 0, 0, 0);
    if p.is_null() {
        return p;
    }

    {
        let h = FreeImage_GetHeight(p);
        let pitch = FreeImage_GetPitch(p);
        let stride_src = image.pitch();
        let src = image.raw();
        let psrc = src.as_ptr() as *const u8;
        let dptr = FreeImage_GetBits(p);
        if dptr.is_null() {
            panic!("No image data!");
        }

        let dptr_end =  dptr.offset((pitch * (h - 1)) as isize);
        for y in 0..h {
            // freeimage save image reversely
            ptr::copy(psrc.offset((y * stride_src) as isize),
            dptr_end.offset(-((y * pitch) as isize)),
            stride_src as usize);
        }
    }
    p
}

unsafe fn save_raw_to_file(p: *mut c_void, path: &Path, src_bits: u8, format: ImageFormat) -> i32 {
    let code;
    let c_path = CString::new(path.to_str().unwrap()).unwrap();
    assert!(src_bits == 8 || src_bits == 24 || src_bits == 32);
    if format != ImageFormat::FIF_JPEG || src_bits != 32 {
        code = FreeImage_Save(format, p, c_path.as_ptr(), 0);
        FreeImage_Unload(p);
    } else {
        let np = FreeImage_ConvertTo24Bits(p);
        FreeImage_Unload(p);
        if np.is_null() {
            return 0;
        }
        code = FreeImage_Save(format, np, c_path.as_ptr(), 0);
        FreeImage_Unload(np);
    }
    code
}

pub struct FreeImageIO;

macro_rules! define_io_for_image(
    ($itype:ident, $bits:expr) => (
        impl ImageIO<$itype> for FreeImageIO {
            fn from_path(path: &Path) -> Result<$itype, ImageError> {
                init();
                let p = unsafe { try_load_from_file(path, $bits) };
                if p.is_null() {
                    Err(ImageError::InvalidImage)
                } else {
                    let image = unsafe { from_raw(p) };
                    Ok(image)
                }
            }

            fn save(path: &Path, image: &$itype) -> Result<(), ImageError> {
                init();

                let c_path = CString::new(path.to_str().unwrap()).unwrap();
                let format = unsafe { FreeImage_GetFIFFromFilename(c_path.as_ptr()) };
                if format == ImageFormat::FIF_UNKNOWN {
                    return Err(ImageError::UnknownImageFormat);
                }

                let p = unsafe { to_raw(image) };
                if p.is_null() {
                    return Err(ImageError::OutOfMemoryError);
                }

                if unsafe { save_raw_to_file(p, path, $bits, format) } != 0 {
                    Ok(())
                } else {
                    Err(ImageError::UnknownError)
                }
            }
        }

);
);

define_io_for_image!(ImageGray, 8);
define_io_for_image!(ImageBgr , 24);
define_io_for_image!(ImageBgra, 32);

#[cfg(test)]
mod test {
    use std::path::Path;
    use convert;
    use image::*;
    use imageio::ImageIO;
    use imageio::FreeImageIO;

    #[test]
    fn test_load() {
        let path = Path::new("./tests/cat.jpg");
        let img: ImageBgra = FreeImageIO::from_path(&path).unwrap();
        assert_eq!(img.width(), 150);
        assert_eq!(img.height(), 120);
        assert_eq!(img.bits_per_pixel(), 32);
        assert_eq!(img.channels(), 4);

        let img: ImageBgr = FreeImageIO::from_path(&path).unwrap();
        assert_eq!(img.width(), 150);
        assert_eq!(img.height(), 120);
        assert_eq!(img.bits_per_pixel(), 24);
        assert_eq!(img.channels(), 3);

        let img: ImageGray = FreeImageIO::from_path(&path).unwrap();
        assert_eq!(img.width(), 150);
        assert_eq!(img.height(), 120);
        assert_eq!(img.bits_per_pixel(), 8);
        assert_eq!(img.channels(), 1);
    }

    #[test]
    fn test_save() {
        let path = Path::new("./tests/cat.jpg");
        let img: ImageBgra = FreeImageIO::from_path(&path).unwrap();
        let target = Path::new("/tmp/test-out.png");
        FreeImageIO::save(&target, &img).unwrap();

        let target = Path::new("/tmp/test-out-32.jpg");
        FreeImageIO::save(&target, &img).unwrap();

        let gray = convert::convert::<convert::MapBgraGray>(&img);
        let target = Path::new("/tmp/test-out-8.jpg");
        FreeImageIO::save(&target, &gray).unwrap();
    }
}

