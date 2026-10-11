use std::path::Path;

use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

use crate::commands::{EncodeRequest, EncodeResult};
use crate::error::AppError;
use crate::events::{EncodeProgressPayload, EncodeStage, ENCODE_PROGRESS};
use crate::image::convert::{prepare_input, read_source_info};
use crate::ktx::anim::write_anim_data;
use crate::ktx::create_args::build_create_args;
use crate::ktx::extract_ops::{extract_preview_png, read_texture_meta};
use crate::ktx::info_types::{ImageSelection, PreviewKind};
use crate::ktx::input_role::{self, InputRole};
use crate::ktx::metrics::parse_metrics;
use crate::ktx::options::{CreateOptions, EncodeCodec};
use crate::ktx::sidecar::{self, CancellationHandle};
use crate::state::{AppState, Job};

#[tauri::command]
pub async fn encode(
    app: AppHandle,
    state: State<'_, AppState>,
    request: EncodeRequest,
) -> Result<EncodeResult, AppError> {
    run_encode(&app, &state, request).await
}

pub async fn run_encode(
    app: &AppHandle,
    state: &AppState,
    request: EncodeRequest,
) -> Result<EncodeResult, AppError> {
    if request.sources.is_empty() {
        return Err(AppError::invalid_input(
            "encode requires at least one source",
        ));
    }

    let job_dir = state.job_dir(&request.job_id)?;
    let convert_dir = job_dir.join("inputs");
    std::fs::create_dir_all(&convert_dir).map_err(|e| AppError::io(e.to_string()))?;

    let cancel = CancellationHandle::new();
    {
        let mut jobs = state
            .jobs
            .lock()
            .map_err(|_| AppError::io("jobs lock poisoned"))?;
        jobs.insert(
            request.job_id.clone(),
            Job {
                dir: job_dir.clone(),
                output_path: job_dir.join("output.ktx2"),
                preview_png: job_dir.join("preview.png"),
                preview_kind: PreviewKind::Png,
                cancel: cancel.clone(),
                default_save_name: default_save_name(&request.sources),
            },
        );
    }

    emit_stage(app, &request.job_id, EncodeStage::Converting, None);

    let mut source_infos = Vec::new();
    let mut prepared_paths = Vec::new();
    for src in &request.sources {
        if cancel.is_cancelled() {
            return Err(AppError::cancelled());
        }
        let path = Path::new(src);
        source_infos.push(read_source_info(path)?);
        let prepared = prepare_input(path, &convert_dir)?;
        prepared_paths.push(prepared.path);
    }

    let mut options = request.options.clone();
    input_role::apply_role(&mut options, request.input_role, prepared_paths.len());
    if request.animation.is_some() {
        if request.input_role != InputRole::ArrayLayers {
            return Err(AppError::invalid_input(
                "animation requires Input role = Array layers",
            ));
        }
        if prepared_paths.len() < 2 {
            return Err(AppError::invalid_input(
                "animation requires at least two layer images",
            ));
        }
        options.container.layers = Some(prepared_paths.len() as u32);
    }

    if let Err(errors) = input_role::validate_output(&options, request.input_role, &source_infos) {
        return Err(AppError::invalid_input(errors.join("; ")));
    }

    let output_path = job_dir.join("output.ktx2");
    let mut args = vec!["create".to_owned()];
    args.extend(build_create_args(&options, &prepared_paths, &output_path)?);

    emit_stage(app, &request.job_id, EncodeStage::Encoding, None);

    let job_id_for_cb = request.job_id.clone();
    let app_for_cb = app.clone();
    let out = sidecar::run_ktx(
        app,
        args,
        move |line| {
            let _ = app_for_cb.emit(
                ENCODE_PROGRESS,
                EncodeProgressPayload {
                    job_id: job_id_for_cb.clone(),
                    stage: EncodeStage::Encoding,
                    stderr_line: Some(line),
                },
            );
        },
        cancel.clone(),
    )
    .await?;

    if let Some(anim) = &request.animation {
        emit_stage(app, &request.job_id, EncodeStage::WritingAnim, None);
        write_anim_data(&output_path, anim)?;
    }

    emit_stage(app, &request.job_id, EncodeStage::Previewing, None);

    let meta = read_texture_meta(&output_path)?;
    let preview_kind = preview_kind_for_options(&options, &meta.preview);
    let mut meta = meta;
    meta.preview = preview_kind.clone();
    meta.needs_transcode = options.encode.is_some() || meta.needs_transcode;

    let selection = ImageSelection {
        level: 0,
        face: 0,
        layer: 0,
        depth_slice: 0,
    };
    let cache = job_dir.join("preview-cache");
    std::fs::create_dir_all(&cache).map_err(|e| AppError::io(e.to_string()))?;
    let (png_bytes, resolved_preview) = extract_preview_png(
        app,
        &output_path,
        &selection,
        &meta,
        &cache,
        CancellationHandle::new(),
    )
    .await?;
    let preview_png = job_dir.join("preview.png");
    std::fs::write(&preview_png, &png_bytes).map_err(|e| AppError::io(e.to_string()))?;

    let output_bytes = std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let (ssim, psnr) = if options.encoder.compare_ssim || options.encoder.compare_psnr {
        match parse_metrics(&out.stdout) {
            Ok(m) => (m.ssim, m.psnr),
            Err(_) => (None, None),
        }
    } else {
        (None, None)
    };

    let codec = codec_label(&options);
    let warnings = out
        .stderr
        .lines()
        .filter(|l| l.to_ascii_lowercase().contains("warning"))
        .map(|l| l.to_owned())
        .collect();

    {
        let mut jobs = state
            .jobs
            .lock()
            .map_err(|_| AppError::io("jobs lock poisoned"))?;
        if let Some(job) = jobs.get_mut(&request.job_id) {
            job.output_path = output_path;
            job.preview_png = preview_png;
            job.preview_kind = resolved_preview.clone();
        }
    }

    emit_stage(app, &request.job_id, EncodeStage::Done, None);

    Ok(EncodeResult {
        job_id: request.job_id,
        output_bytes,
        width: meta.width,
        height: meta.height,
        levels: meta.dims.levels,
        layers: meta.dims.layers,
        faces: meta.dims.faces,
        codec,
        ssim,
        psnr,
        warnings,
        preview: resolved_preview,
    })
}

