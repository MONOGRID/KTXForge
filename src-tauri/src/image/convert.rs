//! Input conversion and thumbnails for `ktx create`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};

use crate::commands::{SourceInfo, SourceKind};
use crate::error::AppError;
use crate::image::classify::{classify, FileKind};

#[derive(Debug, Clone)]
pub struct PreparedInput {
    pub path: PathBuf,
    /// True when the original path is used unchanged (PNG / EXR).
    pub passthrough: bool,
}

/// `ktx create` splits input paths on `,` and treats a leading `@` as a response file.
fn basename_unsafe_for_ktx(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    name.contains(',') || name.starts_with('@')
}

/// Stable workdir name free of characters that break `ktx create` argv parsing.
fn staged_path(workdir: &Path, src: &Path, ext: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    src.hash(&mut hasher);
    workdir.join(format!("input_{:016x}.{ext}", hasher.finish()))
}

fn stage_copy(src: &Path, workdir: &Path, ext: &str) -> Result<PathBuf, AppError> {
    let out = staged_path(workdir, src, ext);
    std::fs::copy(src, &out).map_err(|e| AppError::io(e.to_string()))?;
    Ok(out)
}

fn passthrough_or_stage(src: &Path, workdir: &Path, ext: &str) -> Result<PreparedInput, AppError> {
    if basename_unsafe_for_ktx(src) {
        Ok(PreparedInput {
            path: stage_copy(src, workdir, ext)?,
            passthrough: false,
        })
    } else {
        Ok(PreparedInput {
            path: src.to_path_buf(),
            passthrough: true,
        })
    }
}

/// PNG and EXR pass through; JPEG/WebP/BMP/GIF/TIFF → PNG; Radiance HDR → half-float EXR.
pub fn prepare_input(src: &Path, workdir: &Path) -> Result<PreparedInput, AppError> {
    std::fs::create_dir_all(workdir).map_err(|e| AppError::io(e.to_string()))?;
    match classify(src) {
        FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: false,
        } => passthrough_or_stage(src, workdir, "png"),
        FileKind::Image {
            kind: SourceKind::Hdr,
            needs_conversion: false,
        } => passthrough_or_stage(src, workdir, "exr"),
        FileKind::Image {
            kind: SourceKind::Hdr,
            needs_conversion: true,
        } => {
            let img = image::open(src).map_err(|e| AppError::image(e.to_string()))?;
            let out = staged_path(workdir, src, "exr");
            img.save_with_format(&out, ImageFormat::OpenExr)
                .map_err(|e| AppError::image(e.to_string()))?;
            Ok(PreparedInput {
                path: out,
                passthrough: false,
            })
        }
        FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: true,
        } => {
            let img = load_for_png(src)?;
            let out = staged_path(workdir, src, "png");
            img.save_with_format(&out, ImageFormat::Png)
                .map_err(|e| AppError::image(e.to_string()))?;
            Ok(PreparedInput {
                path: out,
                passthrough: false,
            })
        }
        FileKind::Ktx1 | FileKind::Ktx2 => Err(AppError::invalid_input(
            "KTX files are not valid create inputs; open them in the inspector",
        )),
        FileKind::Unsupported => Err(AppError::unsupported_file(format!(
            "unsupported input: {}",
            src.display()
        ))),
    }
}

fn load_for_png(src: &Path) -> Result<DynamicImage, AppError> {
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if ext == "gif" {
        let file = std::fs::File::open(src).map_err(|e| AppError::io(e.to_string()))?;
        let reader = std::io::BufReader::new(file);
        let decoder = image::codecs::gif::GifDecoder::new(reader)
            .map_err(|e| AppError::image(e.to_string()))?;
        use image::AnimationDecoder;
        let mut frames = decoder.into_frames();
        let frame = frames
            .next()
            .ok_or_else(|| AppError::image("GIF has no frames"))?
            .map_err(|e| AppError::image(e.to_string()))?;
        return Ok(DynamicImage::ImageRgba8(frame.into_buffer()));
    }
    image::open(src).map_err(|e| AppError::image(e.to_string()))
}

pub fn read_source_info(path: &Path) -> Result<SourceInfo, AppError> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_owned();
    match classify(path) {
        FileKind::Image {
            kind,
            needs_conversion,
        } => {
            let (width, height) =
                image::image_dimensions(path).map_err(|e| AppError::image(e.to_string()))?;
            Ok(SourceInfo {
                path: path.to_string_lossy().into_owned(),
                name,
                kind,
                width,
                height,
                needs_conversion,
            })
        }
        FileKind::Ktx1 | FileKind::Ktx2 => Err(AppError::invalid_input(
            "KTX files are inspected, not used as encode sources",
        )),
        FileKind::Unsupported => Err(AppError::unsupported_file(format!(
            "unsupported source: {}",
            path.display()
        ))),
    }
}

