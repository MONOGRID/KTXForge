use std::path::{Path, PathBuf};

use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

use crate::blocking::run_blocking;
use crate::commands::{EncodeRequest, EncodeResult, SourceInfo};
use crate::dialog;
use crate::error::AppError;
use crate::events::{EncodeProgressPayload, EncodeStage, ENCODE_PROGRESS};
use crate::image::convert::{prepare_input, read_source_info};
use crate::ktx::anim::write_anim_data;
use crate::ktx::create_args::build_create_args;
use crate::ktx::extract_ops::{cached_preview_png, extract_preview_png, read_texture_meta};
use crate::ktx::info_types::{ImageSelection, PreviewKind};
use crate::ktx::input_role::InputRole;
use crate::ktx::metrics::parse_metrics;
use crate::ktx::options::{CreateOptions, EncodeCodec};
use crate::ktx::rules;
use crate::ktx::sidecar::{self, CancellationHandle, Sidecar};
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

    let mut args = vec!["create".to_owned()];
    args.extend(build_create_args(&resolved, &prepared_paths, &output_path));

    emit_stage(app, &request.job_id, EncodeStage::Encoding, None);

    let job_id_for_cb = request.job_id.clone();
    let app_for_cb = app.clone();
    let out = sidecar::run(
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
    .await?;

    if let Some(anim) = request.animation.clone() {
        emit_stage(app, &request.job_id, EncodeStage::WritingAnim, None);
        let path = output_path.clone();
        run_blocking(move || write_anim_data(&path, &anim)).await?;
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
    let (output_path, cache, meta) = state.with_job(&job_id, |job| {
        (
            job.output_path.clone(),
            job.dir.join("preview-cache"),
            job.meta.clone(),
        )
    })?;

    if let Some(bytes) = cached_preview_png(&cache, &selection).await {
        return Ok(tauri::ipc::Response::new(bytes));
    }

    let meta = match meta {
        Some(m) => m,
        None => read_texture_meta(&output_path).await?,
    };

    let (bytes, _) = extract_preview_png(
        &app,
        &output_path,
        &selection,
        &meta,
        &cache,
        CancellationHandle::new(),
    )
    .await?;

    Ok(tauri::ipc::Response::new(bytes))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ktx::options::AnimationOptions;
    use crate::test_support::{fixtures, staged_ktx};
    use std::process::Command;

    /// Run argv from the real resolve → build path through the staged `ktx create`.
    /// Each case keeps a value the UI hides; the tool must accept what we emit.
    #[test]
    fn resolved_args_are_accepted_by_ktx_create() {
        use crate::ktx::options::{
            AstcQuality, EncodeCodec, Supercompression, TexcoordOrigin, VkFormat,
        };

        let ktx = staged_ktx();
        let root = fixtures();
        let rgba = vec![root.join("rgba.png")];
        let frames: Vec<_> = (1..=4)
            .map(|i| root.join(format!("frames/frame_00{i}.png")))
            .collect();
        let faces: Vec<_> = (0..6)
            .map(|i| root.join(format!("cube/face_{i}.png")))
            .collect();
        let hdr = vec![root.join("hdr.exr")];

        let mut ssim_no_encode = CreateOptions::default();
        ssim_no_encode.encoder.compare_ssim = true;
        ssim_no_encode.encoder.compare_psnr = true;

        let mut uastc = CreateOptions {
            encode: Some(EncodeCodec::Uastc),
            supercompression: Supercompression::Zstd { level: 3 },
            ..Default::default()
        };
        uastc.uastc_ldr.uastc_rdo = true;
        uastc.encoder.compare_ssim = true;
        uastc.basis_lz.max_endpoints = Some(16);

        let basis_with_zstd = CreateOptions {
            encode: Some(EncodeCodec::BasisLz),
            supercompression: Supercompression::Zstd { level: 3 },
            ..Default::default()
        };

        let mut astc = CreateOptions {
            format: VkFormat::Astc6x6SrgbBlock,
            ..Default::default()
        };
        astc.astc.astc_quality = Some(AstcQuality::Fast);
        astc.encoder.compare_psnr = true;
        astc.uastc_ldr.uastc_rdo = true;

        let mut mip_and_layers = CreateOptions::default();
        mip_and_layers.container.generate_mipmap = true;
        mip_and_layers.container.runtime_mipmap = true;

        let mut cube = CreateOptions::default();
        cube.container.one_d = true;
        cube.color.assign_texcoord_origin = Some(TexcoordOrigin::BottomLeft);

        let mut depth = CreateOptions::default();
        depth.container.generate_mipmap = true;
        depth.container.levels = Some(3);

        let hdr_with_encode = CreateOptions {
            format: VkFormat::R16G16B16A16Sfloat,
            encode: Some(EncodeCodec::BasisLz),
            ..Default::default()
        };

        let cases = [
            (
                "ssim without encode",
                ssim_no_encode,
                InputRole::Base2D,
                &rgba,
            ),
            ("uastc rdo zstd ssim", uastc, InputRole::Base2D, &rgba),
            (
                "basis-lz keeps zstd",
                basis_with_zstd,
                InputRole::Base2D,
                &rgba,
            ),
            ("astc", astc, InputRole::Base2D, &rgba),
            (
                "layers + mipmaps",
                mip_and_layers,
                InputRole::ArrayLayers,
                &frames,
            ),
            (
                "cubemap + 1d + origin",
                cube,
                InputRole::CubemapFaces,
                &faces,
            ),
            ("depth + mipmaps", depth, InputRole::DepthSlices, &frames),
            ("hdr + encode", hdr_with_encode, InputRole::Base2D, &hdr),
        ];

        let dir = tempfile::tempdir().unwrap();
        for (i, (name, options, role, paths)) in cases.into_iter().enumerate() {
            let sources: Vec<_> = paths.iter().map(|p| read_source_info(p).unwrap()).collect();
            let prepared: Vec<_> = paths
                .iter()
                .map(|p| prepare_input(p, &dir.path().join(format!("in{i}"))).unwrap())
                .collect();
            let resolved = rules::resolve_for_encode(&options, &sources, role)
                .unwrap_or_else(|e| panic!("{name}: {e:?}"));
            let out = dir.path().join(format!("out{i}.ktx2"));
            let mut args = vec!["create".to_owned()];
            args.extend(build_create_args(&resolved, &prepared, &out));
            let output = Command::new(&ktx).args(&args).output().unwrap();
            assert!(
                output.status.success(),
                "{name}: ktx {args:?}\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    fn encode_defaults_and_basis_lz_via_staged_binary() {
        let ktx = staged_ktx();
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
        let ktx = staged_ktx();
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
