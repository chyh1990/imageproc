use std::path::Path;

use image::{ImageBGRA, ImageError, GenericImage};

pub mod freeimageio;

pub trait ImageIO<T: GenericImage> {
    fn from_path(path: &Path) -> Result<T, ImageError>;

    // fn save(image: &T) -> Result<(), ImageError>;
}

pub use self::freeimageio::FreeImageIO;
