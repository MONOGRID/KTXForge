//! `CreateOptions` → `ktx create` argv. Implemented in T3.2.

use std::path::Path;

use crate::error::AppError;
use crate::ktx::options::CreateOptions;

pub fn build_create_args(
    _opts: &CreateOptions,
    _inputs: &[std::path::PathBuf],
    _output: &Path,
) -> Result<Vec<String>, AppError> {
    Err(AppError::not_implemented("create_args::build_create_args"))
}
