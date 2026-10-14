//! [`ResolvedOptions`] → `ktx create` argv (v4.4.2).
//!
//! Emits every set field as-is. Which fields may be set is decided once, in
//! [`crate::ktx::rules::resolve_for_encode`]; this module holds no option rules.

use std::path::{Path, PathBuf};

use crate::ktx::options::Supercompression;
use crate::ktx::rules::ResolvedOptions;

fn fmt_f32(v: f32) -> String {
    // Rust's default float formatting always uses `.` as the decimal separator.
    format!("{v}")
}

/// Build argv after the `create` subcommand: flags, then inputs, then output.
pub fn build_create_args(
    resolved: &ResolvedOptions,
    inputs: &[PathBuf],
    output: &Path,
) -> Vec<String> {
    let opts = resolved.options();
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
    if color.normal_mode {
        args.push("--normal-mode".to_owned());
    }

    if let Some(encode) = opts.encode {
        push_flag_value(&mut args, "--encode", encode.cli());
    }

    let b = &opts.basis_lz;
    if let Some(q) = b.qlevel {
        push_flag_value(&mut args, "--qlevel", &q.to_string());
    }
    if let Some(clevel) = b.clevel {
        push_flag_value(&mut args, "--clevel", &clevel.to_string());
    }
    if let Some(v) = b.max_endpoints {
        push_flag_value(&mut args, "--max-endpoints", &v.to_string());
    }
    if let Some(v) = b.max_selectors {
        push_flag_value(&mut args, "--max-selectors", &v.to_string());
    }
    if let Some(v) = b.endpoint_rdo_threshold {
        push_flag_value(&mut args, "--endpoint-rdo-threshold", &fmt_f32(v));
    }
    if let Some(v) = b.selector_rdo_threshold {
        push_flag_value(&mut args, "--selector-rdo-threshold", &fmt_f32(v));
    }
    if b.no_endpoint_rdo {
        args.push("--no-endpoint-rdo".to_owned());
    }
    if b.no_selector_rdo {
        args.push("--no-selector-rdo".to_owned());
    }

    let u = &opts.uastc_ldr;
    if let Some(q) = u.uastc_quality {
        push_flag_value(&mut args, "--uastc-quality", &q.to_string());
    }
    if u.uastc_rdo {
        args.push("--uastc-rdo".to_owned());
    }
    if let Some(v) = u.uastc_rdo_l {
        push_flag_value(&mut args, "--uastc-rdo-l", &fmt_f32(v));
    }
    if let Some(v) = u.uastc_rdo_d {
        push_flag_value(&mut args, "--uastc-rdo-d", &v.to_string());
    }
    if let Some(v) = u.uastc_rdo_b {
        push_flag_value(&mut args, "--uastc-rdo-b", &fmt_f32(v));
    }
    if let Some(v) = u.uastc_rdo_s {
        push_flag_value(&mut args, "--uastc-rdo-s", &fmt_f32(v));
    }
    if u.uastc_rdo_f {
        args.push("--uastc-rdo-f".to_owned());
    }
    if u.uastc_rdo_m {
        args.push("--uastc-rdo-m".to_owned());
    }

    let a = &opts.astc;
    if let Some(q) = a.astc_quality {
        push_flag_value(&mut args, "--astc-quality", q.cli());
    }
    if a.astc_perceptual {
        args.push("--astc-perceptual".to_owned());
    }

    let e = &opts.encoder;
    if let Some(threads) = e.threads {
        push_flag_value(&mut args, "--threads", &threads.to_string());
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
            push_flag_value(&mut args, "--zstd", &level.to_string());
        }
        Supercompression::Zlib { level } => {
            push_flag_value(&mut args, "--zlib", &level.to_string());
        }
    }

    for input in inputs {
        args.push(input.to_string_lossy().into_owned());
    }
    args.push(output.to_string_lossy().into_owned());
    args
}

fn push_flag_value(args: &mut Vec<String>, flag: &str, value: &str) {
    args.push(flag.to_owned());
    args.push(value.to_owned());
}
