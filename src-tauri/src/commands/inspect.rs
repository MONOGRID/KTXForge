use crate::error::AppError;
use crate::ktx::info_types::{ImageSelection, InspectReport};

#[tauri::command]
pub async fn inspect(_path: String) -> Result<InspectReport, AppError> {
    Err(AppError::not_implemented("inspect"))
}

#[tauri::command]
pub async fn inspector_image(
    _path: String,
    _selection: ImageSelection,
) -> Result<tauri::ipc::Response, AppError> {
    Err(AppError::not_implemented("inspector_image"))
}
