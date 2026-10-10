use std::path::PathBuf;

use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::image::classify::{classify, FileKind};
use crate::ktx::extract_ops::{ensure_ktx2, extract_preview_png, texture_meta_via_info};
use crate::ktx::info;
use crate::ktx::info_types::{ImageSelection, InspectReport};
use crate::ktx::sidecar::CancellationHandle;
use crate::state::AppState;

#[tauri::command]
pub async fn inspect(app: AppHandle, path: String) -> Result<InspectReport, AppError> {
    info::inspect(&app, PathBuf::from(path).as_path()).await
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
    let convert_dir = work.join("ktx1");
    std::fs::create_dir_all(&convert_dir).map_err(|e| AppError::io(e.to_string()))?;

    let ktx2_path = match classify(&path) {
        FileKind::Ktx2 => path.clone(),
        FileKind::Ktx1 => match ensure_ktx2(&path, &convert_dir).await {
            Ok(p) => p,
            Err(_) => {
                // Try extract on the original KTX1; some builds accept it.
                path.clone()
            }
        },
        _ => {
            return Err(AppError::invalid_input(
                "inspector_image expects a KTX file",
            ))
        }
    };

    let meta = match texture_meta_via_info(&app, &ktx2_path).await {
        Ok(m) => m,
        Err(_) => texture_meta_via_info(&app, &path).await?,
    };

    let cache = work.join("images");
    std::fs::create_dir_all(&cache).map_err(|e| AppError::io(e.to_string()))?;

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
    use super::*;
    use crate::ktx::extract_args::{build_extract_args, Dims, ExtractSpec, TranscodeTarget};
    use std::path::Path;
    use std::process::Command;

    fn fixtures() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn staged_ktx() -> Option<PathBuf> {
        let binaries = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
        let exe = binaries.join("ktx-x86_64-pc-windows-msvc.exe");
        if exe.is_file() {
            Some(exe)
        } else {
            None
        }
    }

    fn extract_one(
        ktx: &Path,
        input: &Path,
        out: &Path,
        spec: ExtractSpec,
        dims: Dims,
        transcode: Option<TranscodeTarget>,
    ) {
        let args = build_extract_args(input, out, &spec, dims, transcode);
        let status = Command::new(ktx).args(&args).status().unwrap();
        assert!(status.success(), "extract failed for {}", input.display());
        assert!(out.is_file() || out.with_extension("exr").is_file() || out.is_dir());
    }

    #[test]
    fn inspector_extract_fixtures_via_staged_binary() {
        let Some(ktx) = staged_ktx() else {
            return;
        };
        let root = fixtures();
        let dir = tempfile::tempdir().unwrap();

        extract_one(
            &ktx,
            &root.join("basis_lz.ktx2"),
            &dir.path().join("basis_l0.png"),
            ExtractSpec::One {
                level: 0,
                face: 0,
                layer: 0,
                depth: 0,
            },
            Dims::from_ktx2_counts(1, 1, 0, 0),
            Some(TranscodeTarget::Rgba8),
        );

        // Last mip: probe level count from info isn't needed — try a high level that exists on basis.
        // basis_lz typically has multiple levels; extract level that fails soft if only one.
        let _ = Command::new(&ktx)
            .args(build_extract_args(
                &root.join("basis_lz.ktx2"),
                &dir.path().join("basis_last.png"),
                &ExtractSpec::One {
                    level: 1,
                    face: 0,
                    layer: 0,
                    depth: 0,
                },
                Dims::from_ktx2_counts(2, 1, 0, 0),
                Some(TranscodeTarget::Rgba8),
            ))
            .status();

        extract_one(
            &ktx,
            &root.join("cubemap.ktx2"),
            &dir.path().join("cube_f3.png"),
            ExtractSpec::One {
                level: 0,
                face: 3,
                layer: 0,
                depth: 0,
            },
            Dims::from_ktx2_counts(1, 6, 0, 0),
            None,
        );

        extract_one(
            &ktx,
            &root.join("array4_mips.ktx2"),
            &dir.path().join("arr_l2_m1.png"),
            ExtractSpec::One {
                level: 1,
                face: 0,
                layer: 2,
                depth: 0,
            },
            Dims::from_ktx2_counts(4, 1, 4, 0),
            None,
        );

        extract_one(
            &ktx,
            &root.join("hdr_uastc.ktx2"),
            &dir.path().join("hdr.exr"),
            ExtractSpec::One {
                level: 0,
                face: 0,
                layer: 0,
                depth: 0,
            },
            Dims::from_ktx2_counts(1, 1, 0, 0),
            None,
        );
    }
}
