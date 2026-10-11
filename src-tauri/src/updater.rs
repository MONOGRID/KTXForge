//! Launch-time update check.

use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_updater::UpdaterExt;

use crate::error::AppError;
use crate::events::{UpdateState, UpdateStatusPayload, UPDATE_STATUS};
use crate::state::PendingUpdate;

pub fn setup_updater<R: Runtime>(app: &AppHandle<R>) {
    if cfg!(debug_assertions) {
        let _ = app;
        return;
    }

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = run_update_check(handle).await;
    });
}

/// Check GitHub Releases and emit `update://status` when an update is available.
pub async fn run_update_check<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if cfg!(debug_assertions) {
        let _ = app;
        return Ok(());
    }
    let updater = match app.updater() {
        Ok(u) => u,
        Err(_) => return Ok(()),
    };

    let update = match updater.check().await {
        Ok(u) => u,
        Err(e) => {
            let msg = e.to_string();
            let lower = msg.to_ascii_lowercase();
            if lower.contains("endpoint") || lower.contains("url") || lower.contains("pubkey") {
                return Ok(());
            }
            let _ = app.emit(
                UPDATE_STATUS,
                UpdateStatusPayload {
                    state: UpdateState::Error,
                    version: None,
                    notes: None,
                    downloaded: None,
                    total: None,
                    message: Some(msg),
                },
            );
            return Ok(());
        }
    };

    let Some(update) = update else {
        return Ok(());
    };

    let version = update.version.clone();
    let notes = update.body.clone().unwrap_or_default();

    if let Some(pending) = app.try_state::<PendingUpdate>() {
        if let Ok(mut guard) = pending.0.lock() {
            *guard = Some(update);
        }
    }

    let _ = app.emit(
        UPDATE_STATUS,
        UpdateStatusPayload {
            state: UpdateState::Available,
            version: Some(version),
            notes: Some(notes),
            downloaded: None,
            total: None,
            message: None,
        },
    );
    Ok(())
}

/// Shared install path used by the `install_update` command.
pub async fn download_and_install(app: &AppHandle) -> Result<(), AppError> {
    let pending = app
        .try_state::<PendingUpdate>()
        .ok_or_else(|| AppError::update("no pending update"))?;
    let update = {
        let mut guard = pending
            .0
            .lock()
            .map_err(|_| AppError::io("pending update lock poisoned"))?;
        guard
            .take()
            .ok_or_else(|| AppError::update("no pending update"))?
    };

    let app_for_progress = app.clone();
    let mut downloaded: u64 = 0;

    update
        .download_and_install(
            |chunk_len, content_len| {
                downloaded += chunk_len as u64;
                let _ = app_for_progress.emit(
                    UPDATE_STATUS,
                    UpdateStatusPayload {
                        state: UpdateState::Downloading,
                        version: None,
                        notes: None,
                        downloaded: Some(downloaded),
                        total: content_len,
                        message: None,
                    },
                );
            },
            || {},
        )
        .await
        .map_err(|e| AppError::update(e.to_string()))?;

    let _ = app.emit(
        UPDATE_STATUS,
        UpdateStatusPayload {
            state: UpdateState::Installed,
            version: None,
            notes: None,
            downloaded: None,
            total: None,
            message: None,
        },
    );

    app.request_restart();
    Ok(())
}
