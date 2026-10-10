//! Parse SSIM/PSNR from `ktx create` stdout. Implemented in T3.9.

use crate::error::AppError;

pub fn parse_metrics(_stdout: &str) -> Result<(), AppError> {
    Err(AppError::NotImplemented("metrics::parse_metrics"))
}
