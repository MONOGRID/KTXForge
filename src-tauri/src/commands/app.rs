use crate::commands::KtxVersion;
use crate::error::AppError;

#[tauri::command]
pub async fn ktx_version() -> Result<KtxVersion, AppError> {
    Err(AppError::NotImplemented("ktx_version"))
}
