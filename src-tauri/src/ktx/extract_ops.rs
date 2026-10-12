//! Shared extract / preview helpers for encode, inspector, and export.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ktx2::Reader;
use tauri::AppHandle;
use tokio::sync::Semaphore;

use crate::blocking::run_blocking;
use crate::error::AppError;
use crate::image::classify::{classify, FileKind};
use crate::image::export::exr_to_preview_png;
use crate::ktx::extract_args::{
    build_extract_args, ktx2ktx2_args, transcode_target_for, unpacked_channels_from_dfd, Dims,
    TranscodeTarget,
};
use crate::ktx::info::{self, preview_kind_from_vk_format};
use crate::ktx::info_types::{ImageSelection, InspectKind, InspectReport, PreviewKind};
use crate::ktx::sidecar::{self, CancellationHandle, Sidecar};

/// Global cap on concurrent `ktx extract` processes.
/// Disk-cached PNG reads skip this path; only real sidecar work takes a slot.
/// Sized to ~80% of logical cores (see [`extract_slots_for_cores`]) so mip-warm
/// can use the machine without a free-for-all scrub stampede.
static EXTRACT_SLOTS: OnceLock<Semaphore> = OnceLock::new();

/// Hard ceiling so a 64+/128-core host cannot spawn dozens of UASTC/Zstd workers.
const EXTRACT_SLOTS_MAX: usize = 16;

/// ~80% of `cores`, at least 1, at most [`EXTRACT_SLOTS_MAX`].
pub fn extract_slots_for_cores(cores: usize) -> usize {
    let roughly_eighty = cores.saturating_mul(4) / 5;
    roughly_eighty.clamp(1, EXTRACT_SLOTS_MAX)
}

/// Process-wide extract concurrency for this host.
pub fn extract_slot_count() -> usize {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    extract_slots_for_cores(cores)
}

fn extract_slots() -> &'static Semaphore {
    EXTRACT_SLOTS.get_or_init(|| Semaphore::new(extract_slot_count()))
}

#[derive(Debug, Clone)]
pub struct TextureMeta {
    pub dims: Dims,
    pub width: u32,
    pub height: u32,
    pub preview: PreviewKind,
    pub channels: u8,
    pub needs_transcode: bool,
}

async fn read_file(path: &Path) -> Result<Vec<u8>, AppError> {
    tokio::fs::read(path)
        .await
        .map_err(|e| AppError::io(format!("{}: {e}", path.display())))
}

pub async fn read_texture_meta(path: &Path) -> Result<TextureMeta, AppError> {
    let bytes = read_file(path).await?;
    let reader = Reader::new(bytes.as_slice())
        .map_err(|e| AppError::invalid_input(format!("invalid KTX2: {e}")))?;
    let header = reader.header();
    let dims = Dims::from_ktx2_counts(
        header.level_count,
        header.face_count,
        header.layer_count,
        header.pixel_depth,
    );
    let needs_transcode = header.index.sgd_byte_length > 0;
    Ok(TextureMeta {
        dims,
        width: header.pixel_width,
        height: header.pixel_height.max(1),
        preview: PreviewKind::Png,
        channels: 4,
        needs_transcode,
    })
}

pub fn meta_from_inspect(
    vk_format: &str,
    scheme: &str,
    dims: Dims,
    width: u32,
    height: u32,
    channels: u8,
) -> TextureMeta {
    let upper_scheme = scheme.to_ascii_uppercase();
    let upper_fmt = vk_format.to_ascii_uppercase();
    let needs_transcode = upper_scheme.contains("BASIS")
        || upper_fmt.contains("UNDEFINED")
        || vk_format == "VK_FORMAT_UNDEFINED";
    TextureMeta {
        dims,
        width,
        height: height.max(1),
        preview: preview_kind_from_vk_format(vk_format),
        channels: channels.max(1),
        needs_transcode,
    }
}

fn selection_cache_name(sel: &ImageSelection, ext: &str) -> String {
    format!(
        "l{}_f{}_y{}_d{}.{ext}",
        sel.level, sel.face, sel.layer, sel.depth_slice
    )
}

/// Fast path: return PNG bytes when a prior extract already wrote them.
pub async fn cached_preview_png(cache_dir: &Path, selection: &ImageSelection) -> Option<Vec<u8>> {
    let path = cache_dir.join(selection_cache_name(selection, "png"));
    tokio::fs::read(path).await.ok()
}

