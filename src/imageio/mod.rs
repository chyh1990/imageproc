use std::path::Path;

use image::{ImageError, GenericImage};

pub struct ImageInfo {
    pub signature: String,
    pub width: u32,
    pub height: u32,
    pub bits_per_pixel: u32,
}

pub trait ImageIO<T: GenericImage> {
    fn from_path(path: &Path) -> Result<T, ImageError>;
    fn save(path: &Path, image: &T) -> Result<(), ImageError>;
}

pub trait ImagePing {
    fn ping_from_path(path: &Path) -> Result<ImageInfo, ImageError>;
}

pub use self::freeimageio::FreeImageIO;

pub mod freeimageio;
