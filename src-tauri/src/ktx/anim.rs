//! Write `KTXanimData` into a KTX2 file. Implemented in T3.7.

use std::path::Path;

use crate::error::AppError;
use crate::ktx::options::AnimationOptions;

pub fn write_anim_data(path: &Path, _anim: &AnimationOptions) -> Result<(), AppError> {
    // Touch the `ktx2` crate so the scaffold keeps the dependency; real write is T3.7.
    let _bytes = std::fs::read(path).map_err(|e| AppError::Io(e.to_string()))?;
    let _ = ktx2::Reader::new(_bytes.as_slice());
    Err(AppError::NotImplemented("anim::write_anim_data"))
}
