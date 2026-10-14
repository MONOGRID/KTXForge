//! [`ResolvedOptions`] → `ktx create` (and `ktx encode`) argv (v4.4.2).
//!
//! Emits every set field as-is. Which fields may be set is decided once, in
//! [`crate::ktx::rules::resolve_for_encode`]; this module holds no option rules.

use std::path::{Path, PathBuf};

use crate::ktx::options::{CreateOptions, EncodeCodec, Supercompression};
use crate::ktx::rules::ResolvedOptions;

fn fmt_f32(v: f32) -> String {
    // Rust's default float formatting always uses `.` as the decimal separator.
    format!("{v}")
}

/// Whether an animated output is BasisLZ-encoded in a second step, after
/// `KTXanimData` exists. libktx encodes and transcodes an ETC1S array that
/// carries the key as video (P-frames), so a plain array payload stops
/// transcoding once the key is added afterwards.
pub fn encodes_as_video(resolved: &ResolvedOptions, animated: bool) -> bool {
    animated && resolved.options().encode == Some(EncodeCodec::BasisLz)
}

/// Build argv after the `create` subcommand: flags, then inputs, then output.
pub fn build_create_args(
    resolved: &ResolvedOptions,
    inputs: &[PathBuf],
    output: &Path,
) -> Vec<String> {
    let opts = resolved.options();
    let mut args = create_flags(opts);
    push_codec_flags(&mut args, opts, "--encode");
    push_paths(&mut args, inputs, output);
    args
}

/// `ktx create` argv for the uncompressed frames of [`encodes_as_video`]:
/// every flag except the codec ones, which go to [`build_video_encode_args`].
pub fn build_frames_create_args(
    resolved: &ResolvedOptions,
    inputs: &[PathBuf],
    output: &Path,
) -> Vec<String> {
    let mut args = create_flags(resolved.options());
    push_paths(&mut args, inputs, output);
    args
}

/// Argv after the `encode` subcommand that BasisLZ-encodes the frames file.
pub fn build_video_encode_args(
    resolved: &ResolvedOptions,
    input: &Path,
    output: &Path,
) -> Vec<String> {
    let mut args = Vec::new();
    push_codec_flags(&mut args, resolved.options(), "--codec");
    push_paths(&mut args, &[input.to_path_buf()], output);
    args
}

fn push_paths(args: &mut Vec<String>, inputs: &[PathBuf], output: &Path) {
    for input in inputs {
        args.push(input.to_string_lossy().into_owned());
    }
    args.push(output.to_string_lossy().into_owned());
}

/// Format, container, mipmap and color flags: what `ktx create` alone accepts.
fn create_flags(opts: &CreateOptions) -> Vec<String> {
    let mut args = Vec::new();
    push_flag_value(&mut args, "--format", opts.format.cli());

    let c = &opts.container;
    if c.one_d {
        args.push("--1d".to_owned());
    }
    if c.cubemap {
        args.push("--cubemap".to_owned());
    }
    if let Some(layers) = c.layers {
        push_flag_value(&mut args, "--layers", &layers.to_string());
    }
    if let Some(depth) = c.depth {
        push_flag_value(&mut args, "--depth", &depth.to_string());
    }
    if let Some(levels) = c.levels {
        push_flag_value(&mut args, "--levels", &levels.to_string());
    }
    if c.generate_mipmap {
        args.push("--generate-mipmap".to_owned());
    }
    if c.runtime_mipmap {
        args.push("--runtime-mipmap".to_owned());
    }
    if let Some(width) = c.width {
        push_flag_value(&mut args, "--width", &width.to_string());
    }
    if let Some(height) = c.height {
        push_flag_value(&mut args, "--height", &height.to_string());
    }
    if let Some(scale) = c.scale {
        push_flag_value(&mut args, "--scale", &fmt_f32(scale));
    }
    if c.raw {
        args.push("--raw".to_owned());
    }

    let f = &opts.filter;
    if let Some(filter) = f.mipmap_filter {
        push_flag_value(&mut args, "--mipmap-filter", filter.cli());
    }
    if let Some(scale) = f.mipmap_filter_scale {
        push_flag_value(&mut args, "--mipmap-filter-scale", &fmt_f32(scale));
    }
    if let Some(wrap) = f.mipmap_wrap {
        push_flag_value(&mut args, "--mipmap-wrap", wrap.cli());
    }

    let color = &opts.color;
    if let Some(tf) = color.assign_tf {
        push_flag_value(&mut args, "--assign-tf", tf.cli());
    }
    if let Some(tf) = color.convert_tf {
        push_flag_value(&mut args, "--convert-tf", tf.cli());
    }
    if let Some(p) = color.assign_primaries {
        push_flag_value(&mut args, "--assign-primaries", p.cli());
    }
    if let Some(p) = color.convert_primaries {
        push_flag_value(&mut args, "--convert-primaries", p.cli());
    }
    if let Some(o) = color.assign_texcoord_origin {
        push_flag_value(&mut args, "--assign-texcoord-origin", o.cli());
    }
    if let Some(o) = color.convert_texcoord_origin {
        push_flag_value(&mut args, "--convert-texcoord-origin", o.cli());
    }
    if color.fail_on_color_conversions {
        args.push("--fail-on-color-conversions".to_owned());
    }
    if color.warn_on_color_conversions {
        args.push("--warn-on-color-conversions".to_owned());
    }
    if color.no_warn_on_color_conversions {
        args.push("--no-warn-on-color-conversions".to_owned());
    }
    if color.fail_on_origin_changes {
        args.push("--fail-on-origin-changes".to_owned());
    }
    if color.warn_on_origin_changes {
        args.push("--warn-on-origin-changes".to_owned());
    }
    if let Some(sw) = &color.swizzle {
        push_flag_value(&mut args, "--swizzle", sw.as_str());
    }
    if let Some(sw) = &color.input_swizzle {
        push_flag_value(&mut args, "--input-swizzle", sw.as_str());
    }
    if color.normalize {
        args.push("--normalize".to_owned());
    }
    args
}

