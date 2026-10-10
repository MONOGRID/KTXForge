use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// Per-encode job bookkeeping. Filled in by Wave 4.
#[derive(Debug, Default)]
pub struct Job {
    pub dir: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
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
