use std::path::{Path, PathBuf};

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::commands::{ExportFormat, ExportRequest, ExportResult, ExportSelection};
use crate::error::AppError;
use crate::image::export::png_to_jpeg;
use crate::ktx::extract_args::{expected_output_name, ExtractSpec};
use crate::ktx::extract_ops::{ensure_ktx2, run_extract, texture_meta_via_info, TextureMeta};
use crate::ktx::info_types::{ImageSelection, PreviewKind};
use crate::ktx::sidecar::CancellationHandle;
use crate::state::AppState;

#[tauri::command]
pub async fn export_images(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ExportRequest,
) -> Result<ExportResult, AppError> {
    let path = PathBuf::from(&request.path);
    let meta = texture_meta_via_info(&app, &path).await?;
    refuse_ldr_for_exr_only(&meta, &request.format)?;

    let multi = !matches!(request.selection, ExportSelection::One { .. });
    let dest = if multi {
        let folder = app.dialog().file().blocking_pick_folder();
        let Some(folder) = folder else {
            return Ok(ExportResult { files: Vec::new() });
        };
        folder
            .into_path()
            .map_err(|e| AppError::io(format!("dialog path: {e}")))?
    } else {
        let filter = match &request.format {
            ExportFormat::Png => ("PNG", vec!["png"]),
            ExportFormat::Jpeg { .. } => ("JPEG", vec!["jpg", "jpeg"]),
            ExportFormat::Exr => ("OpenEXR", vec!["exr"]),
        };
        let default_name = {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("export");
            match &request.format {
                ExportFormat::Png => format!("{stem}.png"),
                ExportFormat::Jpeg { .. } => format!("{stem}.jpg"),
                ExportFormat::Exr => format!("{stem}.exr"),
            }
        };
        let picked = app
            .dialog()
            .file()
            .add_filter(filter.0, &filter.1)
            .add_filter("PNG", &["png"])
            .add_filter("JPEG", &["jpg", "jpeg"])
            .add_filter("OpenEXR", &["exr"])
            .set_file_name(&default_name)
            .blocking_save_file();
        let Some(file) = picked else {
            return Ok(ExportResult { files: Vec::new() });
        };
        file.into_path()
            .map_err(|e| AppError::io(format!("dialog path: {e}")))?
    };

    export_to(
        &app,
        &state,
        &path,
        &request.selection,
        &request.format,
        &dest,
    )
    .await
}

/// Internal export used by the command and by tests (bypasses dialogs).
pub async fn export_to(
    app: &AppHandle,
    state: &AppState,
    path: &Path,
    selection: &ExportSelection,
    format: &ExportFormat,
    dest: &Path,
) -> Result<ExportResult, AppError> {
    let work = state.inspector_cache(path)?;
    let convert_dir = work.join("ktx1");
    std::fs::create_dir_all(&convert_dir).map_err(|e| AppError::io(e.to_string()))?;
    let ktx2 = ensure_ktx2(path, &convert_dir)
        .await
        .unwrap_or_else(|_| path.to_path_buf());

    let meta = texture_meta_via_info(app, &ktx2).await?;
    refuse_ldr_for_exr_only(&meta, format)?;

    let extract_dir = work.join("export");
    std::fs::create_dir_all(&extract_dir).map_err(|e| AppError::io(e.to_string()))?;

    let base = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let mut written = Vec::new();
    match selection {
        ExportSelection::One { selection } => {
            let file = export_one(ExportOne {
                app,
                ktx2: &ktx2,
                meta: &meta,
                selection,
                format,
                extract_dir: &extract_dir,
                dest,
                base,
                dest_is_file: true,
            })
            .await?;
            written.push(file);
        }
        ExportSelection::AllFrames { level } => {
            if dest.is_file() {
                return Err(AppError::invalid_input(
                    "multi-image export requires a folder destination",
                ));
            }
            std::fs::create_dir_all(dest).map_err(|e| AppError::io(e.to_string()))?;
            for layer in 0..meta.dims.layers {
                let sel = ImageSelection {
                    level: *level,
                    face: 0,
                    layer,
                    depth_slice: 0,
                };
                let file = export_one(ExportOne {
                    app,
                    ktx2: &ktx2,
                    meta: &meta,
                    selection: &sel,
                    format,
                    extract_dir: &extract_dir,
                    dest,
                    base,
                    dest_is_file: false,
                })
                .await?;
                written.push(file);
            }
        }
        ExportSelection::All => {
            if dest.is_file() {
                return Err(AppError::invalid_input(
                    "multi-image export requires a folder destination",
                ));
            }
            std::fs::create_dir_all(dest).map_err(|e| AppError::io(e.to_string()))?;
            for level in 0..meta.dims.levels {
                for face in 0..meta.dims.faces {
                    for layer in 0..meta.dims.layers {
                        for depth in 0..meta.dims.depth {
                            let sel = ImageSelection {
                                level,
                                face,
                                layer,
                                depth_slice: depth,
                            };
                            let file = export_one(ExportOne {
                                app,
                                ktx2: &ktx2,
                                meta: &meta,
                                selection: &sel,
                                format,
                                extract_dir: &extract_dir,
                                dest,
                                base,
                                dest_is_file: false,
                            })
                            .await?;
                            written.push(file);
                        }
                    }
                }
            }
        }
    }

    Ok(ExportResult { files: written })
}

