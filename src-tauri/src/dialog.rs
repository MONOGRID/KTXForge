//! Native file dialogs, awaited instead of blocking a runtime worker thread.

use std::path::PathBuf;

use tauri::Runtime;
use tauri_plugin_dialog::{FileDialogBuilder, FilePath};
use tokio::sync::oneshot;

use crate::error::AppError;

async fn answer<T: Send + 'static>(show: impl FnOnce(oneshot::Sender<Option<T>>)) -> Option<T> {
    let (tx, rx) = oneshot::channel();
    show(tx);
    rx.await.ok().flatten()
}

fn into_path(file: FilePath) -> Result<PathBuf, AppError> {
    file.into_path()
        .map_err(|e| AppError::io(format!("dialog path: {e}")))
}

/// Chosen files; empty when the user cancels.
pub async fn pick_files<R: Runtime>(
    dialog: FileDialogBuilder<R>,
) -> Result<Vec<PathBuf>, AppError> {
    let files = answer(|tx| {
        dialog.pick_files(move |files| {
            let _ = tx.send(files);
        });
    })
    .await;
    files
        .unwrap_or_default()
        .into_iter()
        .map(into_path)
        .collect()
}

/// Chosen folder; `None` when the user cancels.
pub async fn pick_folder<R: Runtime>(
    dialog: FileDialogBuilder<R>,
) -> Result<Option<PathBuf>, AppError> {
    let folder = answer(|tx| {
        dialog.pick_folder(move |folder| {
            let _ = tx.send(folder);
        });
    })
    .await;
    folder.map(into_path).transpose()
}

/// Chosen save path; `None` when the user cancels.
pub async fn save_file<R: Runtime>(
    dialog: FileDialogBuilder<R>,
) -> Result<Option<PathBuf>, AppError> {
    let file = answer(|tx| {
        dialog.save_file(move |file| {
            let _ = tx.send(file);
        });
    })
    .await;
    file.map(into_path).transpose()
}
