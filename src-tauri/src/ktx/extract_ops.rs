//! Shared extract / preview helpers for encode, inspector, and export.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::SystemTime;

use ktx2::Reader;
use tauri::AppHandle;
use tokio::sync::Semaphore;

use crate::error::AppError;
use crate::image::classify::{classify, FileKind};
use crate::image::export::{exr_to_preview_png, png_bytes};
use crate::ktx::extract_args::{
    build_extract_args, transcode_target_for, unpacked_channels_from_dfd, Dims, ExtractSpec,
    TranscodeTarget,
};
use crate::ktx::info::{self, preview_kind_from_vk_format};
use crate::ktx::info_types::{ImageSelection, InspectKind, InspectReport, PreviewKind};
use crate::ktx::sidecar::{self, CancellationHandle};
use crate::workspace::Workspace;

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
    pub vk_format: String,
    pub supercompression: String,
    pub preview: PreviewKind,
    pub channels: u8,
    pub needs_transcode: bool,
}

pub fn read_texture_meta(path: &Path) -> Result<TextureMeta, AppError> {
    let bytes = std::fs::read(path).map_err(|e| AppError::io(e.to_string()))?;
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
        vk_format: String::new(),
        supercompression: String::new(),
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
        vk_format: vk_format.to_owned(),
        supercompression: scheme.to_owned(),
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
pub fn cached_preview_png(cache_dir: &Path, selection: &ImageSelection) -> Option<Vec<u8>> {
    let path = cache_dir.join(selection_cache_name(selection, "png"));
    if path.is_file() {
        png_bytes(&path).ok()
    } else {
        None
    }
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
            vk_format: String::new(),
            supercompression: String::new(),
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

/// Ensure a KTX2 path for extract. KTX1 is converted with legacy `ktx2ktx2` when available.
pub async fn ensure_ktx2(path: &Path, workdir: &Path) -> Result<PathBuf, AppError> {
    match classify(path) {
        FileKind::Ktx2 => Ok(path.to_path_buf()),
        FileKind::Ktx1 => convert_ktx1_to_ktx2(path, workdir),
        _ => Err(AppError::invalid_input("expected a KTX file")),
    }
}

fn convert_ktx1_to_ktx2(path: &Path, workdir: &Path) -> Result<PathBuf, AppError> {
    let out = workdir.join("converted.ktx2");
    let tool = find_ktx2ktx2().ok_or_else(|| {
        AppError::invalid_input("ktx2ktx2 not found; cannot convert KTX1 for extract")
    })?;
    let output = Command::new(&tool)
        .arg(path)
        .arg(&out)
        .output()
        .map_err(|e| AppError::io(format!("spawn ktx2ktx2: {e}")))?;
    if !output.status.success() {
        return Err(AppError::ktx(
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    Ok(out)
}

fn find_ktx2ktx2() -> Option<PathBuf> {
    let binaries = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
    for name in ["ktx2ktx2.exe", "ktx2ktx2"] {
        let c = binaries.join(name);
        if c.is_file() {
            return Some(c);
        }
    }
    let cache = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/ktx-cache");
    find_named_under(&cache, "ktx2ktx2.exe").or_else(|| find_named_under(&cache, "ktx2ktx2"))
}

fn find_named_under(root: &Path, name: &str) -> Option<PathBuf> {
    if !root.is_dir() {
        return None;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
                return Some(path);
            }
        }
    }
    None
}

pub async fn run_extract(
    app: &AppHandle,
    input: &Path,
    output: &Path,
    selection: &ExtractSpec,
    dims: Dims,
    transcode: Option<TranscodeTarget>,
    cancel: CancellationHandle,
) -> Result<(), AppError> {
    let _permit = extract_slots()
        .acquire()
        .await
        .map_err(|_| AppError::io("extract slot closed"))?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::io(e.to_string()))?;
    }
    let args = build_extract_args(input, output, selection, dims, transcode);
    sidecar::run_ktx(app, args, |_| {}, cancel).await?;
    Ok(())
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
    if png_cached.is_file() {
        return Ok((png_bytes(&png_cached)?, meta.preview.clone()));
    }

    let spec = ExtractSpec::One {
        level: selection.level,
        face: selection.face,
        layer: selection.layer,
        depth: selection.depth_slice,
    };

    let try_extract = |transcode: Option<TranscodeTarget>, ext: &str| {
        let path = cache_dir.join(selection_cache_name(selection, ext));
        (path, transcode)
    };

    let (extract_path, preview_kind) = if hdr {
        let (exr_path, _) = try_extract(None, "exr");
        run_extract(app, ktx_path, &exr_path, &spec, meta.dims, None, cancel).await?;
        let bytes = exr_to_preview_png(&exr_path)?;
        std::fs::write(&png_cached, &bytes).map_err(|e| AppError::io(e.to_string()))?;
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
            &spec,
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
                run_extract(app, ktx_path, &retry_path, &spec, meta.dims, t, cancel)
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
        let bytes = exr_to_preview_png(&exr)?;
        std::fs::write(&png_cached, &bytes).map_err(|e| AppError::io(e.to_string()))?;
        return Ok((bytes, PreviewKind::TonemappedFromExr));
    }

    let bytes = png_bytes(&extract_path)?;
    if extract_path != png_cached {
        let _ = std::fs::copy(&extract_path, &png_cached);
    }
    Ok((bytes, preview_kind))
}

pub fn inspector_cache_dir(workspace: &Workspace, path: &Path) -> Result<PathBuf, AppError> {
    let mtime = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    workspace.inspector_cache(path, mtime)
}

pub async fn texture_meta_via_info(app: &AppHandle, path: &Path) -> Result<TextureMeta, AppError> {
    let report = info::inspect(app, path).await?;
    match report.kind {
        InspectKind::Ktx2 { info } => {
            let dims = Dims::from_ktx2_counts(
                info.header.level_count,
                info.header.face_count,
                info.header.layer_count,
                info.header.pixel_depth,
            );
            Ok(meta_from_inspect(
                &info.header.vk_format,
                &info.header.supercompression_scheme,
                dims,
                info.header.pixel_width,
                info.header.pixel_height,
                channels_from_ktx2_info(&info),
            ))
        }
        InspectKind::Ktx1 { header, .. } => {
            let dims = Dims::from_ktx2_counts(
                header.number_of_mipmap_levels.max(1),
                header.number_of_faces.max(1),
                header.number_of_array_elements,
                header.pixel_depth,
            );
            Ok(TextureMeta {
                dims,
                width: header.pixel_width,
                height: header.pixel_height.max(1),
                vk_format: String::new(),
                supercompression: String::new(),
                preview: PreviewKind::Png,
                channels: 4,
                needs_transcode: false,
            })
        }
    }
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
