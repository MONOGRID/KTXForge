//! Keep image decoding and file copies off the async runtime's worker threads.

use crate::error::AppError;

/// Run CPU-bound or blocking file work on the blocking thread pool.
pub async fn run_blocking<T, F>(f: F) -> Result<T, AppError>
where
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| AppError::io(format!("background task failed: {e}")))?
}
