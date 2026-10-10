use crate::commands::{OptionReport, SourceInfo};
use crate::error::AppError;
use crate::ktx::options::CreateOptions;

#[tauri::command]
pub fn default_create_options() -> Result<CreateOptions, AppError> {
    Ok(CreateOptions::default())
}

#[tauri::command]
pub fn describe_options(
    _options: CreateOptions,
    _sources: Vec<SourceInfo>,
) -> Result<OptionReport, AppError> {
    Err(AppError::NotImplemented("describe_options"))
}

#[tauri::command]
pub async fn pick_sources() -> Result<Vec<SourceInfo>, AppError> {
    Err(AppError::NotImplemented("pick_sources"))
}

#[tauri::command]
pub async fn source_thumbnail(_path: String) -> Result<tauri::ipc::Response, AppError> {
    Err(AppError::NotImplemented("source_thumbnail"))
}
