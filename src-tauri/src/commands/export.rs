use std::path::{Path, PathBuf};

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::blocking::run_blocking;
use crate::commands::{ExportFormat, ExportRequest, ExportResult, ExportSelection};
use crate::dialog;
use crate::error::AppError;
use crate::image::export::png_to_jpeg;
use crate::ktx::extract_args::expected_output_name;
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
        let Some(folder) = dialog::pick_folder(app.dialog().file()).await? else {
            return Ok(ExportResult { files: Vec::new() });
        };
        folder
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
        let picker = app
            .dialog()
            .file()
            .add_filter(filter.0, &filter.1)
            .add_filter("PNG", &["png"])
            .add_filter("JPEG", &["jpg", "jpeg"])
            .add_filter("OpenEXR", &["exr"])
            .set_file_name(&default_name);
        let Some(file) = dialog::save_file(picker).await? else {
            return Ok(ExportResult { files: Vec::new() });
        };
        file
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
    let ktx2 = ensure_ktx2(app, path, &work).await?;

    let meta = texture_meta_via_info(app, &ktx2).await?;
    refuse_ldr_for_exr_only(&meta, format)?;

    let extract_dir = work.join("export");

    let base = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let dims = meta.dims;
    let (selections, dest_is_file): (Vec<ImageSelection>, bool) = match selection {
        ExportSelection::One { selection } => (vec![selection.clone()], true),
        ExportSelection::AllFrames { level } => (
            (0..dims.layers)
                .map(|layer| ImageSelection {
                    level: *level,
                    layer,
                    ..Default::default()
                })
                .collect(),
            false,
        ),
        ExportSelection::All => {
            let mut all = Vec::new();
            for level in 0..dims.levels {
                for face in 0..dims.faces {
                    for layer in 0..dims.layers {
                        for depth_slice in 0..dims.depth {
                            all.push(ImageSelection {
                                level,
                                face,
                                layer,
                                depth_slice,
                            });
                        }
                    }
                }
            }
            (all, false)
        }
    };
    if !dest_is_file {
        if dest.is_file() {
            return Err(AppError::invalid_input(
                "multi-image export requires a folder destination",
            ));
        }
        tokio::fs::create_dir_all(dest)
            .await
            .map_err(|e| AppError::io(e.to_string()))?;
    }

    let mut written = Vec::with_capacity(selections.len());
    for selection in &selections {
        let file = export_one(ExportOne {
            app,
            ktx2: &ktx2,
            meta: &meta,
            selection,
            format,
            extract_dir: &extract_dir,
            dest,
            base,
            dest_is_file,
        })
        .await?;
        written.push(file);
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
        selection,
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
            copy_out(&src, &out_path).await?;
        }
        ExportFormat::Jpeg { quality } => {
            let src = if extract_path.is_file() {
                extract_path
            } else {
                extract_path.with_extension("png")
            };
            let (out_path, quality) = (out_path.clone(), *quality);
            run_blocking(move || png_to_jpeg(&src, &out_path, quality)).await?;
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
                        selection,
                        meta.dims,
                        None,
                        CancellationHandle::new(),
                    )
                    .await?;
                }
                exr
            };
            copy_out(&src, &out_path).await?;
        }
    }

    Ok(out_path.to_string_lossy().into_owned())
}

async fn copy_out(src: &Path, dest: &Path) -> Result<(), AppError> {
    tokio::fs::copy(src, dest)
        .await
        .map(|_| ())
        .map_err(|e| AppError::io(format!("{}: {e}", dest.display())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ktx::extract_args::{build_extract_args, Dims};
    use crate::test_support::{fixtures, staged_ktx};
    use std::process::Command;

    #[test]
    fn export_anim_frames_as_jpeg_via_staged_binary() {
        let anim = fixtures().join("anim.ktx2");
        let dir = tempfile::tempdir().unwrap();
        let dims = Dims::from_ktx2_counts(1, 1, 4, 0);
        for layer in 0..4 {
            let png = dir.path().join(format!("frame_layer{layer}.png"));
            let selection = ImageSelection {
                layer,
                ..Default::default()
            };
            let args = build_extract_args(&anim, &png, &selection, dims, None);
            let output = Command::new(staged_ktx()).args(&args).output().unwrap();
            assert!(
                output.status.success(),
                "extract layer {layer}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let jpg = dir.path().join(format!("frame_layer{layer}.jpg"));
            png_to_jpeg(&png, &jpg, 90).unwrap();
            assert!(jpg.is_file());
        }
    }
}
