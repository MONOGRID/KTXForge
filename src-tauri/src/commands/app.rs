use crate::commands::KtxVersion;
use crate::error::AppError;

#[tauri::command]
pub async fn ktx_version() -> Result<KtxVersion, AppError> {
    Err(AppError::not_implemented("ktx_version"))
}
