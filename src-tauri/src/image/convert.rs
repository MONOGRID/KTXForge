//! Input conversion and thumbnails for `ktx create`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};

use crate::commands::{SourceInfo, SourceKind};
use crate::error::AppError;
use crate::image::classify::{classify, FileKind};

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

/// The original path when `ktx create` can read it as is, else a staged copy.
fn passthrough_or_stage(src: &Path, workdir: &Path, ext: &str) -> Result<PathBuf, AppError> {
    if ext == "exr" && exr_is_tiled(src)? {
        let out = staged_path(workdir, src, ext);
        restage_exr_as_scanline(src, &out)?;
        Ok(out)
    } else if basename_unsafe_for_ktx(src) {
        stage_copy(src, workdir, ext)
    } else {
        Ok(src.to_path_buf())
    }
}

/// The Linux v4.4.2 `ktx` segfaults on tiled EXR (any compression) and reads scanline EXR.
/// ZIP over 16-line blocks, written top to bottom.
const SCANLINE_EXR: exr::image::Encoding = exr::image::Encoding::SMALL_LOSSLESS;

fn exr_error(e: exr::error::Error) -> AppError {
    AppError::image(format!("OpenEXR: {e}"))
}

fn exr_is_tiled(src: &Path) -> Result<bool, AppError> {
    let meta = exr::meta::MetaData::read_from_file(src, false).map_err(exr_error)?;
    Ok(meta
        .headers
        .iter()
        .any(|h| matches!(h.blocks, exr::meta::BlockDescription::Tiles(_))))
}

/// Rewrite every layer as scanline, keeping channels, sample types, and attributes.
/// Only the full-resolution level survives; `ktx create` reads nothing else.
fn restage_exr_as_scanline(src: &Path, out: &Path) -> Result<(), AppError> {
    use exr::prelude::WritableImage;

    let mut image = exr::prelude::read_all_flat_layers_from_file(src).map_err(exr_error)?;
    for layer in &mut image.layer_data {
        layer.encoding = SCANLINE_EXR;
    }
    image.write().to_file(out).map_err(exr_error)
}

/// 32-bit float scanline EXR; RGBA only when the source has alpha.
fn write_scanline_exr(img: &DynamicImage, out: &Path) -> Result<(), AppError> {
    use exr::prelude::{Image, Layer, LayerAttributes, SpecificChannels, Vec2, WritableImage};

    let size = (img.width() as usize, img.height() as usize);
    let attributes = LayerAttributes::default();
    if img.color().has_alpha() {
        let rgba = img.to_rgba32f();
        let channels = SpecificChannels::rgba(|Vec2(x, y)| {
            let p = rgba.get_pixel(x as u32, y as u32).0;
            (p[0], p[1], p[2], p[3])
        });
        Image::from_layer(Layer::new(size, attributes, SCANLINE_EXR, channels))
            .write()
            .to_file(out)
    } else {
        let rgb = img.to_rgb32f();
        let channels = SpecificChannels::rgb(|Vec2(x, y)| {
            let p = rgb.get_pixel(x as u32, y as u32).0;
            (p[0], p[1], p[2])
        });
        Image::from_layer(Layer::new(size, attributes, SCANLINE_EXR, channels))
            .write()
            .to_file(out)
    }
    .map_err(exr_error)
}

/// PNG passes through; EXR passes through unless tiled (re-staged as scanline);
/// JPEG/WebP/BMP/GIF/TIFF → PNG; Radiance HDR → 32-bit float scanline EXR.
pub fn prepare_input(src: &Path, workdir: &Path) -> Result<PathBuf, AppError> {
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
            write_scanline_exr(&img, &out)?;
            Ok(out)
        }
        FileKind::Image {
            kind: SourceKind::Ldr,
            needs_conversion: true,
        } => {
            let img = load_for_png(src)?;
            let out = staged_path(workdir, src, "png");
            img.save_with_format(&out, ImageFormat::Png)
                .map_err(|e| AppError::image(e.to_string()))?;
            Ok(out)
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
            crate::image::export::exr_to_preview_png(&prepared)
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
