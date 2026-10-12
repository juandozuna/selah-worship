//! Load still images for stream compositing.

use crate::composite::RgbaImage;
use std::path::Path;

pub fn load_rgba(path: &Path) -> Result<RgbaImage, String> {
    let img = image::open(path).map_err(|e| format!("Could not open media image: {e}"))?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    RgbaImage::new(width, height, rgba.into_raw())
}
