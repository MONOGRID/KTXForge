use std::path::{Path, PathBuf};

use tauri::{AppHandle, Emitter, Runtime, State};
use tauri_plugin_dialog::DialogExt;

use crate::blocking::run_blocking;
use crate::commands::{EncodeRequest, EncodeResult, SourceInfo};
use crate::dialog;
use crate::error::AppError;
use crate::events::{EncodeProgressPayload, EncodeStage, ENCODE_PROGRESS};
use crate::image::convert::{prepare_input, read_source_info};
use crate::ktx::anim::write_anim_data;
use crate::ktx::create_args::{
    build_create_args, build_frames_create_args, build_video_encode_args, encodes_as_video,
};
use crate::ktx::extract_ops::{cached_preview_png, extract_preview_png, read_texture_meta};
use crate::ktx::info_types::{ImageSelection, PreviewKind};
use crate::ktx::input_role::InputRole;
use crate::ktx::metrics::parse_metrics;
use crate::ktx::options::{CreateOptions, EncodeCodec};
use crate::ktx::rules;
use crate::ktx::sidecar::{self, CancellationHandle, KtxOutput, Sidecar};
use crate::state::{AppState, Job};

#[tauri::command]
pub async fn encode(
    app: AppHandle,
    state: State<'_, AppState>,
    request: EncodeRequest,
) -> Result<EncodeResult, AppError> {
    run_encode(&app, &state, request).await
}

