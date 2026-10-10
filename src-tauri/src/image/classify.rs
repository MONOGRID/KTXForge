//! Path classification and frame ordering. Implemented in T3.4.

use std::path::PathBuf;

use crate::error::AppError;

pub fn classify(_path: &std::path::Path) -> Result<(), AppError> {
    Err(AppError::NotImplemented("classify::classify"))
}

pub fn order_frames(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths
}
