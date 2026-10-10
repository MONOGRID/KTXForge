use crate::commands::{EncodeRequest, EncodeResult};
use crate::error::AppError;

#[tauri::command]
pub async fn encode(_request: EncodeRequest) -> Result<EncodeResult, AppError> {
    let _runtime = tokio::runtime::Handle::try_current();
    Err(AppError::not_implemented("encode"))
}

#[tauri::command]
pub async fn cancel_encode(_job_id: String) -> Result<(), AppError> {
    Err(AppError::not_implemented("cancel_encode"))
}

#[tauri::command]
pub async fn encoded_preview(_job_id: String) -> Result<tauri::ipc::Response, AppError> {
    Err(AppError::not_implemented("encoded_preview"))
}

#[tauri::command]
pub async fn save_encoded(_job_id: String) -> Result<Option<String>, AppError> {
    Err(AppError::not_implemented("save_encoded"))
}