/// Build extract meta from an already-parsed [`InspectReport`] (no `ktx info` spawn).
pub fn texture_meta_from_report(report: &InspectReport) -> TextureMeta {
    match &report.kind {
        InspectKind::Ktx2 { info } => {
            let dims = Dims::from_ktx2_counts(
                info.header.level_count,
                info.header.face_count,
                info.header.layer_count,
                info.header.pixel_depth,
            );
            let channels = channels_from_ktx2_info(info);
            let mut meta = meta_from_inspect(
                &info.header.vk_format,
                &info.header.supercompression_scheme,
                dims,
                info.header.pixel_width,
                info.header.pixel_height,
                channels,
            );
            meta.preview = report.preview.clone();
            meta
        }
        InspectKind::Ktx1 { header, .. } => TextureMeta {
            dims: Dims::from_ktx2_counts(
                header.number_of_mipmap_levels.max(1),
                header.number_of_faces.max(1),
                header.number_of_array_elements,
                header.pixel_depth,
            ),
            width: header.pixel_width,
            height: header.pixel_height.max(1),
            preview: report.preview.clone(),
            channels: 4,
            needs_transcode: false,
        },
    }
}

fn channels_from_ktx2_info(info: &crate::ktx::info_types::Ktx2Info) -> u8 {
    info.data_format_descriptor
        .blocks
        .first()
        .map(|b| {
            unpacked_channels_from_dfd(
                &b.color_model,
                b.samples.iter().map(|s| s.channel_type.as_str()),
            )
        })
        .unwrap_or(4)
}

/// KTX2 path for `ktx extract`, which reads no KTX1. KTX1 is converted once into
/// `cache_dir` (keyed by source path + mtime) with the bundled `ktx2ktx2`.
pub async fn ensure_ktx2(
    app: &AppHandle,
    path: &Path,
    cache_dir: &Path,
) -> Result<PathBuf, AppError> {
    match classify(path) {
        FileKind::Ktx2 => Ok(path.to_path_buf()),
        FileKind::Ktx1 => {
            let out = cache_dir.join("converted.ktx2");
            if tokio::fs::try_exists(&out).await.unwrap_or(false) {
                return Ok(out);
            }
            // Inspector tiles request in parallel: convert to a private name, then
            // rename over `out` so no reader sees a half-written file.
            let partial = cache_dir.join(format!("converting-{}.ktx2", uuid::Uuid::new_v4()));
            sidecar::run(
                app,
                Sidecar::Ktx2Ktx2,
                ktx2ktx2_args(path, &partial),
                |_| {},
                CancellationHandle::new(),
            )
            .await?;
            if let Err(e) = tokio::fs::rename(&partial, &out).await {
                // A parallel request may have won (and Windows refuses to replace an
                // open file); its copy is just as good.
                let _ = tokio::fs::remove_file(&partial).await;
                if !tokio::fs::try_exists(&out).await.unwrap_or(false) {
                    return Err(AppError::io(format!("store converted KTX1: {e}")));
                }
            }
            Ok(out)
        }
        _ => Err(AppError::invalid_input("expected a KTX file")),
    }
}

pub async fn run_extract(
    app: &AppHandle,
    input: &Path,
    output: &Path,
    selection: &ImageSelection,
    dims: Dims,
    transcode: Option<TranscodeTarget>,
    cancel: CancellationHandle,
) -> Result<(), AppError> {
    let _permit = extract_slots()
        .acquire()
        .await
        .map_err(|_| AppError::io("extract slot closed"))?;
    if let Some(parent) = output.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| AppError::io(e.to_string()))?;
    }
    let args = build_extract_args(input, output, selection, dims, transcode);
    sidecar::run(app, Sidecar::Ktx, args, |_| {}, cancel).await?;
    Ok(())
}

/// Tone-map an extracted EXR off the async workers and cache the PNG.
async fn tonemap_to_cache(exr: PathBuf, png_cached: &Path) -> Result<Vec<u8>, AppError> {
    let bytes = run_blocking(move || exr_to_preview_png(&exr)).await?;
    tokio::fs::write(png_cached, &bytes)
        .await
        .map_err(|e| AppError::io(e.to_string()))?;
    Ok(bytes)
}

