//! Temp directories per encode job and inspector extract cache.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tempfile::TempDir;

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

    pub fn root(&self) -> &Path {
        self.root.path()
    }

    /// Create (or reuse) a subdirectory for one encode job.
    pub fn job_dir(&self, job_id: &str) -> Result<PathBuf, AppError> {
        let dir = self.root.path().join(job_id);
        std::fs::create_dir_all(&dir).map_err(|e| AppError::io(e.to_string()))?;
        Ok(dir)
    }

    pub fn remove_job(&self, job_id: &str) -> Result<(), AppError> {
        let dir = self.root.path().join(job_id);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_reuses_and_removes_job_dirs() {
        let ws = Workspace::new().unwrap();
        let root = ws.root().to_path_buf();
        assert!(root.exists());

        let a = ws.job_dir("job-a").unwrap();
        assert!(a.is_dir());
        let a2 = ws.job_dir("job-a").unwrap();
        assert_eq!(a, a2);

        let marker = a.join("marker.txt");
        std::fs::write(&marker, b"x").unwrap();
        assert!(marker.is_file());

        ws.remove_job("job-a").unwrap();
        assert!(!a.exists());

        let cache = ws
            .inspector_cache(Path::new("/tmp/sample.ktx2"), SystemTime::UNIX_EPOCH)
            .unwrap();
        assert!(cache.is_dir());
        let cache2 = ws
            .inspector_cache(Path::new("/tmp/sample.ktx2"), SystemTime::UNIX_EPOCH)
            .unwrap();
        assert_eq!(cache, cache2);

        drop(ws);
        assert!(!root.exists());
    }
}
