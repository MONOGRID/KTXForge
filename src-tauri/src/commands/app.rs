use tauri::{AppHandle, Runtime};

use crate::commands::KtxVersion;
use crate::error::AppError;
use crate::ktx::extract_ops::extract_slot_count;
use crate::ktx::sidecar::{self, CancellationHandle, Sidecar};

const KTX_VERSION_JSON: &str = include_str!("../../../ktx-version.json");

/// How many concurrent `ktx extract` processes this host allows (~80% of cores).
#[tauri::command]
pub fn extract_concurrency() -> u32 {
    u32::try_from(extract_slot_count()).unwrap_or(1)
}

#[tauri::command]
pub async fn ktx_version<R: Runtime>(app: AppHandle<R>) -> Result<KtxVersion, AppError> {
    let pinned = pinned_version()?;
    let out = sidecar::run(
        &app,
        Sidecar::Ktx,
        vec!["--version".to_owned()],
        |_| {},
        CancellationHandle::new(),
    )
    .await?;
    let reported = out.stdout.trim().to_owned();
    let reported = if reported.is_empty() {
        out.stderr.trim().to_owned()
    } else {
        reported
    };
    Ok(KtxVersion { pinned, reported })
}

fn pinned_version() -> Result<String, AppError> {
    let value: serde_json::Value = serde_json::from_str(KTX_VERSION_JSON)
        .map_err(|e| AppError::io(format!("ktx-version.json: {e}")))?;
    value
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_owned())
        .ok_or_else(|| AppError::io("ktx-version.json missing version"))
}
