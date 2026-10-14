//! PNG→JPEG and EXR tone-map for previews/export.

use std::path::Path;

use image::codecs::jpeg::JpegEncoder;
use image::{ColorType, DynamicImage, ImageEncoder, ImageFormat, RgbaImage};

use crate::error::AppError;

/// Alpha composited over white; quality clamped to 1–100.
pub fn png_to_jpeg(png: &Path, out: &Path, quality: u8) -> Result<(), AppError> {
    let quality = quality.clamp(1, 100);
    let img = image::open(png).map_err(|e| AppError::image(e.to_string()))?;
    let rgba = img.to_rgba8();
    let rgb = composite_over_white(&rgba);
    let file = std::fs::File::create(out).map_err(|e| AppError::io(e.to_string()))?;
    let encoder = JpegEncoder::new_with_quality(file, quality);
    encoder
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ColorType::Rgb8.into(),
        )
        .map_err(|e| AppError::image(e.to_string()))?;
    Ok(())
}

fn composite_over_white(rgba: &RgbaImage) -> image::RgbImage {
    let mut rgb = image::RgbImage::new(rgba.width(), rgba.height());
    for (x, y, p) in rgba.enumerate_pixels() {
        let a = f32::from(p[3]) / 255.0;
        let r = (f32::from(p[0]) * a + 255.0 * (1.0 - a)).round() as u8;
        let g = (f32::from(p[1]) * a + 255.0 * (1.0 - a)).round() as u8;
        let b = (f32::from(p[2]) * a + 255.0 * (1.0 - a)).round() as u8;
        rgb.put_pixel(x, y, image::Rgb([r, g, b]));
    }
    rgb
}

/// Reinhard tone map on luminance, then sRGB encode → PNG bytes.
/// Uses the `exr` crate so fixtures with unsorted channel names still load.
pub fn exr_to_preview_png(exr_path: &Path) -> Result<Vec<u8>, AppError> {
    let rgba = load_exr_rgba(exr_path)?;
    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    DynamicImage::ImageRgba8(rgba)
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| AppError::image(e.to_string()))?;
    Ok(buf)
}

fn load_exr_rgba(path: &Path) -> Result<RgbaImage, AppError> {
    let image = exr::prelude::read_first_rgba_layer_from_file(
        path,
        |resolution, _| image::RgbaImage::new(resolution.0 as u32, resolution.1 as u32),
        |pixels: &mut RgbaImage, position, (r, g, b, a): (f32, f32, f32, f32)| {
            let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            let scale = if lum > 0.0 {
                (lum / (1.0 + lum)) / lum
            } else {
                0.0
            };
            let map = |c: f32| linear_to_srgb((c * scale).max(0.0));
            let alpha = a.clamp(0.0, 1.0);
            pixels.put_pixel(
                position.0 as u32,
                position.1 as u32,
                image::Rgba([
                    (map(r) * 255.0).round().clamp(0.0, 255.0) as u8,
                    (map(g) * 255.0).round().clamp(0.0, 255.0) as u8,
                    (map(b) * 255.0).round().clamp(0.0, 255.0) as u8,
                    (alpha * 255.0).round().clamp(0.0, 255.0) as u8,
                ]),
            );
        },
    )
    .map_err(|e| AppError::image(format!("OpenEXR: {e}")))?;

    Ok(image.layer_data.channel_data.pixels)
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

pub fn png_bytes(path: &Path) -> Result<Vec<u8>, AppError> {
    std::fs::read(path).map_err(|e| AppError::io(e.to_string()))
}