fn refuse_ldr_for_exr_only(meta: &TextureMeta, format: &ExportFormat) -> Result<(), AppError> {
    let exr_only = matches!(
        meta.preview,
        PreviewKind::TonemappedFromExr | PreviewKind::Unavailable { .. }
    );
    if exr_only && matches!(format, ExportFormat::Png | ExportFormat::Jpeg { .. }) {
        return Err(AppError::invalid_input(
            "this format cannot be exported as PNG/JPEG; use EXR",
        ));
    }
    Ok(())
}

struct ExportOne<'a> {
    app: &'a AppHandle,
    ktx2: &'a Path,
    meta: &'a TextureMeta,
    selection: &'a ImageSelection,
    format: &'a ExportFormat,
    extract_dir: &'a Path,
    dest: &'a Path,
    base: &'a str,
    dest_is_file: bool,
}

async fn export_one(p: ExportOne<'_>) -> Result<String, AppError> {
    let ExportOne {
        app,
        ktx2,
        meta,
        selection,
        format,
        extract_dir,
        dest,
        base,
        dest_is_file,
    } = p;
    let tool_ext = if matches!(meta.preview, PreviewKind::TonemappedFromExr) {
        "exr"
    } else {
        "png"
    };
    let extract_name = expected_output_name(
        "slice",
        selection.level,
        selection.face,
        selection.layer,
        selection.depth_slice,
        meta.dims,
        tool_ext,
    );
    let extract_path = extract_dir.join(&extract_name);
    let spec = ExtractSpec::One {
        level: selection.level,
        face: selection.face,
        layer: selection.layer,
        depth: selection.depth_slice,
    };
    let transcode = if meta.needs_transcode {
        Some(crate::ktx::extract_args::transcode_target_for(
            meta.channels,
            false,
        ))
    } else {
        None
    };
    run_extract(
        app,
        ktx2,
        &extract_path,
        &spec,
        meta.dims,
        transcode,
        CancellationHandle::new(),
    )
    .await?;

    let out_ext = match format {
        ExportFormat::Png => "png",
        ExportFormat::Jpeg { .. } => "jpg",
        ExportFormat::Exr => "exr",
    };
    let out_name = expected_output_name(
        base,
        selection.level,
        selection.face,
        selection.layer,
        selection.depth_slice,
        meta.dims,
        out_ext,
    );
    let out_path = if dest_is_file {
        dest.to_path_buf()
    } else {
        dest.join(out_name)
    };

    match format {
        ExportFormat::Png => {
            let src = if extract_path.is_file() {
                extract_path
            } else {
                extract_path.with_extension("png")
            };
            std::fs::copy(&src, &out_path).map_err(|e| AppError::io(e.to_string()))?;
        }
        ExportFormat::Jpeg { quality } => {
            let src = if extract_path.is_file() {
                extract_path
            } else {
                extract_path.with_extension("png")
            };
            png_to_jpeg(&src, &out_path, *quality)?;
        }
        ExportFormat::Exr => {
            let src = if extract_path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("exr"))
            {
                extract_path
            } else {
                // Re-extract as EXR when the LDR path produced PNG.
                let exr = extract_dir.join(expected_output_name(
                    "slice",
                    selection.level,
                    selection.face,
                    selection.layer,
                    selection.depth_slice,
                    meta.dims,
                    "exr",
                ));
                if !exr.is_file() {
                    run_extract(
                        app,
                        ktx2,
                        &exr,
                        &spec,
                        meta.dims,
                        None,
                        CancellationHandle::new(),
                    )
                    .await?;
                }
                exr
            };
            std::fs::copy(&src, &out_path).map_err(|e| AppError::io(e.to_string()))?;
        }
    }

    Ok(out_path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ktx::extract_args::{build_extract_args, Dims, TranscodeTarget};
    use std::process::Command;

    fn fixtures() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn staged_ktx() -> Option<PathBuf> {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries/ktx-x86_64-pc-windows-msvc.exe");
        p.is_file().then_some(p)
    }

    #[test]
    fn export_anim_frames_as_jpeg_via_staged_binary() {
        let Some(ktx) = staged_ktx() else {
            return;
        };
        let anim = fixtures().join("anim.ktx2");
        let dir = tempfile::tempdir().unwrap();
        let dims = Dims::from_ktx2_counts(1, 1, 4, 0);
        for layer in 0..4 {
            let png = dir.path().join(format!("frame_layer{layer}.png"));
            let args = build_extract_args(
                &anim,
                &png,
                &ExtractSpec::One {
                    level: 0,
                    face: 0,
                    layer,
                    depth: 0,
                },
                dims,
                None,
            );
            let status = Command::new(&ktx).args(&args).status().unwrap();
            assert!(status.success());
            let jpg = dir.path().join(format!("frame_layer{layer}.jpg"));
            png_to_jpeg(&png, &jpg, 90).unwrap();
            assert!(jpg.is_file());
        }
        let _ = TranscodeTarget::Rgba8;
    }
}
