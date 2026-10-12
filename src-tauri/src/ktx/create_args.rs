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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{SourceInfo, SourceKind};
    use crate::ktx::input_role::InputRole;
    use crate::ktx::options::*;
    use crate::ktx::rules::resolve_for_encode;
    use std::collections::HashSet;

    /// Every v4.4.2 UI flag from requirements / OptionKey.
    const ALL_FLAGS: &[&str] = &[
        "--1d",
        "--cubemap",
        "--layers",
        "--depth",
        "--levels",
        "--generate-mipmap",
        "--runtime-mipmap",
        "--width",
        "--height",
        "--scale",
        "--raw",
        "--mipmap-filter",
        "--mipmap-filter-scale",
        "--mipmap-wrap",
        "--assign-tf",
        "--convert-tf",
        "--assign-primaries",
        "--convert-primaries",
        "--assign-texcoord-origin",
        "--convert-texcoord-origin",
        "--fail-on-color-conversions",
        "--warn-on-color-conversions",
        "--no-warn-on-color-conversions",
        "--fail-on-origin-changes",
        "--warn-on-origin-changes",
        "--swizzle",
        "--input-swizzle",
        "--normalize",
        "--normal-mode",
        "--format",
        "--encode",
        "--qlevel",
        "--clevel",
        "--max-endpoints",
        "--max-selectors",
        "--endpoint-rdo-threshold",
        "--selector-rdo-threshold",
        "--no-endpoint-rdo",
        "--no-selector-rdo",
        "--uastc-quality",
        "--uastc-rdo",
        "--uastc-rdo-l",
        "--uastc-rdo-d",
        "--uastc-rdo-b",
        "--uastc-rdo-s",
        "--uastc-rdo-f",
        "--uastc-rdo-m",
        "--astc-quality",
        "--astc-perceptual",
        "--threads",
        "--no-sse",
        "--compare-ssim",
        "--compare-psnr",
        "--zstd",
        "--zlib",
    ];

    fn inputs(n: usize) -> Vec<PathBuf> {
        (0..n)
            .map(|i| PathBuf::from(format!("in{i}.png")))
            .collect()
    }

    fn emit_role(opts: &CreateOptions, role: InputRole, n: usize) -> Vec<String> {
        let sources: Vec<_> = inputs(n)
            .iter()
            .map(|p| SourceInfo {
                path: p.to_string_lossy().into_owned(),
                name: p.to_string_lossy().into_owned(),
                kind: SourceKind::Ldr,
                width: 8,
                height: 8,
                needs_conversion: false,
            })
            .collect();
        let resolved =
            resolve_for_encode(opts, &sources, role).unwrap_or_else(|e| panic!("{role:?}: {e:?}"));
        build_create_args(&resolved, &inputs(n), Path::new("out.ktx2"))
    }

    fn emit(opts: &CreateOptions) -> Vec<String> {
        emit_role(opts, InputRole::Base2D, 1)
    }

    fn has_flag(args: &[String], flag: &str) -> bool {
        args.iter().any(|a| a == flag)
    }

    #[test]
    fn defaults_emit_format_and_paths() {
        let opts = CreateOptions::default();
        let threads = opts.encoder.threads.expect("default threads");
        assert_eq!(
            emit(&opts),
            vec![
                "--format".to_owned(),
                "R8G8B8A8_SRGB".to_owned(),
                "--scale".to_owned(),
                "1".to_owned(),
                "--mipmap-filter".to_owned(),
                "lanczos4".to_owned(),
                "--mipmap-filter-scale".to_owned(),
                "1".to_owned(),
                "--mipmap-wrap".to_owned(),
                "clamp".to_owned(),
                "--threads".to_owned(),
                threads.to_string(),
                "in0.png".to_owned(),
                "out.ktx2".to_owned(),
            ]
        );
    }

    #[test]
    fn hidden_compare_ssim_is_not_emitted() {
        // Regression: SSIM enabled under UASTC, then encode set back to none.
        let mut opts = CreateOptions::default();
        opts.encoder.compare_ssim = true;
        assert!(!has_flag(&emit(&opts), "--compare-ssim"));
        opts.encode = Some(EncodeCodec::Uastc);
        assert!(has_flag(&emit(&opts), "--compare-ssim"));
    }

    #[test]
    fn zstd_not_emitted_with_basis_lz() {
        let opts = CreateOptions {
            encode: Some(EncodeCodec::BasisLz),
            supercompression: Supercompression::Zstd { level: 3 },
            ..Default::default()
        };
        assert!(!has_flag(&emit(&opts), "--zstd"));
    }

    #[test]
    fn layers_with_four_inputs() {
        let args = emit_role(&CreateOptions::default(), InputRole::ArrayLayers, 4);
        assert!(args.windows(2).any(|w| w[0] == "--layers" && w[1] == "4"));
        assert!(args.ends_with(&[
            "in0.png".to_owned(),
            "in1.png".to_owned(),
            "in2.png".to_owned(),
            "in3.png".to_owned(),
            "out.ktx2".to_owned(),
        ]));
    }

    #[test]
    fn every_all_flags_entry_is_emitted_by_some_case() {
        let mut emitted = HashSet::new();
        let mut note = |args: Vec<String>| {
            for a in args {
                if a.starts_with("--") {
                    emitted.insert(a);
                }
            }
        };

        let mut o = CreateOptions::default();
        o.container.one_d = true;
        note(emit(&o));

        note(emit_role(
            &CreateOptions::default(),
            InputRole::CubemapFaces,
            6,
        ));

        let mut o = CreateOptions::default();
        o.container.levels = Some(3);
        o.container.generate_mipmap = true;
        o.container.width = Some(64);
        o.container.height = Some(64);
        o.container.scale = Some(0.5);
        o.filter.mipmap_filter = Some(MipmapFilter::Lanczos4);
        o.filter.mipmap_filter_scale = Some(1.25);
        o.filter.mipmap_wrap = Some(MipmapWrap::Clamp);
        note(emit_role(&o, InputRole::ArrayLayers, 2));

        let mut o = CreateOptions::default();
        o.container.runtime_mipmap = true;
        note(emit(&o));

        let mut o = CreateOptions::default();
        o.container.raw = true;
        o.container.width = Some(8);
        o.container.height = Some(8);
        note(emit(&o));

        let mut o = CreateOptions::default();
        o.color.convert_texcoord_origin = Some(TexcoordOrigin::BottomLeftFront);
        note(emit_role(&o, InputRole::DepthSlices, 4));

        // UNORM (not *_SRGB) so --convert-tf linear is valid under the hard TF rules.
        let mut o = CreateOptions {
            format: VkFormat::R8G8B8A8Unorm,
            ..Default::default()
        };
        o.color.assign_tf = Some(TransferFunction::Srgb);
        o.color.convert_tf = Some(ConvertTransferFunction::Linear);
        o.color.assign_primaries = Some(ColorPrimaries::Bt709);
        o.color.convert_primaries = Some(ConvertPrimaries::Srgb);
        o.color.fail_on_color_conversions = true;
        o.color.warn_on_color_conversions = true;
        o.color.no_warn_on_color_conversions = true;
        o.color.fail_on_origin_changes = true;
        o.color.warn_on_origin_changes = true;
        o.color.swizzle = Some(Swizzle::parse("rgba").unwrap());
        o.color.input_swizzle = Some(Swizzle::parse("bgra").unwrap());
        o.color.normalize = true;
        o.color.normal_mode = true;
        note(emit(&o));

        let mut o = CreateOptions {
            encode: Some(EncodeCodec::BasisLz),
            ..Default::default()
        };
        o.basis_lz.max_endpoints = Some(16);
        o.basis_lz.max_selectors = Some(16);
        o.basis_lz.no_endpoint_rdo = true;
        o.basis_lz.no_selector_rdo = true;
        note(emit(&o));

        let mut o = CreateOptions {
            encode: Some(EncodeCodec::Uastc),
            supercompression: Supercompression::Zstd { level: 3 },
            ..Default::default()
        };
        o.uastc_ldr.uastc_quality = Some(2);
        o.uastc_ldr.uastc_rdo = true;
        o.uastc_ldr.uastc_rdo_f = true;
        o.uastc_ldr.uastc_rdo_m = true;
        o.encoder.threads = Some(4);
        o.encoder.no_sse = true;
        o.encoder.compare_ssim = true;
        o.encoder.compare_psnr = true;
        note(emit(&o));

        let mut o = CreateOptions {
            format: VkFormat::Astc6x6SrgbBlock,
            ..Default::default()
        };
        o.astc.astc_perceptual = true;
        note(emit(&o));

        note(emit(&CreateOptions {
            encode: Some(EncodeCodec::Uastc),
            supercompression: Supercompression::Zlib { level: 6 },
            ..Default::default()
        }));

        for flag in ALL_FLAGS {
            assert!(
                emitted.contains(*flag),
                "flag {flag} was never emitted by any test case; emitted={emitted:?}"
            );
        }
    }
}
