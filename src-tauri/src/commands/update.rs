use crate::commands::UpdateInfo;
use crate::error::AppError;

#[tauri::command]
pub async fn check_update() -> Result<Option<UpdateInfo>, AppError> {
    Err(AppError::not_implemented("check_update"))
}

#[tauri::command]
pub async fn install_update() -> Result<(), AppError> {
    Err(AppError::not_implemented("install_update"))
}