pub async fn run_encode<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    request: EncodeRequest,
) -> Result<EncodeResult, AppError> {
    if request.sources.is_empty() {
        return Err(AppError::invalid_input(
            "encode requires at least one source",
        ));
    }
    if request.animation.is_some() {
        if request.input_role != InputRole::ArrayLayers {
            return Err(AppError::invalid_input(
                "animation requires Input role = Array layers",
            ));
        }
        if request.sources.len() < 2 {
            return Err(AppError::invalid_input(
                "animation requires at least two layer images",
            ));
        }
    }

    // A newer encode of an output replaces its previous job and temp files.
    state.discard_output_jobs(std::slice::from_ref(&request.output_id))?;
    let job_dir = state.job_dir(&request.job_id)?;
    let output_path = job_dir.join("output.ktx2");
    let preview_cache = job_dir.join("preview-cache");
    let cancel = CancellationHandle::new();
    state.insert_job(
        request.job_id.clone(),
        Job {
            output_id: request.output_id.clone(),
            dir: job_dir.clone(),
            output_path: output_path.clone(),
            meta: None,
            cancel: cancel.clone(),
            default_save_name: default_save_name(&request.sources),
        },
    )?;

    emit_stage(app, &request.job_id, EncodeStage::Converting, None);

    let (source_infos, prepared_paths) = prepare_sources(
        request.sources.clone(),
        job_dir.join("inputs"),
        cancel.clone(),
    )
    .await?;

    let resolved = rules::resolve_for_encode(&request.options, &source_infos, request.input_role)
        .map_err(|errors| AppError::invalid_input(errors.join("; ")))?;
    let options = resolved.options();

    let video = encodes_as_video(&resolved, request.animation.is_some());
    // Video: uncompressed frames first, then `ktx encode` once `KTXanimData` exists.
    let create_output = if video {
        job_dir.join("frames.ktx2")
    } else {
        output_path.clone()
    };
    let mut args = vec!["create".to_owned()];
    if video {
        args.extend(build_frames_create_args(
            &resolved,
            &prepared_paths,
            &create_output,
        ));
    } else {
        args.extend(build_create_args(
            &resolved,
            &prepared_paths,
            &create_output,
        ));
    }

    emit_stage(app, &request.job_id, EncodeStage::Encoding, None);
    let mut out = run_ktx_logged(app, &request.job_id, args, &cancel).await?;

    if let Some(anim) = request.animation.clone() {
        emit_stage(app, &request.job_id, EncodeStage::WritingAnim, None);
        let path = create_output.clone();
        run_blocking(move || write_anim_data(&path, &anim)).await?;
    }

    if video {
        emit_stage(app, &request.job_id, EncodeStage::Encoding, None);
        let mut args = vec!["encode".to_owned()];
        args.extend(build_video_encode_args(
            &resolved,
            &create_output,
            &output_path,
        ));
        let encoded = run_ktx_logged(app, &request.job_id, args, &cancel).await?;
        out.stderr.push_str(&encoded.stderr);
        out.stdout = encoded.stdout;
        let _ = tokio::fs::remove_file(&create_output).await;
    }

    emit_stage(app, &request.job_id, EncodeStage::Previewing, None);

    let mut meta = read_texture_meta(&output_path).await?;
    meta.preview = preview_kind_for_options(options, &meta.preview);
    meta.needs_transcode = options.encode.is_some() || meta.needs_transcode;

    // Warms the level-0 tile in the preview cache for `encoded_preview`.
    let (_, resolved_preview) = extract_preview_png(
        app,
        &output_path,
        &ImageSelection::default(),
        &meta,
        &preview_cache,
        CancellationHandle::new(),
    )
    .await?;

    let output_bytes = tokio::fs::metadata(&output_path)
        .await
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

    let codec = codec_label(options);
    let warnings = out
        .stderr
        .lines()
        .filter(|l| l.to_ascii_lowercase().contains("warning"))
        .map(|l| l.to_owned())
        .collect();

    let finished = meta.clone();
    state.update_job(&request.job_id, |job| job.meta = Some(finished))?;

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

/// Run one `ktx` step for an encode job, logging argv and forwarding stderr as progress.
async fn run_ktx_logged<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    args: Vec<String>,
    cancel: &CancellationHandle,
) -> Result<KtxOutput, AppError> {
    eprintln!("ktxforge: encode job {job_id} → ktx {}", shell_join(&args));
    let job_id_for_cb = job_id.to_owned();
    let app_for_cb = app.clone();
    sidecar::run(
        app,
        Sidecar::Ktx,
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
    .await
}

/// Read and stage every source on the blocking pool (image decode, EXR rewrite).
async fn prepare_sources(
    sources: Vec<String>,
    convert_dir: PathBuf,
    cancel: CancellationHandle,
) -> Result<(Vec<SourceInfo>, Vec<PathBuf>), AppError> {
    run_blocking(move || {
        let mut infos = Vec::with_capacity(sources.len());
        let mut prepared = Vec::with_capacity(sources.len());
        for src in &sources {
            if cancel.is_cancelled() {
                return Err(AppError::cancelled());
            }
            let path = Path::new(src);
            infos.push(read_source_info(path)?);
            prepared.push(prepare_input(path, &convert_dir)?);
        }
        Ok((infos, prepared))
    })
    .await
}

fn preview_kind_for_options(options: &CreateOptions, fallback: &PreviewKind) -> PreviewKind {
    if options.format.is_hdr_float() {
        return PreviewKind::TonemappedFromExr;
    }
    fallback.clone()
}

/// Quote argv for a copy-pasteable log line (spaces / quotes in paths).
fn shell_join(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if arg.is_empty()
                || arg.contains(|c: char| c.is_whitespace() || c == '"' || c == '\\' || c == '\'')
            {
                let escaped = arg.replace('\\', "\\\\").replace('"', "\\\"");
                format!("\"{escaped}\"")
            } else {
                arg.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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

fn emit_stage<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    stage: EncodeStage,
    stderr_line: Option<String>,
) {
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
    state.with_job(&job_id, |job| job.cancel.cancel())
}

/// Outputs removed from the queue: cancel their encodes and delete their temp files.
#[tauri::command]
pub async fn discard_outputs(
    state: State<'_, AppState>,
    output_ids: Vec<String>,
) -> Result<(), AppError> {
    state.discard_output_jobs(&output_ids)
}

#[tauri::command]
pub async fn encoded_preview(
    app: AppHandle,
    state: State<'_, AppState>,
    job_id: String,
    selection: ImageSelection,
) -> Result<tauri::ipc::Response, AppError> {
    let bytes = encoded_preview_png(&app, &state, &job_id, &selection).await?;
    Ok(tauri::ipc::Response::new(bytes))
}

/// PNG the encoder wipe shows on the encoded side for one subresource.
pub async fn encoded_preview_png<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    job_id: &str,
    selection: &ImageSelection,
) -> Result<Vec<u8>, AppError> {
    let (output_path, cache, meta) = state.with_job(job_id, |job| {
        (
            job.output_path.clone(),
            job.dir.join("preview-cache"),
            job.meta.clone(),
        )
    })?;

    if let Some(bytes) = cached_preview_png(&cache, selection).await {
        return Ok(bytes);
    }

    let meta = match meta {
        Some(m) => m,
        None => read_texture_meta(&output_path).await?,
    };

    let (bytes, _) = extract_preview_png(
        app,
        &output_path,
        selection,
        &meta,
        &cache,
        CancellationHandle::new(),
    )
    .await?;
    Ok(bytes)
}

async fn copy_output(src: &Path, dest: &Path) -> Result<(), AppError> {
    tokio::fs::copy(src, dest)
        .await
        .map(|_| ())
        .map_err(|e| AppError::io(format!("{}: {e}", dest.display())))
}

#[tauri::command]
pub async fn save_encoded(
    app: AppHandle,
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<String>, AppError> {
    let (output_path, default_name) = state.with_job(&job_id, |job| {
        (job.output_path.clone(), job.default_save_name.clone())
    })?;

    let picker = app
        .dialog()
        .file()
        .add_filter("KTX2", &["ktx2"])
        .set_file_name(&default_name);
    let Some(dest) = dialog::save_file(picker).await? else {
        return Ok(None);
    };
    copy_output(&output_path, &dest).await?;
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

    let Some(dir) = dialog::pick_folder(app.dialog().file()).await? else {
        return Ok(None);
    };

    let mut saved = 0u32;
    for job_id in &job_ids {
        let (output_path, default_name) = state.with_job(job_id, |job| {
            (job.output_path.clone(), job.default_save_name.clone())
        })?;
        copy_output(&output_path, &dir.join(&default_name)).await?;
        saved += 1;
    }
    Ok(Some(saved))
}
