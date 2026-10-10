//! Option rules engine: field enablement, warnings, and hard validation.

use strum::IntoEnumIterator;

use crate::commands::SourceInfo;
use crate::ktx::options::{
    ColorPrimaries, CreateOptions, EncodeCodec, FieldState, OptionKey, OptionReport, OptionWarning,
    Supercompression, TransferFunction, VkFormat, WarningLevel,
};

/// Return a [`FieldState`] for every [`OptionKey`], plus color / encode warnings.
pub fn describe(opts: &CreateOptions, _sources: &[SourceInfo]) -> OptionReport {
    let mut fields = Vec::new();
    for key in OptionKey::iter() {
        let (enabled, reason) = field_enabled(opts, key);
        fields.push(FieldState {
            key,
            enabled,
            reason,
        });
    }
    let mut warnings = Vec::new();
    warn_nonlinear_mipmap(opts, &mut warnings);
    warn_srgb_format_tf(opts, &mut warnings);
    warn_encode_color_space(opts, &mut warnings);
    warn_srgb_to_linear_lossy(opts, &mut warnings);
    OptionReport { fields, warnings }
}

/// Hard errors used by `create_args` so a bad request never reaches the binary.
pub fn validate(opts: &CreateOptions) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    reject_zstd_with_basis_lz(opts, &mut errors);
    reject_zlib_with_basis_lz(opts, &mut errors);
    reject_generate_mipmap_constraints(opts, &mut errors);
    reject_runtime_vs_generate(opts, &mut errors);
    reject_scale_with_raw(opts, &mut errors);
    reject_raw_missing_size(opts, &mut errors);
    reject_convert_tf_with_raw(opts, &mut errors);
    reject_convert_primaries_with_raw(opts, &mut errors);
    reject_convert_primaries_when_none(opts, &mut errors);
    reject_convert_origin_with_raw(opts, &mut errors);
    reject_normalize_with_raw(opts, &mut errors);
    reject_depth_origin(opts, &mut errors);
    reject_cubemap_origin(opts, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn field_enabled(opts: &CreateOptions, key: OptionKey) -> (bool, Option<String>) {
    match key {
        // "--zstd <1-22> — not valid with BasisLZ."
        OptionKey::Zstd => disable_if(
            opts.encode == Some(EncodeCodec::BasisLz),
            "not valid with BasisLZ",
        ),
        // "--zlib <1-9> — not valid with BasisLZ."
        OptionKey::Zlib => disable_if(
            opts.encode == Some(EncodeCodec::BasisLz),
            "not valid with BasisLZ",
        ),
        // "--generate-mipmap … cannot be used with SINT, UINT or 3D textures or --raw."
        OptionKey::GenerateMipmap => {
            if opts.container.raw {
                return (false, Some("cannot be used with raw".to_owned()));
            }
            if opts.container.depth.is_some() {
                return (false, Some("not valid for 3D textures".to_owned()));
            }
            if is_sint_or_uint(opts.format) {
                return (false, Some("not valid for SINT or UINT formats".to_owned()));
            }
            if opts.container.runtime_mipmap {
                return (
                    false,
                    Some("mutually exclusive with --runtime-mipmap".to_owned()),
                );
            }
            (true, None)
        }
        // "--runtime-mipmap … Mutually exclusive with --generate-mipmap"
        OptionKey::RuntimeMipmap => disable_if(
            opts.container.generate_mipmap,
            "mutually exclusive with --generate-mipmap",
        ),
        // "--scale … Not valid with raw"
        OptionKey::Scale => disable_if(opts.container.raw, "cannot be used with raw"),
        // "--width, --height … Required for raw"
        OptionKey::Width | OptionKey::Height => (true, None),
        // "--raw"
        OptionKey::Raw => (true, None),
        // Mipmap filters: "Ignored unless --generate-mipmap, --scale, --width or --height"
        OptionKey::MipmapFilter | OptionKey::MipmapFilterScale | OptionKey::MipmapWrap => {
            if opts.container.raw {
                return (false, Some("ignored for raw input".to_owned()));
            }
            let resampling = opts.container.generate_mipmap
                || opts.container.scale.is_some()
                || opts.container.width.is_some()
                || opts.container.height.is_some();
            disable_if(
                !resampling,
                "ignored unless --generate-mipmap, --scale, --width or --height are specified",
            )
        }
        // "--convert-tf … Not valid with raw"
        OptionKey::ConvertTf => disable_if(opts.container.raw, "not valid with raw"),
        // "--convert-primaries … Not valid with raw, and not valid when assigned primaries are none"
        OptionKey::ConvertPrimaries => {
            if opts.container.raw {
                return (false, Some("not valid with raw".to_owned()));
            }
            if opts.color.assign_primaries == Some(ColorPrimaries::None) {
                return (
                    false,
                    Some("not valid when assigned primaries are none".to_owned()),
                );
            }
            (true, None)
        }
        // "--convert-texcoord-origin … Not valid with raw"
        OptionKey::ConvertTexcoordOrigin => disable_if(opts.container.raw, "not valid with raw"),
        // "--normalize … Not valid with raw or premultiply"
        OptionKey::Normalize => disable_if(opts.container.raw, "not valid with raw"),
        // BasisLZ options only with basis-lz
        OptionKey::Qlevel
        | OptionKey::Clevel
        | OptionKey::MaxEndpoints
        | OptionKey::MaxSelectors
        | OptionKey::EndpointRdoThreshold
        | OptionKey::SelectorRdoThreshold
        | OptionKey::NoEndpointRdo
        | OptionKey::NoSelectorRdo => disable_if(
            opts.encode != Some(EncodeCodec::BasisLz),
            "ignored unless BasisLZ is selected",
        ),
        // "--max-endpoints … Takes effect only together with --max-selectors"
        // (still enabled when BasisLZ; informational in reason when only one set — kept enabled)
        // UASTC LDR options
        OptionKey::UastcQuality | OptionKey::UastcRdo => disable_if(
            opts.encode != Some(EncodeCodec::Uastc),
            "ignored unless UASTC is selected",
        ),
        OptionKey::UastcRdoL
        | OptionKey::UastcRdoD
        | OptionKey::UastcRdoB
        | OptionKey::UastcRdoS
        | OptionKey::UastcRdoF
        | OptionKey::UastcRdoM => {
            if opts.encode != Some(EncodeCodec::Uastc) {
                return (false, Some("ignored unless UASTC is selected".to_owned()));
            }
            disable_if(
                !opts.uastc_ldr.uastc_rdo,
                "ignored unless --uastc-rdo is enabled",
            )
        }
        // ASTC options when format is ASTC
        OptionKey::AstcQuality | OptionKey::AstcPerceptual => disable_if(
            !is_astc_format(opts.format),
            "ignored unless an ASTC format is selected",
        ),
        // "--no-sse — BasisLZ and UASTC only"
        OptionKey::NoSse => disable_if(
            !matches!(opts.encode, Some(EncodeCodec::BasisLz | EncodeCodec::Uastc)),
            "BasisLZ and UASTC only",
        ),
        // "--compare-ssim / --compare-psnr — Requires Basis-LZ, UASTC or ASTC encoding"
        OptionKey::CompareSsim | OptionKey::ComparePsnr => {
            let ok = opts.encode.is_some() || is_astc_format(opts.format);
            disable_if(!ok, "requires BasisLZ, UASTC, or ASTC encoding")
        }
        // Cubemap vs 1d vs depth — soft disable for conflicting shapes
        OptionKey::OneD => disable_if(
            opts.container.cubemap || opts.container.depth.is_some(),
            "not valid with cubemap or 3D",
        ),
        OptionKey::Cubemap => disable_if(
            opts.container.one_d || opts.container.depth.is_some(),
            "not valid with 1D or 3D",
        ),
        OptionKey::Depth => disable_if(
            opts.container.one_d || opts.container.cubemap,
            "not valid with 1D or cubemap",
        ),
        OptionKey::Layers
        | OptionKey::Levels
        | OptionKey::AssignTf
        | OptionKey::AssignPrimaries
        | OptionKey::AssignTexcoordOrigin
        | OptionKey::FailOnColorConversions
        | OptionKey::WarnOnColorConversions
        | OptionKey::NoWarnOnColorConversions
        | OptionKey::FailOnOriginChanges
        | OptionKey::WarnOnOriginChanges
        | OptionKey::Swizzle
        | OptionKey::InputSwizzle
        | OptionKey::NormalMode
        | OptionKey::Format
        | OptionKey::Encode
        | OptionKey::Threads => (true, None),
    }
}

fn disable_if(cond: bool, reason: &str) -> (bool, Option<String>) {
    if cond {
        (false, Some(reason.to_owned()))
    } else {
        (true, None)
    }
}

fn is_astc_format(format: VkFormat) -> bool {
    format.cli().starts_with("ASTC_")
}

fn is_sint_or_uint(format: VkFormat) -> bool {
    let s = format.cli();
    s.contains("_SINT") || s.contains("_UINT")
}

fn is_srgb_format(format: VkFormat) -> bool {
    format.cli().contains("_SRGB")
}

fn has_srgb_twin(format: VkFormat) -> bool {
    let s = format.cli();
    if s.contains("_SRGB") {
        return false;
    }
    matches!(
        format,
        VkFormat::R8Unorm | VkFormat::R8G8Unorm | VkFormat::R8G8B8Unorm | VkFormat::R8G8B8A8Unorm
    ) || (s.starts_with("ASTC_") && s.contains("_UNORM_"))
}

fn output_tf_is_srgb(opts: &CreateOptions) -> bool {
    if let Some(tf) = opts.color.convert_tf {
        return matches!(
            tf,
            crate::ktx::options::ConvertTransferFunction::Srgb
                | crate::ktx::options::ConvertTransferFunction::SrgbEotf
                | crate::ktx::options::ConvertTransferFunction::Scrgb
                | crate::ktx::options::ConvertTransferFunction::ScrgbEotf
        );
    }
    match opts.color.assign_tf {
        Some(
            TransferFunction::Srgb
            | TransferFunction::SrgbEotf
            | TransferFunction::Scrgb
            | TransferFunction::ScrgbEotf,
        ) => true,
        Some(_) => false,
        None => is_srgb_format(opts.format),
    }
}

fn is_nonlinear_non_srgb(tf: TransferFunction) -> bool {
    !matches!(
        tf,
        TransferFunction::Linear
            | TransferFunction::Srgb
            | TransferFunction::SrgbEotf
            | TransferFunction::Scrgb
            | TransferFunction::ScrgbEotf
    )
}

fn effective_output_tf(opts: &CreateOptions) -> Option<TransferFunction> {
    if let Some(tf) = opts.color.convert_tf {
        return Some(match tf {
            crate::ktx::options::ConvertTransferFunction::Linear => TransferFunction::Linear,
            crate::ktx::options::ConvertTransferFunction::Srgb
            | crate::ktx::options::ConvertTransferFunction::SrgbEotf
            | crate::ktx::options::ConvertTransferFunction::Scrgb
            | crate::ktx::options::ConvertTransferFunction::ScrgbEotf => TransferFunction::Srgb,
        });
    }
    opts.color.assign_tf
}

// "Mip generation is avoided when the output transfer function is non-linear and is not sRGB."
fn warn_nonlinear_mipmap(opts: &CreateOptions, warnings: &mut Vec<OptionWarning>) {
    if !opts.container.generate_mipmap {
        return;
    }
    if let Some(tf) = effective_output_tf(opts) {
        if is_nonlinear_non_srgb(tf) {
            warnings.push(OptionWarning {
                key: Some(OptionKey::GenerateMipmap),
                level: WarningLevel::Warn,
                message: "mip generation is avoided when the output transfer function is non-linear and is not sRGB; the resampler only decodes sRGB".to_owned(),
            });
        }
    }
}

// "*_SRGB formats require an sRGB output transfer function. Non-sRGB formats that have an sRGB twin reject an sRGB output transfer function."
fn warn_srgb_format_tf(opts: &CreateOptions, warnings: &mut Vec<OptionWarning>) {
    let srgb_out = output_tf_is_srgb(opts);
    if is_srgb_format(opts.format)
        && !srgb_out
        && (opts.color.convert_tf.is_some() || opts.color.assign_tf.is_some())
    {
        warnings.push(OptionWarning {
            key: Some(OptionKey::Format),
            level: WarningLevel::Warn,
            message: "*_SRGB formats require an sRGB output transfer function".to_owned(),
        });
    }
    if has_srgb_twin(opts.format) && srgb_out && opts.color.convert_tf.is_some() {
        warnings.push(OptionWarning {
            key: Some(OptionKey::ConvertTf),
            level: WarningLevel::Warn,
            message:
                "non-sRGB formats that have an sRGB twin reject an sRGB output transfer function"
                    .to_owned(),
        });
    }
}

// "Encoding to ASTC, BasisLZ, or UASTC is discouraged unless the data is linear or sRGB."
fn warn_encode_color_space(opts: &CreateOptions, warnings: &mut Vec<OptionWarning>) {
    let encoding = opts.encode.is_some() || is_astc_format(opts.format);
    if !encoding {
        return;
    }
    if let Some(tf) = effective_output_tf(opts) {
        if is_nonlinear_non_srgb(tf) {
            warnings.push(OptionWarning {
                key: Some(OptionKey::Encode),
                level: WarningLevel::Warn,
                message: "encoding to ASTC, BasisLZ, or UASTC is discouraged unless the data is linear or sRGB".to_owned(),
            });
        }
    }
}

// "sRGB to linear is treated as visually lossy."
fn warn_srgb_to_linear_lossy(opts: &CreateOptions, warnings: &mut Vec<OptionWarning>) {
    let from_srgb = matches!(
        opts.color.assign_tf,
        Some(
            TransferFunction::Srgb
                | TransferFunction::SrgbEotf
                | TransferFunction::Scrgb
                | TransferFunction::ScrgbEotf
        )
    );
    let to_linear = matches!(
        opts.color.convert_tf,
        Some(crate::ktx::options::ConvertTransferFunction::Linear)
    );
    if from_srgb && to_linear {
        warnings.push(OptionWarning {
            key: Some(OptionKey::ConvertTf),
            level: WarningLevel::Info,
            message: "sRGB to linear is treated as visually lossy".to_owned(),
        });
    }
}

// --- hard rejects (quotes from requirements / create help) ---

fn reject_zstd_with_basis_lz(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "--zstd … not valid with BasisLZ"
    if opts.encode == Some(EncodeCodec::BasisLz)
        && matches!(opts.supercompression, Supercompression::Zstd { .. })
    {
        errors.push("--zstd is not valid with BasisLZ".to_owned());
    }
}

fn reject_zlib_with_basis_lz(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "--zlib … not valid with BasisLZ"
    if opts.encode == Some(EncodeCodec::BasisLz)
        && matches!(opts.supercompression, Supercompression::Zlib { .. })
    {
        errors.push("--zlib is not valid with BasisLZ".to_owned());
    }
}

fn reject_generate_mipmap_constraints(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "Not valid for SINT, UINT, 3D, or raw"
    if !opts.container.generate_mipmap {
        return;
    }
    if opts.container.raw {
        errors.push("--generate-mipmap cannot be used with --raw".to_owned());
    }
    if opts.container.depth.is_some() {
        errors.push("--generate-mipmap is not valid for 3D textures".to_owned());
    }
    if is_sint_or_uint(opts.format) {
        errors.push("--generate-mipmap is not valid for SINT or UINT formats".to_owned());
    }
}

fn reject_runtime_vs_generate(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "Mutually exclusive with --generate-mipmap"
    if opts.container.generate_mipmap && opts.container.runtime_mipmap {
        errors.push("--runtime-mipmap is mutually exclusive with --generate-mipmap".to_owned());
    }
}

fn reject_scale_with_raw(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "Not valid with raw"
    if opts.container.raw && opts.container.scale.is_some() {
        errors.push("--scale cannot be used with --raw".to_owned());
    }
}

fn reject_raw_missing_size(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "Width and height required" for raw
    if opts.container.raw && (opts.container.width.is_none() || opts.container.height.is_none()) {
        errors.push("--width and --height are required for --raw".to_owned());
    }
}

fn reject_convert_tf_with_raw(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "Not valid with raw"
    if opts.container.raw && opts.color.convert_tf.is_some() {
        errors.push("--convert-tf is not valid with --raw".to_owned());
    }
}

fn reject_convert_primaries_with_raw(opts: &CreateOptions, errors: &mut Vec<String>) {
    if opts.container.raw && opts.color.convert_primaries.is_some() {
        errors.push("--convert-primaries is not valid with --raw".to_owned());
    }
}

fn reject_convert_primaries_when_none(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "not valid when assigned primaries are none"
    if opts.color.assign_primaries == Some(ColorPrimaries::None)
        && opts.color.convert_primaries.is_some()
    {
        errors.push("--convert-primaries is not valid when --assign-primaries is none".to_owned());
    }
}

fn reject_convert_origin_with_raw(opts: &CreateOptions, errors: &mut Vec<String>) {
    if opts.container.raw && opts.color.convert_texcoord_origin.is_some() {
        errors.push("--convert-texcoord-origin is not valid with --raw".to_owned());
    }
}

fn reject_normalize_with_raw(opts: &CreateOptions, errors: &mut Vec<String>) {
    if opts.container.raw && opts.color.normalize {
        errors.push("--normalize is not valid with --raw".to_owned());
    }
}

fn reject_depth_origin(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "--depth … Requires a front/back texture-coordinate origin"
    if opts.container.depth.is_none() {
        return;
    }
    let origin = opts
        .color
        .assign_texcoord_origin
        .or(opts.color.convert_texcoord_origin);
    let ok = matches!(
        origin,
        Some(
            crate::ktx::options::TexcoordOrigin::TopLeftFront
                | crate::ktx::options::TexcoordOrigin::TopLeftBack
                | crate::ktx::options::TexcoordOrigin::BottomLeftFront
                | crate::ktx::options::TexcoordOrigin::BottomLeftBack
        )
    );
    if !ok {
        errors.push("--depth requires a front/back texture-coordinate origin".to_owned());
    }
}

fn reject_cubemap_origin(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "Cubemaps must stay top-left"
    if !opts.container.cubemap {
        return;
    }
    for origin in [
        opts.color.assign_texcoord_origin,
        opts.color.convert_texcoord_origin,
    ]
    .into_iter()
    .flatten()
    {
        if !matches!(
            origin,
            crate::ktx::options::TexcoordOrigin::TopLeft
                | crate::ktx::options::TexcoordOrigin::TopLeftFront
                | crate::ktx::options::TexcoordOrigin::TopLeftBack
        ) {
            errors.push("cubemap texture-coordinate origin must be top-left".to_owned());
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ktx::options::*;

    fn opts() -> CreateOptions {
        CreateOptions::default()
    }

    fn report(o: &CreateOptions) -> OptionReport {
        describe(o, &[])
    }

    fn field(report: &OptionReport, key: OptionKey) -> &FieldState {
        report
            .fields
            .iter()
            .find(|f| f.key == key)
            .expect("field present")
    }

    #[test]
    fn report_has_one_entry_per_option_key() {
        let r = report(&opts());
        let keys: Vec<_> = OptionKey::iter().collect();
        assert_eq!(r.fields.len(), keys.len());
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(r.fields[i].key, *key);
        }
    }

    #[test]
    fn zstd_disabled_with_basis_lz() {
        // "--zstd … not valid with BasisLZ"
        let mut o = opts();
        o.encode = Some(EncodeCodec::BasisLz);
        assert!(!field(&report(&o), OptionKey::Zstd).enabled);
        o.supercompression = Supercompression::Zstd { level: 3 };
        assert!(validate(&o).is_err());
    }

    #[test]
    fn zlib_disabled_with_basis_lz() {
        // "--zlib … not valid with BasisLZ"
        let mut o = opts();
        o.encode = Some(EncodeCodec::BasisLz);
        assert!(!field(&report(&o), OptionKey::Zlib).enabled);
        o.supercompression = Supercompression::Zlib { level: 6 };
        assert!(validate(&o).is_err());
    }

    #[test]
    fn generate_mipmap_not_with_raw() {
        // "Not valid for … or raw"
        let mut o = opts();
        o.container.raw = true;
        o.container.width = Some(8);
        o.container.height = Some(8);
        assert!(!field(&report(&o), OptionKey::GenerateMipmap).enabled);
        o.container.generate_mipmap = true;
        assert!(validate(&o).is_err());
    }

    #[test]
    fn generate_mipmap_not_with_3d() {
        // "Not valid for … 3D …"
        let mut o = opts();
        o.container.depth = Some(4);
        o.color.assign_texcoord_origin = Some(TexcoordOrigin::TopLeftFront);
        assert!(!field(&report(&o), OptionKey::GenerateMipmap).enabled);
        o.container.generate_mipmap = true;
        assert!(validate(&o).is_err());
    }

    #[test]
    fn runtime_mutex_generate() {
        // "Mutually exclusive with --generate-mipmap"
        let mut o = opts();
        o.container.generate_mipmap = true;
        assert!(!field(&report(&o), OptionKey::RuntimeMipmap).enabled);
        o.container.runtime_mipmap = true;
        assert!(validate(&o).is_err());
    }

    #[test]
    fn scale_not_with_raw() {
        // "Not valid with raw"
        let mut o = opts();
        o.container.raw = true;
        o.container.width = Some(8);
        o.container.height = Some(8);
        assert!(!field(&report(&o), OptionKey::Scale).enabled);
        o.container.scale = Some(2.0);
        assert!(validate(&o).is_err());
    }

    #[test]
    fn raw_requires_width_height() {
        // "Width and height required"
        let mut o = opts();
        o.container.raw = true;
        assert!(validate(&o).is_err());
        o.container.width = Some(8);
        o.container.height = Some(8);
        assert!(validate(&o).is_ok());
    }

    #[test]
    fn convert_tf_not_with_raw() {
        // "Not valid with raw"
        let mut o = opts();
        o.container.raw = true;
        o.container.width = Some(8);
        o.container.height = Some(8);
        assert!(!field(&report(&o), OptionKey::ConvertTf).enabled);
        o.color.convert_tf = Some(ConvertTransferFunction::Linear);
        assert!(validate(&o).is_err());
    }

    #[test]
    fn convert_primaries_not_with_raw() {
        let mut o = opts();
        o.container.raw = true;
        o.container.width = Some(8);
        o.container.height = Some(8);
        assert!(!field(&report(&o), OptionKey::ConvertPrimaries).enabled);
    }

    #[test]
    fn convert_primaries_not_when_assign_none() {
        // "not valid when assigned primaries are none"
        let mut o = opts();
        o.color.assign_primaries = Some(ColorPrimaries::None);
        assert!(!field(&report(&o), OptionKey::ConvertPrimaries).enabled);
        o.color.convert_primaries = Some(ConvertPrimaries::Srgb);
        assert!(validate(&o).is_err());
    }

    #[test]
    fn convert_origin_not_with_raw() {
        let mut o = opts();
        o.container.raw = true;
        o.container.width = Some(8);
        o.container.height = Some(8);
        assert!(!field(&report(&o), OptionKey::ConvertTexcoordOrigin).enabled);
    }

    #[test]
    fn normalize_not_with_raw() {
        // "Not valid with raw or premultiply"
        let mut o = opts();
        o.container.raw = true;
        o.container.width = Some(8);
        o.container.height = Some(8);
        assert!(!field(&report(&o), OptionKey::Normalize).enabled);
        o.color.normalize = true;
        assert!(validate(&o).is_err());
    }

    #[test]
    fn depth_requires_front_back_origin() {
        // "Requires a front/back texture-coordinate origin"
        let mut o = opts();
        o.container.depth = Some(4);
        assert!(validate(&o).is_err());
        o.color.assign_texcoord_origin = Some(TexcoordOrigin::TopLeftFront);
        assert!(validate(&o).is_ok());
    }

    #[test]
    fn cubemap_origin_must_be_top_left() {
        // "Cubemaps must stay top-left"
        let mut o = opts();
        o.container.cubemap = true;
        o.color.assign_texcoord_origin = Some(TexcoordOrigin::BottomLeft);
        assert!(validate(&o).is_err());
    }

    #[test]
    fn mipmap_filters_ignored_unless_resampling() {
        // "Ignored unless --generate-mipmap, --scale, --width or --height"
        let o = opts();
        assert!(!field(&report(&o), OptionKey::MipmapFilter).enabled);
        let mut o = opts();
        o.container.generate_mipmap = true;
        assert!(field(&report(&o), OptionKey::MipmapFilter).enabled);
    }

    #[test]
    fn basis_lz_options_only_when_selected() {
        // "ignored unless BasisLZ is selected"
        let o = opts();
        assert!(!field(&report(&o), OptionKey::Qlevel).enabled);
        let mut o = opts();
        o.encode = Some(EncodeCodec::BasisLz);
        assert!(field(&report(&o), OptionKey::Qlevel).enabled);
    }

    #[test]
    fn uastc_options_only_when_selected() {
        // "ignored unless UASTC is selected"
        let o = opts();
        assert!(!field(&report(&o), OptionKey::UastcQuality).enabled);
        let mut o = opts();
        o.encode = Some(EncodeCodec::Uastc);
        assert!(field(&report(&o), OptionKey::UastcQuality).enabled);
    }

    #[test]
    fn uastc_rdo_params_need_rdo() {
        // RDO sub-flags ignored unless --uastc-rdo
        let mut o = opts();
        o.encode = Some(EncodeCodec::Uastc);
        assert!(!field(&report(&o), OptionKey::UastcRdoL).enabled);
        o.uastc_ldr.uastc_rdo = true;
        assert!(field(&report(&o), OptionKey::UastcRdoL).enabled);
    }

    #[test]
    fn astc_options_need_astc_format() {
        let o = opts();
        assert!(!field(&report(&o), OptionKey::AstcQuality).enabled);
        let mut o = opts();
        o.format = VkFormat::Astc6x6SrgbBlock;
        assert!(field(&report(&o), OptionKey::AstcQuality).enabled);
    }

    #[test]
    fn no_sse_basis_or_uastc_only() {
        // "BasisLZ and UASTC only"
        let o = opts();
        assert!(!field(&report(&o), OptionKey::NoSse).enabled);
        let mut o = opts();
        o.encode = Some(EncodeCodec::BasisLz);
        assert!(field(&report(&o), OptionKey::NoSse).enabled);
    }

    #[test]
    fn compare_metrics_need_encoding() {
        // "Requires Basis-LZ, UASTC or ASTC encoding"
        let o = opts();
        assert!(!field(&report(&o), OptionKey::CompareSsim).enabled);
        let mut o = opts();
        o.encode = Some(EncodeCodec::Uastc);
        assert!(field(&report(&o), OptionKey::CompareSsim).enabled);
    }

    #[test]
    fn one_d_not_with_cubemap() {
        let mut o = opts();
        o.container.cubemap = true;
        assert!(!field(&report(&o), OptionKey::OneD).enabled);
    }

    #[test]
    fn warn_nonlinear_mipmap() {
        // non-linear non-sRGB + generate-mipmap
        let mut o = opts();
        o.container.generate_mipmap = true;
        o.color.assign_tf = Some(TransferFunction::PqOetf);
        let r = report(&o);
        assert!(r
            .warnings
            .iter()
            .any(|w| w.message.contains("mip generation is avoided")));
    }

    #[test]
    fn warn_srgb_format_needs_srgb_tf() {
        // "*_SRGB formats require an sRGB output transfer function"
        let mut o = opts();
        o.format = VkFormat::R8G8B8A8Srgb;
        o.color.convert_tf = Some(ConvertTransferFunction::Linear);
        let r = report(&o);
        assert!(r.warnings.iter().any(|w| w.message.contains("*_SRGB")));
    }

    #[test]
    fn warn_non_srgb_rejects_srgb_tf() {
        let mut o = opts();
        o.format = VkFormat::R8G8B8A8Unorm;
        o.color.convert_tf = Some(ConvertTransferFunction::Srgb);
        let r = report(&o);
        assert!(r.warnings.iter().any(|w| w.message.contains("sRGB twin")));
    }

    #[test]
    fn warn_encode_discouraged_for_other_tf() {
        // "Encoding … discouraged unless the data is linear or sRGB"
        let mut o = opts();
        o.encode = Some(EncodeCodec::BasisLz);
        o.color.assign_tf = Some(TransferFunction::PqOetf);
        let r = report(&o);
        assert!(r.warnings.iter().any(|w| w.message.contains("discouraged")));
    }

    #[test]
    fn warn_srgb_to_linear_lossy() {
        // "sRGB to linear is treated as visually lossy"
        let mut o = opts();
        o.color.assign_tf = Some(TransferFunction::Srgb);
        o.color.convert_tf = Some(ConvertTransferFunction::Linear);
        let r = report(&o);
        assert!(r
            .warnings
            .iter()
            .any(|w| w.message.contains("visually lossy")));
    }

    #[test]
    fn zstd_ok_with_uastc() {
        let mut o = opts();
        o.encode = Some(EncodeCodec::Uastc);
        o.supercompression = Supercompression::Zstd { level: 3 };
        assert!(field(&report(&o), OptionKey::Zstd).enabled);
        assert!(validate(&o).is_ok());
    }
}
