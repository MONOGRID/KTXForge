use crate::error::AppError;

#[tauri::command]
pub async fn open_inspector(_path: String) -> Result<(), AppError> {
    Err(AppError::NotImplemented("open_inspector"))
}

#[tauri::command]
pub fn inspector_current_file() -> Result<Option<String>, AppError> {
    Err(AppError::NotImplemented("inspector_current_file"))
}
