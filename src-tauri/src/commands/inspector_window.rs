use std::path::PathBuf;

use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::error::AppError;
use crate::events::{InspectorFilePayload, INSPECTOR_FILE};
use crate::state::AppState;

pub const INSPECTOR_LABEL: &str = "inspector";

#[tauri::command]
pub async fn open_inspector(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<(), AppError> {
    open_inspector_path(&app, &state, PathBuf::from(path)).await
}

#[tauri::command]
pub fn inspector_current_file(state: State<'_, AppState>) -> Result<Option<String>, AppError> {
    let guard = state
        .inspector_file
        .lock()
        .map_err(|_| AppError::io("inspector_file lock poisoned"))?;
    Ok(guard.as_ref().map(|p| p.to_string_lossy().into_owned()))
}

pub async fn open_inspector_path(
    app: &AppHandle,
    state: &AppState,
    path: PathBuf,
) -> Result<(), AppError> {
    {
        let mut guard = state
            .inspector_file
            .lock()
            .map_err(|_| AppError::io("inspector_file lock poisoned"))?;
        *guard = Some(path.clone());
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_owned();
    let title = format!("KTXForge Inspector — {file_name}");
    let payload = InspectorFilePayload {
        path: path.to_string_lossy().into_owned(),
    };

    if let Some(window) = app.get_webview_window(INSPECTOR_LABEL) {
        let _ = window.set_title(&title);
        let _ = window.set_focus();
        window
            .emit(INSPECTOR_FILE, payload)
            .map_err(|e| AppError::io(format!("emit inspector file: {e}")))?;
        return Ok(());
    }

    let url = WebviewUrl::App("inspector.html".into());
    let builder = WebviewWindowBuilder::new(app, INSPECTOR_LABEL, url)
        .title(title)
        .inner_size(1100.0, 760.0);
    let window = crate::window_bounds::with_saved_bounds(builder, app, INSPECTOR_LABEL)
        .build()
        .map_err(|e| AppError::io(format!("create inspector window: {e}")))?;

    // Cold OS-open destroys hidden `main` first; if that tore down the shared
    // native menu, the inspector would otherwise open with no menu bar.
    #[cfg(desktop)]
    crate::menu::ensure_app_menu_on_window(app, &window);

    // Fresh window may miss the first emit if listeners are not ready yet; it also
    // loads via `inspector_current_file` on mount.
    let _ = window.emit(INSPECTOR_FILE, payload);
    Ok(())
}
