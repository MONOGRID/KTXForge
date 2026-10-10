//! `ktx extract` argv and output naming (v4.4.2 surface).

use std::path::Path;

/// Selection for one extract, or every slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractSpec {
    All,
    One {
        level: u32,
        face: u32,
        layer: u32,
        depth: u32,
    },
}

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

pub fn build_extract_args(
    input: &Path,
    output: &Path,
    selection: &ExtractSpec,
    transcode: Option<TranscodeTarget>,
) -> Vec<String> {
    let mut args = vec!["extract".to_owned()];
    if let Some(t) = transcode {
        args.push("--transcode".to_owned());
        args.push(t.cli().to_owned());
    }
    match selection {
        ExtractSpec::All => {
            args.push("--all".to_owned());
        }
        ExtractSpec::One {
            level,
            face,
            layer,
            depth,
        } => {
            args.push("--level".to_owned());
            args.push(level.to_string());
            args.push("--face".to_owned());
            args.push(face.to_string());
            args.push("--layer".to_owned());
            args.push(layer.to_string());
            args.push("--depth".to_owned());
            args.push(depth.to_string());
        }
    }
    args.push(input.to_string_lossy().into_owned());
    args.push(output.to_string_lossy().into_owned());
    args
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn single_image_args() {
        let args = build_extract_args(
            Path::new("in.ktx2"),
            Path::new("out.png"),
            &ExtractSpec::One {
                level: 0,
                face: 0,
                layer: 0,
                depth: 0,
            },
            None,
        );
        assert_eq!(
            args,
            vec![
                "extract", "--level", "0", "--face", "0", "--layer", "0", "--depth", "0",
                "in.ktx2", "out.png",
            ]
        );
    }

    #[test]
    fn all_flag_and_transcode() {
        let args = build_extract_args(
            Path::new("a.ktx2"),
            Path::new("b.png"),
            &ExtractSpec::All,
            Some(TranscodeTarget::Rgba8),
        );
        assert_eq!(
            args,
            vec![
                "extract",
                "--transcode",
                "rgba8",
                "--all",
                "a.ktx2",
                "b.png",
            ]
        );
    }

    #[test]
    fn transcode_target_per_channel_count() {
        assert_eq!(transcode_target_for(1, false), TranscodeTarget::R8);
        assert_eq!(transcode_target_for(2, false), TranscodeTarget::Rg8);
        assert_eq!(transcode_target_for(3, false), TranscodeTarget::Rgb8);
        assert_eq!(transcode_target_for(4, false), TranscodeTarget::Rgba8);
        // HDR half-float deferred on v4.4.2 — still returns 8-bit unpacked targets.
        assert_eq!(transcode_target_for(4, true), TranscodeTarget::Rgba8);
    }

    #[test]
    fn naming_omits_unit_dimensions() {
        let dims = Dims {
            levels: 1,
            faces: 1,
            layers: 1,
            depth: 1,
        };
        assert_eq!(
            expected_output_name("output", 0, 0, 0, 0, dims, "png"),
            "output.png"
        );
        let dims = Dims {
            levels: 4,
            faces: 6,
            layers: 2,
            depth: 3,
        };
        assert_eq!(
            expected_output_name("output", 1, 3, 0, 2, dims, "png"),
            "output_level1_face3_layer0_depth2.png"
        );
        let _ = PathBuf::from(".");
    }
}