/// Extract one subresource to PNG bytes (tone-mapped when the tool writes EXR).
pub async fn extract_preview_png(
    app: &AppHandle,
    ktx_path: &Path,
    selection: &ImageSelection,
    meta: &TextureMeta,
    cache_dir: &Path,
    cancel: CancellationHandle,
) -> Result<(Vec<u8>, PreviewKind), AppError> {
    if let PreviewKind::Unavailable { reason } = &meta.preview {
        return Err(AppError::invalid_input(reason.clone()));
    }

    let hdr = matches!(meta.preview, PreviewKind::TonemappedFromExr);
    let png_cached = cache_dir.join(selection_cache_name(selection, "png"));
    if let Some(bytes) = cached_preview_png(cache_dir, selection).await {
        return Ok((bytes, meta.preview.clone()));
    }

    let try_extract = |transcode: Option<TranscodeTarget>, ext: &str| {
        let path = cache_dir.join(selection_cache_name(selection, ext));
        (path, transcode)
    };

    let (extract_path, preview_kind) = if hdr {
        let (exr_path, _) = try_extract(None, "exr");
        run_extract(app, ktx_path, &exr_path, selection, meta.dims, None, cancel).await?;
        let bytes = tonemap_to_cache(exr_path, &png_cached).await?;
        return Ok((bytes, PreviewKind::TonemappedFromExr));
    } else {
        let transcode = if meta.needs_transcode {
            Some(transcode_target_for(meta.channels, false))
        } else {
            None
        };
        let (png_path, t) = try_extract(transcode, "png");
        match run_extract(
            app,
            ktx_path,
            &png_path,
            selection,
            meta.dims,
            t,
            cancel.clone(),
        )
        .await
        {
            Ok(()) => (png_path, PreviewKind::Png),
            Err(first_err) if !meta.needs_transcode => {
                let t = Some(transcode_target_for(meta.channels, false));
                let (retry_path, _) = try_extract(t, "png");
                run_extract(app, ktx_path, &retry_path, selection, meta.dims, t, cancel)
                    .await
                    .map_err(|_| first_err)?;
                (retry_path, PreviewKind::Png)
            }
            Err(e) => return Err(e),
        }
    };

    // If the tool wrote EXR despite an LDR request, tone-map.
    if extract_path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("exr"))
        || (!extract_path.is_file()
            && cache_dir
                .join(selection_cache_name(selection, "exr"))
                .is_file())
    {
        let exr = if extract_path.is_file() {
            extract_path
        } else {
            cache_dir.join(selection_cache_name(selection, "exr"))
        };
        let bytes = tonemap_to_cache(exr, &png_cached).await?;
        return Ok((bytes, PreviewKind::TonemappedFromExr));
    }

    let bytes = read_file(&extract_path).await?;
    if extract_path != png_cached {
        let _ = tokio::fs::copy(&extract_path, &png_cached).await;
    }
    Ok((bytes, preview_kind))
}

pub async fn texture_meta_via_info(app: &AppHandle, path: &Path) -> Result<TextureMeta, AppError> {
    let report = info::inspect(app, path).await?;
    Ok(texture_meta_from_report(&report))
}

#[cfg(test)]
mod extract_slot_tests {
    use super::{extract_slot_count, extract_slots, extract_slots_for_cores};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn extract_slots_for_cores_targets_eighty_percent() {
        assert_eq!(extract_slots_for_cores(1), 1);
        assert_eq!(extract_slots_for_cores(4), 3);
        assert_eq!(extract_slots_for_cores(8), 6);
        assert_eq!(extract_slots_for_cores(10), 8);
        assert_eq!(extract_slots_for_cores(16), 12);
        assert_eq!(extract_slots_for_cores(32), 16);
        assert_eq!(extract_slots_for_cores(128), 16);
    }

    #[tokio::test]
    async fn extract_slots_cap_concurrent_acquirers() {
        let slots = extract_slot_count();
        let concurrent = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));

        let mut joins = Vec::new();
        for _ in 0..(slots * 2 + 2) {
            let concurrent = Arc::clone(&concurrent);
            let peak = Arc::clone(&peak);
            joins.push(tokio::spawn(async move {
                let _permit = extract_slots().acquire().await.expect("slot");
                let now = concurrent.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(now, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(15)).await;
                concurrent.fetch_sub(1, Ordering::SeqCst);
            }));
        }

        for join in joins {
            join.await.expect("join");
        }

        assert_eq!(peak.load(Ordering::SeqCst), slots);
    }
}
