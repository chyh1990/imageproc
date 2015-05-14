use std::path::Path;

use image::{ImageError, GenericImage};

pub trait ImageIO<T: GenericImage> {
    fn from_path(path: &Path) -> Result<T, ImageError>;

    fn save(path: &Path, image: &T) -> Result<(), ImageError>;
}

pub use self::freeimageio::FreeImageIO;

pub mod freeimageio;
