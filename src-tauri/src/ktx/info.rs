//! Run `ktx info` and parse. Implemented in T3.9.

use crate::error::AppError;
use crate::ktx::info_types::InspectReport;

pub async fn inspect(_path: &str) -> Result<InspectReport, AppError> {
    Err(AppError::NotImplemented("info::inspect"))
}
