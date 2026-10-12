use std::path::PathBuf;

use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::ktx::extract_ops::{
    cached_preview_png, ensure_ktx2, extract_preview_png, texture_meta_from_report,
    texture_meta_via_info,
};
use crate::ktx::info;
use crate::ktx::info_types::{ImageSelection, InspectKind, InspectReport};
use crate::ktx::sidecar::CancellationHandle;
use crate::state::AppState;

#[tauri::command]
pub async fn inspect(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<InspectReport, AppError> {
    let path_buf = PathBuf::from(&path);
    let report = info::inspect(&app, path_buf.as_path()).await?;
    // KTX1 meta comes from the converted KTX2 on the first image request.
    if matches!(report.kind, InspectKind::Ktx2 { .. }) {
        state.set_inspector_meta(path_buf, texture_meta_from_report(&report));
    }
    Ok(report)
}

#[tauri::command]
pub async fn inspector_image(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    selection: ImageSelection,
) -> Result<tauri::ipc::Response, AppError> {
    let path = PathBuf::from(path);
    let work = state.inspector_cache(&path)?;
    let cache = work.join("images");

    // Disk cache hit: no sidecar, no info parse.
    if let Some(bytes) = cached_preview_png(&cache, &selection).await {
        return Ok(tauri::ipc::Response::new(bytes));
    }

    let ktx2_path = ensure_ktx2(&app, &path, &work).await?;
    let meta = match state.inspector_meta_for(&path) {
        Some(meta) => meta,
        None => {
            let meta = texture_meta_via_info(&app, &ktx2_path).await?;
            state.set_inspector_meta(path.clone(), meta.clone());
            meta
        }
    };

    let (bytes, _) = extract_preview_png(
        &app,
        &ktx2_path,
        &selection,
        &meta,
        &cache,
        CancellationHandle::new(),
    )
    .await?;

    Ok(tauri::ipc::Response::new(bytes))
}

#[cfg(test)]
mod tests {
    use crate::ktx::extract_args::{build_extract_args, ktx2ktx2_args, Dims, TranscodeTarget};
    use crate::ktx::info_types::ImageSelection;
    use crate::ktx::sidecar::Sidecar;
    use crate::test_support::{fixtures, staged_ktx, staged_sidecar};
    use std::path::Path;
    use std::process::Command;

    fn extract_one(
        input: &Path,
        out: &Path,
        selection: ImageSelection,
        dims: Dims,
        transcode: Option<TranscodeTarget>,
    ) {
        let args = build_extract_args(input, out, &selection, dims, transcode);
        let output = Command::new(staged_ktx()).args(&args).output().unwrap();
        assert!(
            output.status.success(),
            "extract {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(out.is_file(), "missing {}", out.display());
    }

    #[test]
    fn inspector_extract_fixtures_via_staged_binary() {
        let root = fixtures();
        let dir = tempfile::tempdir().unwrap();

        extract_one(
            &root.join("basis_lz.ktx2"),
            &dir.path().join("basis_l0.png"),
            ImageSelection::default(),
            Dims::from_ktx2_counts(1, 1, 0, 0),
            Some(TranscodeTarget::Rgba8),
        );

        extract_one(
            &root.join("cubemap.ktx2"),
            &dir.path().join("cube_f3.png"),
            ImageSelection {
                face: 3,
                ..Default::default()
            },
            Dims::from_ktx2_counts(1, 6, 0, 0),
            None,
        );

        extract_one(
            &root.join("array4_mips.ktx2"),
            &dir.path().join("arr_l2_m1.png"),
            ImageSelection {
                level: 1,
                layer: 2,
                ..Default::default()
            },
            Dims::from_ktx2_counts(4, 1, 4, 0),
            None,
        );

        extract_one(
            &root.join("hdr_uastc.ktx2"),
            &dir.path().join("hdr.exr"),
            ImageSelection::default(),
            Dims::from_ktx2_counts(1, 1, 0, 0),
            None,
        );
    }

    /// The legacy `.ktx` path: bundled `ktx2ktx2` converts, then `ktx extract` reads it.
    #[test]
    fn ktx1_converts_with_bundled_ktx2ktx2_then_extracts() {
        let dir = tempfile::tempdir().unwrap();
        let converted = dir.path().join("converted.ktx2");
        let output = Command::new(staged_sidecar(Sidecar::Ktx2Ktx2))
            .args(ktx2ktx2_args(&fixtures().join("legacy.ktx"), &converted))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "ktx2ktx2: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        extract_one(
            &converted,
            &dir.path().join("legacy.png"),
            ImageSelection::default(),
            Dims::from_ktx2_counts(1, 1, 0, 0),
            None,
        );
    }
}
