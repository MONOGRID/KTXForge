//! Run `ktx info` and build [`InspectReport`].

use std::path::Path;

use tauri::{AppHandle, Runtime};

use crate::error::AppError;
use crate::image::classify::{classify, FileKind};
use crate::ktx::info_types::{InspectKind, InspectReport, Ktx2Info, PreviewKind};
use crate::ktx::ktx1;
use crate::ktx::sidecar::{self, CancellationHandle, Sidecar};

/// Inspect a KTX1 or KTX2 file. Non-zero `ktx info` with JSON on stdout still yields a report.
pub async fn inspect<R: Runtime>(
    app: &AppHandle<R>,
    path: &Path,
) -> Result<InspectReport, AppError> {
    let path_str = path.to_string_lossy().into_owned();
    let file_size = tokio::fs::metadata(path)
        .await
        .map_err(|e| AppError::io(e.to_string()))?
        .len();

    match classify(path) {
        FileKind::Ktx2 => {
            let args = vec![
                "info".to_owned(),
                "--format".to_owned(),
                "json".to_owned(),
                path_str.clone(),
            ];
            let out =
                sidecar::run_collect(app, Sidecar::Ktx, args, |_| {}, CancellationHandle::new())
                    .await?;
            let mut info = match parse_ktx2_info_json(&out.stdout) {
                Ok(info) => info,
                Err(parse_err) => {
                    return Err(ktx_info_failure(out.code, &out.stderr, parse_err));
                }
            };
            info.normalize_other_keys();
            let preview = preview_kind_from_vk_format(&info.header.vk_format);
            Ok(InspectReport {
                path: path_str,
                file_size,
                kind: InspectKind::Ktx2 {
                    info: Box::new(info),
                },
                preview,
            })
        }
        FileKind::Ktx1 => {
            let args = vec!["info".to_owned(), path_str.clone()];
            let out =
                sidecar::run_collect(app, Sidecar::Ktx, args, |_| {}, CancellationHandle::new())
                    .await?;
            if out.code != 0 && out.stdout.trim().is_empty() {
                return Err(AppError::ktx(out.code, out.stderr));
            }
            let bytes = tokio::fs::read(path)
                .await
                .map_err(|e| AppError::io(e.to_string()))?;
            let header = ktx1::read_header(&bytes)?;
            Ok(InspectReport {
                path: path_str,
                file_size,
                kind: InspectKind::Ktx1 {
                    text: out.stdout,
                    header,
                },
                preview: PreviewKind::Png,
            })
        }
        other => Err(AppError::invalid_input(format!(
            "inspect expects a KTX file, got {other:?}"
        ))),
    }
}

/// Prefer the sidecar exit/stderr when `ktx info` failed; otherwise keep the parse error.
fn ktx_info_failure(code: i32, stderr: &str, parse_err: AppError) -> AppError {
    if code != 0 {
        AppError::ktx(code, stderr)
    } else {
        parse_err
    }
}

/// Parse info JSON without spawning (unit-tested against saved samples).
pub fn parse_ktx2_info_json(stdout: &str) -> Result<Ktx2Info, AppError> {
    let trimmed = stdout.trim();
    // Some tools wrap JSON; take the outermost object.
    let start = trimmed
        .find('{')
        .ok_or_else(|| AppError::invalid_input("ktx info stdout has no JSON object"))?;
    let json = &trimmed[start..];
    serde_json::from_str(json)
        .map_err(|e| AppError::invalid_input(format!("failed to parse ktx info JSON: {e}")))
}

pub fn preview_kind_from_vk_format(vk_format: &str) -> PreviewKind {
    let upper = vk_format.to_ascii_uppercase();
    if upper.contains("SINT") || upper.contains("UINT") {
        return PreviewKind::Unavailable {
            reason: "integer formats cannot be previewed as color images".to_owned(),
        };
    }
    // `ktx extract` writes EXR for float formats only; 16-bit UNORM is a 16-bit PNG.
    if upper.contains("SFLOAT") || upper.contains("UFLOAT") {
        return PreviewKind::TonemappedFromExr;
    }
    PreviewKind::Png
}
