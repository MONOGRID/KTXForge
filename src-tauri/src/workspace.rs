//! Temp directories per encode job and inspector extract cache.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tempfile::TempDir;
use uuid::Uuid;

use crate::error::AppError;

/// App-scoped temp root (`ktx-encoder-*`) removed on drop.
#[derive(Debug)]
pub struct Workspace {
    root: TempDir,
}

impl Workspace {
    pub fn new() -> Result<Self, AppError> {
        let root = tempfile::Builder::new()
            .prefix("ktx-encoder-")
            .tempdir()
            .map_err(|e| AppError::io(format!("workspace tempdir: {e}")))?;
        Ok(Self { root })
    }

    /// Job ids come from the webview and name a directory: accept only a UUID.
    fn job_path(&self, job_id: &str) -> Result<PathBuf, AppError> {
        let id = Uuid::parse_str(job_id)
            .map_err(|_| AppError::invalid_input(format!("job id is not a UUID: {job_id}")))?;
        Ok(self.root.path().join(id.hyphenated().to_string()))
    }

    /// Create (or reuse) a subdirectory for one encode job.
    pub fn job_dir(&self, job_id: &str) -> Result<PathBuf, AppError> {
        let dir = self.job_path(job_id)?;
        std::fs::create_dir_all(&dir).map_err(|e| AppError::io(e.to_string()))?;
        Ok(dir)
    }

    pub fn remove_job(&self, job_id: &str) -> Result<(), AppError> {
        let dir = self.job_path(job_id)?;
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|e| AppError::io(e.to_string()))?;
        }
        Ok(())
    }

    /// Stable cache directory for extracted images of one file (keyed by path + mtime).
    pub fn inspector_cache(&self, path: &Path, mtime: SystemTime) -> Result<PathBuf, AppError> {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        mtime.hash(&mut hasher);
        let key = format!("{:016x}", hasher.finish());
        let dir = self.root.path().join("inspector-cache").join(key);
        std::fs::create_dir_all(&dir).map_err(|e| AppError::io(e.to_string()))?;
        Ok(dir)
    }
}
