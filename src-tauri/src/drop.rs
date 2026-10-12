//! Window drag-drop handler.

use std::path::PathBuf;

use tauri::{AppHandle, DragDropEvent, Emitter, Manager, Window, WindowEvent};

use crate::blocking::run_blocking;
use crate::commands::inspector_window::{self, INSPECTOR_LABEL};
use crate::error::AppError;
use crate::events::{DropRejectedPayload, ENCODER_DROP_REJECTED, ENCODER_SOURCES_ADDED};
use crate::image::classify::{classify, order_frames, FileKind};
use crate::image::convert::read_source_info;
use crate::state::AppState;

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    let WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event else {
        return;
    };
    let label = window.label().to_owned();
    let app = window.app_handle().clone();
    let paths = paths.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = handle_drop(&app, &label, paths).await {
            let _ = app.emit(
                ENCODER_DROP_REJECTED,
                DropRejectedPayload {
                    paths: Vec::new(),
                    reason: e.to_string(),
                },
            );
        }
    });
}

async fn handle_drop(app: &AppHandle, label: &str, paths: Vec<PathBuf>) -> Result<(), AppError> {
    if label == INSPECTOR_LABEL {
        handle_inspector_drop(app, paths).await
    } else {
        handle_main_drop(app, paths).await
    }
}

async fn handle_main_drop(app: &AppHandle, paths: Vec<PathBuf>) -> Result<(), AppError> {
    let ordered = order_frames(paths);
    let (images, ktx_files, rejected) = run_blocking(move || {
        let mut images = Vec::new();
        let mut ktx_files = Vec::new();
        let mut rejected = Vec::new();
        for path in ordered {
            match classify(&path) {
                FileKind::Image { .. } => match read_source_info(&path) {
                    Ok(info) => images.push(info),
                    Err(_) => rejected.push(path.to_string_lossy().into_owned()),
                },
                FileKind::Ktx1 | FileKind::Ktx2 => ktx_files.push(path),
                FileKind::Unsupported => rejected.push(path.to_string_lossy().into_owned()),
            }
        }
        Ok((images, ktx_files, rejected))
    })
    .await?;

    if !images.is_empty() {
        app.emit(ENCODER_SOURCES_ADDED, images)
            .map_err(|e| AppError::io(format!("emit sources-added: {e}")))?;
    }

    if let Some(first) = ktx_files.into_iter().next() {
        let state = app.state::<AppState>();
        inspector_window::open_inspector_path(app, state.inner(), first).await?;
    }

    if !rejected.is_empty() {
        app.emit(
            ENCODER_DROP_REJECTED,
            DropRejectedPayload {
                paths: rejected,
                reason: "unsupported file type".to_owned(),
            },
        )
        .map_err(|e| AppError::io(format!("emit drop-rejected: {e}")))?;
    }

    Ok(())
}

async fn handle_inspector_drop(app: &AppHandle, paths: Vec<PathBuf>) -> Result<(), AppError> {
    let ktx = paths
        .into_iter()
        .find(|p| matches!(classify(p), FileKind::Ktx1 | FileKind::Ktx2));
    let Some(path) = ktx else {
        return Ok(());
    };
    let state = app.state::<AppState>();
    inspector_window::open_inspector_path(app, state.inner(), path).await
}
