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
    /// True when the original path is used unchanged (PNG / scanline EXR).
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
    if ext == "exr" && exr_is_tiled(src)? {
        let out = staged_path(workdir, src, ext);
        restage_exr_as_scanline(src, &out)?;
        Ok(PreparedInput {
            path: out,
            passthrough: false,
        })
    } else if basename_unsafe_for_ktx(src) {
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
            write_scanline_exr(&img, &out)?;
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

    fn staged_ktx() -> Option<PathBuf> {
        let binaries = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
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

    /// Run the staged `ktx create` on a prepared HDR input; skip when no binary is staged.
    fn assert_ktx_accepts_hdr(input: &Path, out_dir: &Path) {
        let Some(ktx) = staged_ktx() else {
            return;
        };
        let out = out_dir.join("out.ktx2");
        let output = std::process::Command::new(&ktx)
            .args(["create", "--format", "R16G16B16A16_SFLOAT"])
            .arg(input)
            .arg(&out)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "ktx create {}: {:?}\n{}",
            input.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(out.is_file());
    }

    fn first_pixel_rgba(path: &Path) -> (f32, f32, f32, f32) {
        let image = exr::prelude::read_first_rgba_layer_from_file(
            path,
            |_, _| None,
            |first: &mut Option<(f32, f32, f32, f32)>, _, px: (f32, f32, f32, f32)| {
                first.get_or_insert(px);
            },
        )
        .unwrap();
        image.layer_data.channel_data.pixels.unwrap()
    }

    #[test]
    fn radiance_hdr_converts_to_scanline_exr_ktx_accepts() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("in.hdr");
        image::Rgb32FImage::from_pixel(32, 32, image::Rgb([0.5, 1.5, 2.0]))
            .save(&src)
            .unwrap();

        let prepared = prepare_input(&src, &dir.path().join("work")).unwrap();
        assert!(!prepared.passthrough);
        assert_eq!(
            prepared.path.extension().and_then(|e| e.to_str()),
            Some("exr")
        );
        assert!(!exr_is_tiled(&prepared.path).unwrap());
        let (r, g, b, a) = first_pixel_rgba(&prepared.path);
        assert_eq!((r, g, b, a), (0.5, 1.5, 2.0, 1.0));

        assert_ktx_accepts_hdr(&prepared.path, dir.path());
    }

    #[test]
    fn tiled_exr_restaged_as_scanline_ktx_accepts() {
        use exr::prelude::{
            Encoding, Image, Layer, LayerAttributes, SpecificChannels, WritableImage,
        };

        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("tiled.exr");
        let channels = SpecificChannels::rgba(|_| (0.25_f32, 4.0_f32, 8.0_f32, 0.5_f32));
        // Tiled RLE: the layout the `image` crate writes and the Linux `ktx` crashes on.
        Image::from_layer(Layer::new(
            (32, 32),
            LayerAttributes::default(),
            Encoding::FAST_LOSSLESS,
            channels,
        ))
        .write()
        .to_file(&src)
        .unwrap();
        assert!(exr_is_tiled(&src).unwrap());

        let prepared = prepare_input(&src, &dir.path().join("work")).unwrap();
        assert!(!prepared.passthrough);
        assert_ne!(prepared.path, src);
        assert!(!exr_is_tiled(&prepared.path).unwrap());
        assert_eq!(first_pixel_rgba(&prepared.path), (0.25, 4.0, 8.0, 0.5));

        assert_ktx_accepts_hdr(&prepared.path, dir.path());
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
        assert_ktx_accepts_hdr(&prepared.path, dir.path());
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
