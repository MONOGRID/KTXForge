use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// Per-encode job bookkeeping. Child handle filled in Wave 4.
#[derive(Debug, Default)]
pub struct Job {
    pub dir: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    /// Opaque cancel token / process id placeholder until the sidecar runner lands.
    pub child_id: Option<u32>,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub jobs: Mutex<HashMap<String, Job>>,
    pub inspector_file: Mutex<Option<PathBuf>>,
}

/// Reserve a job id shape for Wave 4 (keeps `uuid` linked in the scaffold).
pub fn new_job_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
