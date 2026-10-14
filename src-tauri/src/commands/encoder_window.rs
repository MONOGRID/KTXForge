//! Recreate / focus the encoder (`main`) window when File → Open picks images
//! while only the inspector is up (e.g. after a cold OS-open destroyed `main`).

use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::commands::SourceInfo;
use crate::error::AppError;
use crate::events::ENCODER_SOURCES_ADDED;

pub const ENCODER_LABEL: &str = "main";

/// Images from File → Open while `main` was missing; drained on encoder mount.
#[derive(Default)]
pub struct PendingEncoderSources(Mutex<Option<Vec<SourceInfo>>>);

/// Show the encoder with `sources`. Recreates `main` when it was destroyed.
///
/// A freshly built window may miss the first emit, so sources are also stored
/// for [`take_pending_encoder_sources`] (same pattern as `inspector_current_file`).
pub async fn deliver_sources(app: &AppHandle, sources: Vec<SourceInfo>) -> Result<(), AppError> {
    if sources.is_empty() {
        return Ok(());
    }

    if let Some(window) = app.get_webview_window(ENCODER_LABEL) {
        clear_pending(app)?;
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        app.emit(ENCODER_SOURCES_ADDED, sources)
            .map_err(|e| AppError::io(format!("emit sources-added: {e}")))?;
        return Ok(());
    }

    {
        let pending = app.state::<PendingEncoderSources>();
        let mut guard = pending
            .0
            .lock()
            .map_err(|_| AppError::io("pending_encoder_sources lock poisoned"))?;
        *guard = Some(sources);
    }

    create_encoder_window(app).await
}

async fn create_encoder_window(app: &AppHandle) -> Result<(), AppError> {
    let url = WebviewUrl::App("index.html".into());
    let builder = WebviewWindowBuilder::new(app, ENCODER_LABEL, url)
        .title("KTXForge")
        .inner_size(1100.0, 720.0)
        .min_inner_size(960.0, 640.0)
        .visible(true);
    let window = crate::window_bounds::with_saved_bounds(builder, app, ENCODER_LABEL)
        .build()
        .map_err(|e| AppError::io(format!("create encoder window: {e}")))?;

    #[cfg(desktop)]
    crate::menu::ensure_app_menu_on_window(app, &window);

    let _ = window.set_focus();
    Ok(())
}

fn clear_pending(app: &AppHandle) -> Result<(), AppError> {
    let pending = app.state::<PendingEncoderSources>();
    let mut guard = pending
        .0
        .lock()
        .map_err(|_| AppError::io("pending_encoder_sources lock poisoned"))?;
    *guard = None;
    Ok(())
}

/// Drain sources queued for a freshly created encoder window.
#[tauri::command]
pub fn take_pending_encoder_sources(
    pending: State<'_, PendingEncoderSources>,
) -> Result<Option<Vec<SourceInfo>>, AppError> {
    let mut guard = pending
        .0
        .lock()
        .map_err(|_| AppError::io("pending_encoder_sources lock poisoned"))?;
    Ok(guard.take())
}
