use std::path::PathBuf;

use tauri::{AppHandle, Runtime, State};

use crate::error::AppError;
use crate::ktx::extract_ops::{
    cached_preview_png, ensure_ktx2, extract_preview_png, texture_meta_from_report,
    texture_meta_via_info,
};
use crate::ktx::info;
use crate::ktx::info_types::{ImageSelection, InspectKind, InspectReport};
use crate::ktx::sidecar::CancellationHandle;
use crate::state::AppState;

#[tauri::command]
pub async fn inspect(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<InspectReport, AppError> {
    let path_buf = PathBuf::from(&path);
    let report = info::inspect(&app, path_buf.as_path()).await?;
    // KTX1 meta comes from the converted KTX2 on the first image request.
    if matches!(report.kind, InspectKind::Ktx2 { .. }) {
        state.set_inspector_meta(path_buf, texture_meta_from_report(&report));
    }
    Ok(report)
}

#[tauri::command]
pub async fn inspector_image(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    selection: ImageSelection,
) -> Result<tauri::ipc::Response, AppError> {
    let bytes = inspector_image_png(&app, &state, PathBuf::from(path), &selection).await?;
    Ok(tauri::ipc::Response::new(bytes))
}

/// PNG the inspector viewport shows for one subresource.
pub async fn inspector_image_png<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    path: PathBuf,
    selection: &ImageSelection,
) -> Result<Vec<u8>, AppError> {
    let work = state.inspector_cache(&path)?;
    let cache = work.join("images");

    // Disk cache hit: no sidecar, no info parse.
    if let Some(bytes) = cached_preview_png(&cache, selection).await {
        return Ok(bytes);
    }

    let ktx2_path = ensure_ktx2(app, &path, &work).await?;
    let meta = match state.inspector_meta_for(&path) {
        Some(meta) => meta,
        None => {
            let meta = texture_meta_via_info(app, &ktx2_path).await?;
            state.set_inspector_meta(path.clone(), meta.clone());
            meta
        }
    };

    let (bytes, _) = extract_preview_png(
        app,
        &ktx2_path,
        selection,
        &meta,
        &cache,
        CancellationHandle::new(),
    )
    .await?;
    Ok(bytes)
}