/// Codec, codec settings and supercompression: the options `ktx create` and
/// `ktx encode` share. `codec_flag` is `--encode` for create, `--codec` for encode.
fn push_codec_flags(args: &mut Vec<String>, opts: &CreateOptions, codec_flag: &str) {
    if opts.color.normal_mode {
        args.push("--normal-mode".to_owned());
    }
    if let Some(encode) = opts.encode {
        push_flag_value(args, codec_flag, encode.cli());
    }

    let b = &opts.basis_lz;
    if let Some(q) = b.qlevel {
        push_flag_value(args, "--qlevel", &q.to_string());
    }
    if let Some(clevel) = b.clevel {
        push_flag_value(args, "--clevel", &clevel.to_string());
    }
    if let Some(v) = b.max_endpoints {
        push_flag_value(args, "--max-endpoints", &v.to_string());
    }
    if let Some(v) = b.max_selectors {
        push_flag_value(args, "--max-selectors", &v.to_string());
    }
    if let Some(v) = b.endpoint_rdo_threshold {
        push_flag_value(args, "--endpoint-rdo-threshold", &fmt_f32(v));
    }
    if let Some(v) = b.selector_rdo_threshold {
        push_flag_value(args, "--selector-rdo-threshold", &fmt_f32(v));
    }
    if b.no_endpoint_rdo {
        args.push("--no-endpoint-rdo".to_owned());
    }
    if b.no_selector_rdo {
        args.push("--no-selector-rdo".to_owned());
    }

    let u = &opts.uastc_ldr;
    if let Some(q) = u.uastc_quality {
        push_flag_value(args, "--uastc-quality", &q.to_string());
    }
    if u.uastc_rdo {
        args.push("--uastc-rdo".to_owned());
    }
    if let Some(v) = u.uastc_rdo_l {
        push_flag_value(args, "--uastc-rdo-l", &fmt_f32(v));
    }
    if let Some(v) = u.uastc_rdo_d {
        push_flag_value(args, "--uastc-rdo-d", &v.to_string());
    }
    if let Some(v) = u.uastc_rdo_b {
        push_flag_value(args, "--uastc-rdo-b", &fmt_f32(v));
    }
    if let Some(v) = u.uastc_rdo_s {
        push_flag_value(args, "--uastc-rdo-s", &fmt_f32(v));
    }
    if u.uastc_rdo_f {
        args.push("--uastc-rdo-f".to_owned());
    }
    if u.uastc_rdo_m {
        args.push("--uastc-rdo-m".to_owned());
    }

    let a = &opts.astc;
    if let Some(q) = a.astc_quality {
        push_flag_value(args, "--astc-quality", q.cli());
    }
    if a.astc_perceptual {
        args.push("--astc-perceptual".to_owned());
    }

    let e = &opts.encoder;
    if let Some(threads) = e.threads {
        push_flag_value(args, "--threads", &threads.to_string());
    }
    if e.no_sse {
        args.push("--no-sse".to_owned());
    }
    if e.compare_ssim {
        args.push("--compare-ssim".to_owned());
    }
    if e.compare_psnr {
        args.push("--compare-psnr".to_owned());
    }

    match opts.supercompression {
        Supercompression::None => {}
        Supercompression::Zstd { level } => {
            push_flag_value(args, "--zstd", &level.to_string());
        }
        Supercompression::Zlib { level } => {
            push_flag_value(args, "--zlib", &level.to_string());
        }
    }
}

fn push_flag_value(args: &mut Vec<String>, flag: &str, value: &str) {
    args.push(flag.to_owned());
    args.push(value.to_owned());
}
