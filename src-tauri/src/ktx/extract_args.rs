//! `ktx extract` argv and output naming (v4.4.2 surface).

use std::path::Path;

use crate::ktx::info_types::ImageSelection;

/// Unpacked transcode targets available in v4.4.2 `ktx extract`.
/// Half-float targets are deferred (not in v4.4.2 help).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscodeTarget {
    R8,
    Rg8,
    Rgb8,
    Rgba8,
}

impl TranscodeTarget {
    pub fn cli(self) -> &'static str {
        match self {
            Self::R8 => "r8",
            Self::Rg8 => "rg8",
            Self::Rgb8 => "rgb8",
            Self::Rgba8 => "rgba8",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dims {
    pub levels: u32,
    pub faces: u32,
    pub layers: u32,
    pub depth: u32,
}

impl Dims {
    /// Normalize KTX2 header counts (`layer_count` / `pixel_depth` of 0 → 1).
    pub fn from_ktx2_counts(
        level_count: u32,
        face_count: u32,
        layer_count: u32,
        pixel_depth: u32,
    ) -> Self {
        Self {
            levels: level_count.max(1),
            faces: face_count.max(1),
            layers: if layer_count == 0 { 1 } else { layer_count },
            depth: pixel_depth.max(1),
        }
    }
}

pub fn build_extract_args(
    input: &Path,
    output: &Path,
    selection: &ImageSelection,
    dims: Dims,
    transcode: Option<TranscodeTarget>,
) -> Vec<String> {
    let mut args = vec!["extract".to_owned()];
    if let Some(t) = transcode {
        args.push("--transcode".to_owned());
        args.push(t.cli().to_owned());
    }
    // Explicit `--layer 0` on a non-array texture fails; omit unit dimensions.
    if dims.levels > 1 || selection.level > 0 {
        args.push("--level".to_owned());
        args.push(selection.level.to_string());
    }
    if dims.faces > 1 {
        args.push("--face".to_owned());
        args.push(selection.face.to_string());
    }
    if dims.layers > 1 {
        args.push("--layer".to_owned());
        args.push(selection.layer.to_string());
    }
    if dims.depth > 1 {
        args.push("--depth".to_owned());
        args.push(selection.depth_slice.to_string());
    }
    args.push(input.to_string_lossy().into_owned());
    args.push(output.to_string_lossy().into_owned());
    args
}

/// `ktx2ktx2` argv: KTX1 `input` → KTX2 `output`, replacing any existing file.
pub fn ktx2ktx2_args(input: &Path, output: &Path) -> Vec<String> {
    vec![
        "-f".to_owned(),
        "-o".to_owned(),
        output.to_string_lossy().into_owned(),
        input.to_string_lossy().into_owned(),
    ]
}

/// Choose an unpacked target from channel count. HDR half-float targets deferred on v4.4.2.
pub fn transcode_target_for(channels: u8, _hdr: bool) -> TranscodeTarget {
    match channels {
        0 | 1 => TranscodeTarget::R8,
        2 => TranscodeTarget::Rg8,
        3 => TranscodeTarget::Rgb8,
        _ => TranscodeTarget::Rgba8,
    }
}

/// Unpacked R/G/B/A count for `--transcode`, from a DFD basic block.
///
/// Block-compressed models pack several color channels into one DFD sample
/// (`ETC1S_RGB`, `UASTC_RGBA`). Using `samples.len()` there picks `r8`/`rg8`
/// and drops blue — yellow color previews and solid-red normal maps.
pub fn unpacked_channels_from_dfd<'a>(
    color_model: &str,
    channel_types: impl IntoIterator<Item = &'a str>,
) -> u8 {
    let model = color_model.to_ascii_uppercase();
    let types: Vec<String> = channel_types
        .into_iter()
        .map(|t| t.to_ascii_uppercase())
        .collect();

    if model.contains("ETC1S") {
        let has_rgb = types.iter().any(|t| t.contains("ETC1S_RGB"));
        let has_aaa = types.iter().any(|t| t.contains("ETC1S_AAA"));
        if has_rgb {
            return if has_aaa { 4 } else { 3 };
        }
        // RRR / GGG / etc. — one sample per plane.
        return (types.len().min(4) as u8).max(1);
    }

    if model.contains("UASTC") {
        if types.iter().any(|t| t.contains("RGBA")) {
            return 4;
        }
        if types.iter().any(|t| t.contains("RGB")) {
            return 3;
        }
        return 4;
    }

    // RGBSDA and other per-component models: one sample per channel.
    (types.len().min(4) as u8).max(1)
}

/// `base_level{n}_face{n}_layer{n}_depth{n}.<ext>` with suffixes omitted when dim is 1.
pub fn expected_output_name(
    base: &str,
    level: u32,
    face: u32,
    layer: u32,
    depth: u32,
    dims: Dims,
    ext: &str,
) -> String {
    let mut name = base.to_owned();
    if dims.levels > 1 {
        name.push_str(&format!("_level{level}"));
    }
    if dims.faces > 1 {
        name.push_str(&format!("_face{face}"));
    }
    if dims.layers > 1 {
        name.push_str(&format!("_layer{layer}"));
    }
    if dims.depth > 1 {
        name.push_str(&format!("_depth{depth}"));
    }
    name.push('.');
    name.push_str(ext);
    name
}
