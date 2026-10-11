use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::AppError;
use crate::ktx::extract_ops::TextureMeta;
use crate::ktx::info_types::PreviewKind;
use crate::ktx::sidecar::CancellationHandle;
use crate::workspace::Workspace;

/// Per-encode job bookkeeping.
#[derive(Debug)]
pub struct Job {
    pub dir: PathBuf,
    pub output_path: PathBuf,
    pub preview_png: PathBuf,
    pub preview_kind: PreviewKind,
    pub cancel: CancellationHandle,
    pub default_save_name: String,
}

#[derive(Debug, Default)]
pub struct AppState {
    workspace: Mutex<Option<Workspace>>,
    pub jobs: Mutex<HashMap<String, Job>>,
    pub inspector_file: Mutex<Option<PathBuf>>,
    /// Texture meta for the open inspector file (avoids re-running `ktx info` per image).
    pub inspector_meta: Mutex<Option<(PathBuf, TextureMeta)>>,
}

impl AppState {
    pub fn workspace_root(&self) -> Result<PathBuf, AppError> {
        self.with_workspace(|ws| Ok(ws.root().to_path_buf()))
    }

    pub fn job_dir(&self, job_id: &str) -> Result<PathBuf, AppError> {
        self.with_workspace(|ws| ws.job_dir(job_id))
    }

    pub fn inspector_cache(&self, path: &std::path::Path) -> Result<PathBuf, AppError> {
        self.with_workspace(|ws| {
            let mtime = std::fs::metadata(path)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            ws.inspector_cache(path, mtime)
        })
    }

    pub fn set_inspector_meta(&self, path: PathBuf, meta: TextureMeta) {
        if let Ok(mut guard) = self.inspector_meta.lock() {
            *guard = Some((path, meta));
        }
    }

    pub fn inspector_meta_for(&self, path: &Path) -> Option<TextureMeta> {
        let guard = self.inspector_meta.lock().ok()?;
        let (cached_path, meta) = guard.as_ref()?;
        if cached_path == path {
            Some(meta.clone())
        } else {
            None
        }
    }

    pub fn with_workspace<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Workspace) -> Result<T, AppError>,
    {
        let mut guard = self
            .workspace
            .lock()
            .map_err(|_| AppError::io("workspace lock poisoned"))?;
        if guard.is_none() {
            *guard = Some(Workspace::new()?);
        }
        f(guard.as_ref().expect("workspace just initialized"))
    }
}

/// Pending updater payload (separate manage so `Update` need not live in `AppState`).
pub struct PendingUpdate(pub Mutex<Option<tauri_plugin_updater::Update>>);

impl Default for PendingUpdate {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

/// Reserve a job id shape for Wave 4 (keeps `uuid` linked in the scaffold).
pub fn new_job_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