fn preview_kind_for_options(options: &CreateOptions, fallback: &PreviewKind) -> PreviewKind {
    let fmt = options.format.cli().to_ascii_uppercase();
    if fmt.contains("SFLOAT") || fmt.contains("UFLOAT") || fmt.contains("DFLOAT") {
        return PreviewKind::TonemappedFromExr;
    }
    fallback.clone()
}

fn codec_label(options: &CreateOptions) -> String {
    match options.encode {
        Some(EncodeCodec::BasisLz) => "basis-lz".to_owned(),
        Some(EncodeCodec::Uastc) => "uastc".to_owned(),
        None => options.format.cli().to_owned(),
    }
}

fn default_save_name(sources: &[String]) -> String {
    sources
        .first()
        .map(|s| {
            Path::new(s)
                .file_stem()
                .and_then(|st| st.to_str())
                .unwrap_or("output")
                .to_owned()
        })
        .unwrap_or_else(|| "output".to_owned())
        + ".ktx2"
}

fn emit_stage(app: &AppHandle, job_id: &str, stage: EncodeStage, stderr_line: Option<String>) {
    let _ = app.emit(
        ENCODE_PROGRESS,
        EncodeProgressPayload {
            job_id: job_id.to_owned(),
            stage,
            stderr_line,
        },
    );
}

#[tauri::command]
pub async fn cancel_encode(state: State<'_, AppState>, job_id: String) -> Result<(), AppError> {
    let jobs = state
        .jobs
        .lock()
        .map_err(|_| AppError::io("jobs lock poisoned"))?;
    if let Some(job) = jobs.get(&job_id) {
        job.cancel.cancel();
        Ok(())
    } else {
        Err(AppError::invalid_input(format!("unknown job: {job_id}")))
    }
}

