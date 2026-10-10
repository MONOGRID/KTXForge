//! PNG→JPEG and EXR tone-map. Implemented in T3.6.

use std::path::Path;

use crate::error::AppError;

pub fn png_to_jpeg(_png: &Path, _out: &Path, _quality: u8) -> Result<(), AppError> {
    Err(AppError::not_implemented("export::png_to_jpeg"))
}

pub fn exr_to_preview_png(_exr: &Path) -> Result<Vec<u8>, AppError> {
    Err(AppError::not_implemented("export::exr_to_preview_png"))
}
