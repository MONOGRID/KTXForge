//! Input conversion and thumbnails. Implemented in T3.5.

use std::path::Path;

use crate::error::AppError;

pub fn prepare_input(_src: &Path, _workdir: &Path) -> Result<(), AppError> {
    let _ = image::ImageFormat::Png;
    Err(AppError::not_implemented("convert::prepare_input"))
}

pub fn thumbnail_png(_path: &Path, _max: u32) -> Result<Vec<u8>, AppError> {
    Err(AppError::not_implemented("convert::thumbnail_png"))
}
