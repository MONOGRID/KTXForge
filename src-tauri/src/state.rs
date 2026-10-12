use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use crate::error::AppError;
use crate::ktx::extract_ops::TextureMeta;
use crate::ktx::sidecar::CancellationHandle;
use crate::workspace::Workspace;

/// Per-encode job bookkeeping.
#[derive(Debug)]
pub struct Job {
    /// Queue output this job encodes; a newer encode of the same output replaces it.
    pub output_id: String,
    pub dir: PathBuf,
    pub output_path: PathBuf,
    /// Texture dims / preview hints from the finished encode (for subresource extracts).
    pub meta: Option<TextureMeta>,
    pub cancel: CancellationHandle,
    pub default_save_name: String,
}

#[derive(Debug, Default)]
pub struct AppState {
    workspace: Mutex<Option<Workspace>>,
    jobs: Mutex<HashMap<String, Job>>,
    pub inspector_file: Mutex<Option<PathBuf>>,
    /// Texture meta for the open inspector file (avoids re-running `ktx info` per image).
    pub inspector_meta: Mutex<Option<(PathBuf, TextureMeta)>>,
}

impl AppState {
    pub fn job_dir(&self, job_id: &str) -> Result<PathBuf, AppError> {
        self.with_workspace(|ws| ws.job_dir(job_id))
    }

    fn jobs(&self) -> Result<MutexGuard<'_, HashMap<String, Job>>, AppError> {
        self.jobs
            .lock()
            .map_err(|_| AppError::io("jobs lock poisoned"))
    }

    pub fn insert_job(&self, job_id: String, job: Job) -> Result<(), AppError> {
        self.jobs()?.insert(job_id, job);
        Ok(())
    }

    /// Read one job; unknown ids are an input error.
    pub fn with_job<T>(&self, job_id: &str, f: impl FnOnce(&Job) -> T) -> Result<T, AppError> {
        let jobs = self.jobs()?;
        let job = jobs
            .get(job_id)
            .ok_or_else(|| AppError::invalid_input(format!("unknown job: {job_id}")))?;
        Ok(f(job))
    }

    /// Update one job if it is still tracked (it may have been discarded meanwhile).
    pub fn update_job(&self, job_id: &str, f: impl FnOnce(&mut Job)) -> Result<(), AppError> {
        if let Some(job) = self.jobs()?.get_mut(job_id) {
            f(job);
        }
        Ok(())
    }

    /// Cancel and delete the jobs of these outputs (re-encoded or removed from the queue).
    pub fn discard_output_jobs(&self, output_ids: &[String]) -> Result<(), AppError> {
        let discarded: Vec<String> = {
            let mut jobs = self.jobs()?;
            let ids: Vec<String> = jobs
                .iter()
                .filter(|(_, job)| output_ids.contains(&job.output_id))
                .map(|(id, _)| id.clone())
                .collect();
            for id in &ids {
                if let Some(job) = jobs.remove(id) {
                    job.cancel.cancel();
                }
            }
            ids
        };
        self.with_workspace(|ws| {
            for id in &discarded {
                // Best effort: a cancelled `ktx` may still be writing; the root goes on exit.
                let _ = ws.remove_job(id);
            }
            Ok(())
        })
    }

    /// Drop every job and delete the temp workspace. Tauri exits the process without
    /// running destructors, so the `TempDir` must be dropped here.
    pub fn shutdown(&self) {
        if let Ok(mut jobs) = self.jobs.lock() {
            jobs.clear();
        }
        let workspace = self.workspace.lock().ok().and_then(|mut ws| ws.take());
        drop(workspace);
    }

    pub fn inspector_cache(&self, path: &Path) -> Result<PathBuf, AppError> {
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

    fn with_workspace<F, T>(&self, f: F) -> Result<T, AppError>
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

#[cfg(test)]
mod tests {
    use super::*;

    fn job(state: &AppState, job_id: &str, output_id: &str) -> PathBuf {
        let dir = state.job_dir(job_id).unwrap();
        state
            .insert_job(
                job_id.to_owned(),
                Job {
                    output_id: output_id.to_owned(),
                    dir: dir.clone(),
                    output_path: dir.join("output.ktx2"),
                    meta: None,
                    cancel: CancellationHandle::new(),
                    default_save_name: "out.ktx2".to_owned(),
                },
            )
            .unwrap();
        dir
    }

    #[test]
    fn discarding_an_output_cancels_and_deletes_only_its_jobs() {
        let state = AppState::default();
        let a = "0b7a3a52-91a4-4d1e-8d0e-3f1c5c1d2a01";
        let b = "0b7a3a52-91a4-4d1e-8d0e-3f1c5c1d2a02";
        let dir_a = job(&state, a, "out-1");
        let dir_b = job(&state, b, "out-2");
        let cancel_a = state.with_job(a, |j| j.cancel.clone()).unwrap();

        state.discard_output_jobs(&["out-1".to_owned()]).unwrap();

        assert!(cancel_a.is_cancelled());
        assert!(!dir_a.exists());
        assert!(state.with_job(a, |_| ()).is_err());
        assert!(dir_b.is_dir());
        assert!(state.with_job(b, |_| ()).is_ok());
    }

    #[test]
    fn shutdown_deletes_the_workspace() {
        let state = AppState::default();
        let dir = job(&state, "0b7a3a52-91a4-4d1e-8d0e-3f1c5c1d2a03", "out-1");
        let root = dir.parent().unwrap().to_path_buf();
        assert!(root.is_dir());

        state.shutdown();

        assert!(!root.exists());
    }
}
