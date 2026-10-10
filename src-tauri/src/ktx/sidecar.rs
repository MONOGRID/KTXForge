//! Run the embedded `ktx` sidecar. Implemented in T3.1.

use crate::error::AppError;

pub fn exit_meaning(code: i32) -> &'static str {
    AppError::exit_meaning(code)
}
