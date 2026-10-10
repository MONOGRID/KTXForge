//! KTX1 header reader. Implemented in T3.9.

use crate::error::AppError;

pub fn read_header(_bytes: &[u8]) -> Result<(), AppError> {
    Err(AppError::NotImplemented("ktx1::read_header"))
}
