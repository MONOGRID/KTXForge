use tauri::AppHandle;

use crate::commands::KtxVersion;
use crate::error::AppError;
use crate::ktx::sidecar::{self, CancellationHandle};

const KTX_VERSION_JSON: &str = include_str!("../../../ktx-version.json");

#[tauri::command]
pub async fn ktx_version(app: AppHandle) -> Result<KtxVersion, AppError> {
    let pinned = pinned_version()?;
    let out = sidecar::run_ktx(
        &app,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_version_from_json() {
        let v = pinned_version().unwrap();
        assert_eq!(v, "4.4.2");
    }
}
