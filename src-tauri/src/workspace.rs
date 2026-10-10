//! Temp directories per encode job. Implemented in T3.10.

use crate::error::AppError;

#[derive(Debug, Default)]
pub struct Workspace;

impl Workspace {
    pub fn new() -> Result<Self, AppError> {
        let _builder = tempfile::Builder::new().prefix("ktx-encoder-");
        Err(AppError::not_implemented("Workspace::new"))
    }
}
