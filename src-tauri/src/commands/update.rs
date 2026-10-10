use tauri::{AppHandle, Emitter, State};

use crate::commands::UpdateInfo;
use crate::error::AppError;
use crate::events::{UpdateState, UpdateStatusPayload, UPDATE_STATUS};
use crate::state::PendingUpdate;
use crate::updater;

#[tauri::command]
pub async fn check_update(
    app: AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<Option<UpdateInfo>, AppError> {
    if cfg!(debug_assertions) {
        let _ = (&app, &pending);
        return Ok(None);
    }

    use tauri_plugin_updater::UpdaterExt;

    let updater = match app.updater() {
        Ok(u) => u,
        Err(_) => return Ok(None),
    };
    let update = updater
        .check()
        .await
        .map_err(|e| AppError::update(e.to_string()))?;
    let Some(update) = update else {
        return Ok(None);
    };
    let info = UpdateInfo {
        version: update.version.clone(),
        notes: update.body.clone().unwrap_or_default(),
        date: update.date.map(|d| d.to_string()).unwrap_or_default(),
    };
    *pending
        .0
        .lock()
        .map_err(|_| AppError::io("pending update lock poisoned"))? = Some(update);
    Ok(Some(info))
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), AppError> {
    match updater::download_and_install(&app).await {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = app.emit(
                UPDATE_STATUS,
                UpdateStatusPayload {
                    state: UpdateState::Error,
                    version: None,
                    notes: None,
                    downloaded: None,
                    total: None,
                    message: Some(e.to_string()),
                },
            );
            Err(e)
        }
    }
}
