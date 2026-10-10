//! Run `ktx info` and build [`InspectReport`].

use std::path::Path;

use tauri::AppHandle;

use crate::error::AppError;
use crate::image::classify::{classify, FileKind};
use crate::ktx::info_types::{InspectKind, InspectReport, Ktx2Info, PreviewKind};
use crate::ktx::ktx1;
use crate::ktx::sidecar::{self, CancellationHandle};

/// Inspect a KTX1 or KTX2 file. Non-zero `ktx info` with JSON on stdout still yields a report.
pub async fn inspect(app: &AppHandle, path: &Path) -> Result<InspectReport, AppError> {
    let path_str = path.to_string_lossy().into_owned();
    let file_size = std::fs::metadata(path)
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
                sidecar::run_ktx_collect(app, args, |_| {}, CancellationHandle::new()).await?;
            let mut info = parse_ktx2_info_json(&out.stdout)?;
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
                sidecar::run_ktx_collect(app, args, |_| {}, CancellationHandle::new()).await?;
            let bytes = std::fs::read(path).map_err(|e| AppError::io(e.to_string()))?;
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
    if upper.contains("SFLOAT")
        || upper.contains("UFLOAT")
        || upper.contains("DFLOAT")
        || upper.contains("R16G16B16")
        || upper.contains("R32")
    {
        return PreviewKind::TonemappedFromExr;
    }
    PreviewKind::Png
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn samples() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/ktx-cli/samples")
    }

    #[test]
    fn parses_saved_info_json_sample() {
        let text = std::fs::read_to_string(samples().join("basis_lz.info.json")).unwrap();
        let mut info = parse_ktx2_info_json(&text).unwrap();
        info.normalize_other_keys();
        assert!(info.header.pixel_width > 0);
        let preview = preview_kind_from_vk_format(&info.header.vk_format);
        assert!(matches!(
            preview,
            PreviewKind::Png | PreviewKind::Unavailable { .. } | PreviewKind::TonemappedFromExr
        ));
    }

    #[test]
    fn hdr_format_preview_is_tonemapped() {
        assert!(matches!(
            preview_kind_from_vk_format("R16G16B16A16_SFLOAT"),
            PreviewKind::TonemappedFromExr
        ));
    }

    #[test]
    fn integer_format_preview_unavailable() {
        assert!(matches!(
            preview_kind_from_vk_format("R8_UINT"),
            PreviewKind::Unavailable { .. }
        ));
    }

    #[test]
    #[ignore = "requires AppHandle + staged ktx sidecar"]
    fn inspect_process_call_ignored() {
        // Covered by Wave 4 integration; kept as a marker for the process path.
    }
}