#[tauri::command]
pub async fn encoded_preview(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<tauri::ipc::Response, AppError> {
    let jobs = state
        .jobs
        .lock()
        .map_err(|_| AppError::io("jobs lock poisoned"))?;
    let job = jobs
        .get(&job_id)
        .ok_or_else(|| AppError::invalid_input(format!("unknown job: {job_id}")))?;
    let bytes = std::fs::read(&job.preview_png).map_err(|e| AppError::io(e.to_string()))?;
    Ok(tauri::ipc::Response::new(bytes))
}

#[tauri::command]
pub async fn save_encoded(
    app: AppHandle,
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<String>, AppError> {
    let (output_path, default_name) = {
        let jobs = state
            .jobs
            .lock()
            .map_err(|_| AppError::io("jobs lock poisoned"))?;
        let job = jobs
            .get(&job_id)
            .ok_or_else(|| AppError::invalid_input(format!("unknown job: {job_id}")))?;
        (job.output_path.clone(), job.default_save_name.clone())
    };

    let picked = app
        .dialog()
        .file()
        .add_filter("KTX2", &["ktx2"])
        .set_file_name(&default_name)
        .blocking_save_file();

    let Some(file) = picked else {
        return Ok(None);
    };
    let dest = file
        .into_path()
        .map_err(|e| AppError::io(format!("dialog path: {e}")))?;
    std::fs::copy(&output_path, &dest).map_err(|e| AppError::io(e.to_string()))?;
    Ok(Some(dest.to_string_lossy().into_owned()))
}

/// Save multiple encoded jobs into a chosen directory.
#[tauri::command]
pub async fn save_encoded_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    job_ids: Vec<String>,
) -> Result<Option<u32>, AppError> {
    if job_ids.is_empty() {
        return Ok(None);
    }

    let picked = app.dialog().file().blocking_pick_folder();
    let Some(folder) = picked else {
        return Ok(None);
    };
    let dir = folder
        .into_path()
        .map_err(|e| AppError::io(format!("dialog path: {e}")))?;

    let mut saved = 0u32;
    for job_id in &job_ids {
        let (output_path, default_name) = {
            let jobs = state
                .jobs
                .lock()
                .map_err(|_| AppError::io("jobs lock poisoned"))?;
            let job = jobs
                .get(job_id)
                .ok_or_else(|| AppError::invalid_input(format!("unknown job: {job_id}")))?;
            (job.output_path.clone(), job.default_save_name.clone())
        };
        let dest = dir.join(&default_name);
        std::fs::copy(&output_path, &dest).map_err(|e| AppError::io(e.to_string()))?;
        saved += 1;
    }
    Ok(Some(saved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ktx::options::AnimationOptions;
    use crate::ktx::options::CreateOptions;
    use std::path::PathBuf;
    use std::process::Command;

    fn fixtures() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn staged_ktx() -> Option<PathBuf> {
        let binaries = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
        let exe = binaries.join("ktx-x86_64-pc-windows-msvc.exe");
        if exe.is_file() {
            return Some(exe);
        }
        std::fs::read_dir(&binaries).ok()?.flatten().find_map(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("ktx-") && (name.ends_with(".exe") || !name.contains('.')) {
                Some(e.path())
            } else {
                None
            }
        })
    }

    #[test]
    fn encode_defaults_and_basis_lz_via_staged_binary() {
        let Some(ktx) = staged_ktx() else {
            return;
        };
        let root = fixtures();
        let dir = tempfile::tempdir().unwrap();
        let out_default = dir.path().join("default.ktx2");
        let status = Command::new(&ktx)
            .args([
                "create",
                "--format",
                "R8G8B8A8_SRGB",
                root.join("rgba.png").to_str().unwrap(),
                out_default.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        assert!(status.success());
        assert!(out_default.is_file());

        let out_basis = dir.path().join("basis.ktx2");
        let status = Command::new(&ktx)
            .args([
                "create",
                "--format",
                "R8G8B8A8_SRGB",
                "--encode",
                "basis-lz",
                root.join("rgba.png").to_str().unwrap(),
                out_basis.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        assert!(status.success());
        assert!(out_basis.is_file());
    }

    #[test]
    fn encode_animation_frames_writes_anim_key() {
        let Some(ktx) = staged_ktx() else {
            return;
        };
        let frames = fixtures().join("frames");
        let inputs: Vec<PathBuf> = (1..=4)
            .map(|i| frames.join(format!("frame_00{i}.png")))
            .collect();
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("anim.ktx2");
        let mut args = vec![
            "create".to_owned(),
            "--format".to_owned(),
            "R8G8B8A8_SRGB".to_owned(),
            "--layers".to_owned(),
            "4".to_owned(),
        ];
        for p in &inputs {
            args.push(p.to_string_lossy().into_owned());
        }
        args.push(out.to_string_lossy().into_owned());
        let status = Command::new(&ktx).args(&args).status().unwrap();
        assert!(status.success(), "create frames failed");

        let anim = AnimationOptions {
            duration: 1,
            timescale: 10,
            loop_count: 0,
        };
        write_anim_data(&out, &anim).unwrap();

        let bytes = std::fs::read(&out).unwrap();
        let reader = ktx2::Reader::new(bytes.as_slice()).unwrap();
        let mut found = false;
        for (key, value) in reader.key_value_data() {
            if key == "KTXanimData" {
                assert_eq!(value.len(), 12);
                found = true;
            }
        }
        assert!(found, "KTXanimData missing");
        let _ = CreateOptions::default();
    }
}