pub fn thumbnail_png(path: &Path, max: u32) -> Result<Vec<u8>, AppError> {
    let max = max.max(1);
    let img = image::open(path).map_err(|e| AppError::image(e.to_string()))?;
    let thumb = img.resize(max, max, FilterType::Triangle);
    encode_png(&thumb)
}

/// Full-resolution PNG suitable for the encoder compare wipe (not the 128px list thumb).
pub fn source_preview_png(path: &Path) -> Result<Vec<u8>, AppError> {
    match classify(path) {
        FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: false,
        } => crate::image::export::png_bytes(path),
        FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: true,
        } => encode_png(&load_for_png(path)?),
        FileKind::Image {
            kind: SourceKind::Hdr,
            needs_conversion: false,
        } => crate::image::export::exr_to_preview_png(path),
        FileKind::Image {
            kind: SourceKind::Hdr,
            needs_conversion: true,
        } => {
            let dir = tempfile::tempdir().map_err(|e| AppError::io(e.to_string()))?;
            let prepared = prepare_input(path, dir.path())?;
            crate::image::export::exr_to_preview_png(&prepared.path)
        }
        FileKind::Ktx1 | FileKind::Ktx2 => Err(AppError::invalid_input(
            "KTX files are inspected, not used as encode sources",
        )),
        FileKind::Unsupported => Err(AppError::unsupported_file(format!(
            "unsupported source: {}",
            path.display()
        ))),
    }
}

fn encode_png(img: &DynamicImage) -> Result<Vec<u8>, AppError> {
    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    img.write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| AppError::image(e.to_string()))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;

    fn fixtures() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    #[test]
    fn converts_jpeg_to_png_same_size() {
        let root = fixtures();
        let dir = tempfile::tempdir().unwrap();
        let prepared = prepare_input(&root.join("rgb.jpg"), dir.path()).unwrap();
        assert!(!prepared.passthrough);
        assert_eq!(
            prepared.path.extension().and_then(|e| e.to_str()),
            Some("png")
        );
        let name = prepared
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        assert!(
            !name.contains(','),
            "staged name must be safe for ktx create: {name}"
        );
        let (w, h) = image::image_dimensions(&prepared.path).unwrap();
        let (sw, sh) = image::image_dimensions(root.join("rgb.jpg")).unwrap();
        assert_eq!((w, h), (sw, sh));
    }

    #[test]
    fn exr_passthrough_byte_identical() {
        let root = fixtures();
        let dir = tempfile::tempdir().unwrap();
        let src = root.join("hdr.exr");
        let prepared = prepare_input(&src, dir.path()).unwrap();
        assert!(prepared.passthrough);
        assert_eq!(prepared.path, src);
        let a = std::fs::read(&src).unwrap();
        let b = std::fs::read(&prepared.path).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn stages_comma_in_basename_for_passthrough_png() {
        let root = fixtures();
        let dir = tempfile::tempdir().unwrap();
        let src = dir
            .path()
            .join("snapshot_camera_cucina_nfc31_irwg_06_08_2026, 18_33_22.png");
        std::fs::copy(root.join("rgba.png"), &src).unwrap();
        let prepared = prepare_input(&src, dir.path()).unwrap();
        assert!(!prepared.passthrough);
        assert!(prepared.path.exists());
        let name = prepared
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        assert!(!name.contains(','));
        assert!(!basename_unsafe_for_ktx(&prepared.path));
    }

    #[test]
    fn stages_comma_in_basename_for_jpeg_convert() {
        let root = fixtures();
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("foo, bar.jpg");
        std::fs::copy(root.join("rgb.jpg"), &src).unwrap();
        let prepared = prepare_input(&src, dir.path()).unwrap();
        assert!(!prepared.passthrough);
        let name = prepared
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        assert!(!name.contains(','));
        assert_eq!(
            prepared.path.extension().and_then(|e| e.to_str()),
            Some("png")
        );
    }

    #[test]
    fn thumbnail_long_side_at_most_max() {
        let root = fixtures();
        let bytes = thumbnail_png(&root.join("rgba.png"), 32).unwrap();
        let img = image::load_from_memory(&bytes).unwrap();
        let (w, h) = img.dimensions();
        assert!(w <= 32 && h <= 32, "{w}x{h}");
        let (sw, sh) = image::image_dimensions(root.join("rgba.png")).unwrap();
        if sw.max(sh) >= 32 {
            assert_eq!(w.max(h), 32);
        }
    }

    #[test]
    fn source_preview_keeps_full_dimensions() {
        let root = fixtures();
        let path = root.join("rgba.png");
        let (sw, sh) = image::image_dimensions(&path).unwrap();
        let bytes = source_preview_png(&path).unwrap();
        let img = image::load_from_memory(&bytes).unwrap();
        assert_eq!(img.dimensions(), (sw, sh));
    }
}
