use crate::commands::UpdateInfo;
use crate::error::AppError;

#[tauri::command]
pub async fn check_update() -> Result<Option<UpdateInfo>, AppError> {
    Err(AppError::NotImplemented("check_update"))
}

#[tauri::command]
pub async fn install_update() -> Result<(), AppError> {
    Err(AppError::NotImplemented("install_update"))
}
