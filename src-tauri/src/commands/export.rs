use crate::commands::{ExportRequest, ExportResult};
use crate::error::AppError;

#[tauri::command]
pub async fn export_images(_request: ExportRequest) -> Result<ExportResult, AppError> {
    Err(AppError::NotImplemented("export_images"))
}
