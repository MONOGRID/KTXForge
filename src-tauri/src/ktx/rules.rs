//! Option rules engine. Implemented in T3.3.

use crate::error::AppError;
use crate::ktx::options::CreateOptions;

pub fn describe(_opts: &CreateOptions) -> Result<(), AppError> {
    Err(AppError::NotImplemented("rules::describe"))
}
