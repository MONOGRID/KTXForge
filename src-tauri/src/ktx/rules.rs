//! Option rules engine: field enablement, warnings, and hard validation.

use strum::IntoEnumIterator;

use crate::commands::{SourceInfo, SourceKind};
use crate::ktx::option_help;
use crate::ktx::options::{
    ColorPrimaries, ConvertTransferFunction, CreateOptions, EncodeCodec, FieldState, OptionKey,
    OptionReport, OptionSuggestion, OptionWarning, SourcesClass, Supercompression,
    TransferFunction, VkFormat, WarningLevel,
};

/// Return a [`FieldState`] for every [`OptionKey`], plus color / encode warnings.
///
/// Filters format / encode choices from the source set so the UI cannot pick a combination
/// known to fail `ktx create`. When current options are invalid, [`OptionReport::suggestion`]
/// tells Vue what to apply.
pub fn describe(opts: &CreateOptions, sources: &[SourceInfo]) -> OptionReport {
    let sources_class = classify_sources(sources);
    let mut fields = Vec::new();
    for key in OptionKey::iter() {
        let (mut enabled, mut reason) = field_enabled(opts, key);
        apply_source_enablement(key, sources_class, &mut enabled, &mut reason);
        let choices = filter_choices(key, opts, sources_class, option_help::choices_for(key));
        fields.push(FieldState {
            key,
            enabled,
            reason,
            help: option_help::help_for(key).to_owned(),
            choices,
        });
    }
    let mut warnings = Vec::new();
    warn_nonlinear_mipmap(opts, &mut warnings);
    warn_encode_color_space(opts, &mut warnings);
    warn_srgb_to_linear_lossy(opts, &mut warnings);
    warn_mixed_sources(sources_class, &mut warnings);
    let suggestion = suggestion_for(opts, sources_class);
    OptionReport {
        fields,
        warnings,
        sources_class,
        suggestion,
    }
}

/// Hard errors used by `create_args` so a bad request never reaches the binary.
pub fn validate(opts: &CreateOptions) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    reject_option_constraints(opts, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Like [`validate`], plus source/format pairing rules (mixed LDR+HDR, HDR vs 8-bit, etc.).
pub fn validate_for_sources(
    opts: &CreateOptions,
    sources: &[SourceInfo],
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    reject_option_constraints(opts, &mut errors);
    reject_source_option_mismatch(opts, sources, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn reject_option_constraints(opts: &CreateOptions, errors: &mut Vec<String>) {
    reject_zstd_with_basis_lz(opts, errors);
    reject_zlib_with_basis_lz(opts, errors);
    reject_generate_mipmap_constraints(opts, errors);
    reject_runtime_vs_generate(opts, errors);
    reject_raw_missing_size(opts, errors);
    reject_convert_tf_with_raw(opts, errors);
    reject_tf_format_mismatch(opts, errors);
    reject_convert_primaries_with_raw(opts, errors);
    reject_convert_primaries_when_none(opts, errors);
    reject_convert_origin_with_raw(opts, errors);
    reject_normalize_with_raw(opts, errors);
    reject_depth_origin(opts, errors);
    reject_cubemap_origin(opts, errors);
}

pub fn classify_sources(sources: &[SourceInfo]) -> SourcesClass {
    if sources.is_empty() {
        return SourcesClass::Empty;
    }
    let mut saw_ldr = false;
    let mut saw_hdr = false;
    for source in sources {
        match source.kind {
            SourceKind::Ldr => saw_ldr = true,
            SourceKind::Hdr => saw_hdr = true,
        }
    }
    match (saw_ldr, saw_hdr) {
        (true, true) => SourcesClass::Mixed,
        (false, true) => SourcesClass::Hdr,
        (true, false) => SourcesClass::Ldr,
        (false, false) => SourcesClass::Empty,
    }
}

fn is_hdr_uncompressed(format: VkFormat) -> bool {
    matches!(
        format,
        VkFormat::R16G16B16Sfloat | VkFormat::R16G16B16A16Sfloat
    )
}

fn format_allowed_for_class(format: VkFormat, class: SourcesClass) -> bool {
    match class {
        SourcesClass::Empty | SourcesClass::Mixed => true,
        SourcesClass::Hdr => is_hdr_uncompressed(format),
        SourcesClass::Ldr => !is_hdr_uncompressed(format),
    }
}

fn options_compatible(opts: &CreateOptions, class: SourcesClass) -> bool {
    match class {
        SourcesClass::Empty | SourcesClass::Mixed => true,
        SourcesClass::Hdr => {
            is_hdr_uncompressed(opts.format)
                && opts.encode.is_none()
                && !is_astc_format(opts.format)
        }
        SourcesClass::Ldr => !is_hdr_uncompressed(opts.format),
    }
}

fn suggestion_for(opts: &CreateOptions, class: SourcesClass) -> Option<OptionSuggestion> {
    let (format, encode) = if options_compatible(opts, class) {
        (opts.format, opts.encode)
    } else {
        match class {
            SourcesClass::Hdr => (VkFormat::R16G16B16A16Sfloat, None),
            SourcesClass::Ldr => (VkFormat::R8G8B8A8Srgb, opts.encode),
            SourcesClass::Empty | SourcesClass::Mixed => return None,
        }
    };
    let convert_tf = sanitize_convert_tf(format, opts.color.convert_tf);
    let assign_tf = sanitize_assign_tf(format, opts.color.assign_tf);
    if format == opts.format
        && encode == opts.encode
        && convert_tf == opts.color.convert_tf
        && assign_tf == opts.color.assign_tf
    {
        return None;
    }
    Some(OptionSuggestion {
        format,
        encode,
        convert_tf,
        assign_tf,
    })
}

fn apply_source_enablement(
    key: OptionKey,
    class: SourcesClass,
    enabled: &mut bool,
    reason: &mut Option<String>,
) {
    if !*enabled {
        return;
    }
    match (key, class) {
        (OptionKey::Encode, SourcesClass::Hdr) => {
            *enabled = false;
            *reason = Some(
                "BasisLZ and UASTC LDR require 8-bit input; HDR sources need an uncompressed float format"
                    .to_owned(),
            );
        }
        (OptionKey::Encode, SourcesClass::Mixed) => {
            *enabled = false;
            *reason = Some("cannot encode mixed LDR and HDR sources together".to_owned());
        }
        (OptionKey::AstcQuality | OptionKey::AstcPerceptual, SourcesClass::Hdr) => {
            *enabled = false;
            *reason =
                Some("ASTC is LDR-only; HDR sources need an uncompressed float format".to_owned());
        }
        (OptionKey::AstcQuality | OptionKey::AstcPerceptual, SourcesClass::Mixed) => {
            *enabled = false;
            *reason = Some("cannot encode mixed LDR and HDR sources together".to_owned());
        }
        (
            OptionKey::CompareSsim | OptionKey::ComparePsnr,
            SourcesClass::Hdr | SourcesClass::Mixed,
        ) => {
            *enabled = false;
            *reason = Some("compare metrics require BasisLZ, UASTC, or ASTC".to_owned());
        }
        _ => {}
    }
}

fn filter_choices(
    key: OptionKey,
    opts: &CreateOptions,
    class: SourcesClass,
    choices: Vec<crate::ktx::options::OptionChoice>,
) -> Vec<crate::ktx::options::OptionChoice> {
    match key {
        OptionKey::Format => choices
            .into_iter()
            .filter(|c| {
                VkFormat::iter()
                    .any(|f| f.cli() == c.value.as_str() && format_allowed_for_class(f, class))
            })
            .collect(),
        OptionKey::Encode if matches!(class, SourcesClass::Hdr | SourcesClass::Mixed) => Vec::new(),
        OptionKey::ConvertTf => choices
            .into_iter()
            .filter(|c| {
                ConvertTransferFunction::iter()
                    .any(|tf| tf.cli() == c.value.as_str() && convert_tf_allowed(opts.format, tf))
            })
            .collect(),
        OptionKey::AssignTf => choices
            .into_iter()
            .filter(|c| {
                TransferFunction::iter()
                    .any(|tf| tf.cli() == c.value.as_str() && assign_tf_allowed(opts.format, tf))
            })
            .collect(),
        _ => choices,
    }
}

fn warn_mixed_sources(class: SourcesClass, warnings: &mut Vec<OptionWarning>) {
    if class != SourcesClass::Mixed {
        return;
    }
    warnings.push(OptionWarning {
        key: Some(OptionKey::Format),
        level: WarningLevel::Warn,
        message: "sources must be all LDR or all HDR; mixed inputs cannot be encoded together"
            .to_owned(),
    });
}

fn reject_source_option_mismatch(
    opts: &CreateOptions,
    sources: &[SourceInfo],
    errors: &mut Vec<String>,
) {
    let class = classify_sources(sources);
    match class {
        SourcesClass::Empty => {}
        SourcesClass::Mixed => {
            errors.push("sources must be all LDR or all HDR".to_owned());
        }
        SourcesClass::Hdr => {
            if !is_hdr_uncompressed(opts.format) {
                errors
                    .push("HDR sources require R16G16B16_SFLOAT or R16G16B16A16_SFLOAT".to_owned());
            }
            if opts.encode.is_some() {
                errors.push(
                    "HDR sources cannot use BasisLZ or UASTC LDR encode on this tool version"
                        .to_owned(),
                );
            }
            if is_astc_format(opts.format) {
                errors.push("HDR sources cannot use ASTC formats".to_owned());
            }
        }
        SourcesClass::Ldr => {
            if is_hdr_uncompressed(opts.format) {
                errors.push(
                    "LDR sources cannot use half-float formats; use an 8-bit or ASTC format"
                        .to_owned(),
                );
            }
        }
    }
}

/// Atomic facts about [`CreateOptions`], composed with [`Pred::Any`] / [`Pred::Not`].
/// AND across conditions is multiple [`Gate`]s on one key (each with its own reason).
#[derive(Clone, Copy)]
enum Pred {
    Raw,
    OneD,
    Cubemap,
    DepthSet,
    GenerateMipmap,
    RuntimeMipmap,
    ScaleSet,
    WidthSet,
    HeightSet,
    UastcRdo,
    EncodeBasisLz,
    EncodeUastc,
    EncodeSet,
    AstcFormat,
    SintOrUintFormat,
    AssignPrimariesNone,
    Not(&'static Pred),
    Any(&'static [Pred]),
}

/// One must-hold condition. All gates for a key must pass; first failure supplies `reason`.
#[derive(Clone, Copy)]
struct Gate {
    pred: Pred,
    reason: &'static str,
}

const NOT_RAW: Pred = Pred::Not(&Pred::Raw);
const NOT_ENCODE_BASIS_LZ: Pred = Pred::Not(&Pred::EncodeBasisLz);
const NOT_GENERATE_MIPMAP: Pred = Pred::Not(&Pred::GenerateMipmap);
const NOT_RUNTIME_MIPMAP: Pred = Pred::Not(&Pred::RuntimeMipmap);
const NOT_ONE_D: Pred = Pred::Not(&Pred::OneD);
const NOT_CUBEMAP: Pred = Pred::Not(&Pred::Cubemap);
const NOT_DEPTH_SET: Pred = Pred::Not(&Pred::DepthSet);
const NOT_SINT_OR_UINT: Pred = Pred::Not(&Pred::SintOrUintFormat);
const NOT_ASSIGN_PRIMARIES_NONE: Pred = Pred::Not(&Pred::AssignPrimariesNone);

const CUBEMAP_OR_DEPTH: &[Pred] = &[Pred::Cubemap, Pred::DepthSet];
const ONE_D_OR_DEPTH: &[Pred] = &[Pred::OneD, Pred::DepthSet];
const ONE_D_OR_CUBEMAP: &[Pred] = &[Pred::OneD, Pred::Cubemap];
const NOT_CUBEMAP_OR_DEPTH: Pred = Pred::Not(&Pred::Any(CUBEMAP_OR_DEPTH));
const NOT_ONE_D_OR_DEPTH: Pred = Pred::Not(&Pred::Any(ONE_D_OR_DEPTH));
const NOT_ONE_D_OR_CUBEMAP: Pred = Pred::Not(&Pred::Any(ONE_D_OR_CUBEMAP));

const RESAMPLE: &[Pred] = &[
    Pred::GenerateMipmap,
    Pred::ScaleSet,
    Pred::WidthSet,
    Pred::HeightSet,
];
const RESAMPLE_ANY: Pred = Pred::Any(RESAMPLE);

const BASIS_OR_UASTC: &[Pred] = &[Pred::EncodeBasisLz, Pred::EncodeUastc];
const BASIS_OR_UASTC_ANY: Pred = Pred::Any(BASIS_OR_UASTC);

const ENCODE_OR_ASTC: &[Pred] = &[Pred::EncodeSet, Pred::AstcFormat];
const ENCODE_OR_ASTC_ANY: Pred = Pred::Any(ENCODE_OR_ASTC);

fn gates(key: OptionKey) -> &'static [Gate] {
    match key {
        OptionKey::Zstd | OptionKey::Zlib => &[Gate {
            pred: NOT_ENCODE_BASIS_LZ,
            reason: "not valid with BasisLZ",
        }],
        OptionKey::GenerateMipmap => &[
            Gate {
                pred: NOT_RAW,
                reason: "cannot be used with raw",
            },
            Gate {
                pred: NOT_DEPTH_SET,
                reason: "not valid for 3D textures",
            },
            Gate {
                pred: NOT_SINT_OR_UINT,
                reason: "not valid for SINT or UINT formats",
            },
            Gate {
                pred: NOT_RUNTIME_MIPMAP,
                reason: "mutually exclusive with --runtime-mipmap",
            },
        ],
        OptionKey::RuntimeMipmap => &[Gate {
            pred: NOT_GENERATE_MIPMAP,
            reason: "mutually exclusive with --generate-mipmap",
        }],
        OptionKey::Scale => &[Gate {
            pred: NOT_RAW,
            reason: "cannot be used with raw",
        }],
        OptionKey::MipmapFilter | OptionKey::MipmapFilterScale | OptionKey::MipmapWrap => &[
            Gate {
                pred: NOT_RAW,
                reason: "ignored for raw input",
            },
            Gate {
                pred: RESAMPLE_ANY,
                reason:
                    "ignored unless --generate-mipmap, --scale, --width or --height are specified",
            },
        ],
        OptionKey::ConvertTf | OptionKey::ConvertTexcoordOrigin | OptionKey::Normalize => &[Gate {
            pred: NOT_RAW,
            reason: "not valid with raw",
        }],
        OptionKey::ConvertPrimaries => &[
            Gate {
                pred: NOT_RAW,
                reason: "not valid with raw",
            },
            Gate {
                pred: NOT_ASSIGN_PRIMARIES_NONE,
                reason: "not valid when assigned primaries are none",
            },
        ],
        OptionKey::Qlevel
        | OptionKey::Clevel
        | OptionKey::MaxEndpoints
        | OptionKey::MaxSelectors
        | OptionKey::EndpointRdoThreshold
        | OptionKey::SelectorRdoThreshold
        | OptionKey::NoEndpointRdo
        | OptionKey::NoSelectorRdo => &[Gate {
            pred: Pred::EncodeBasisLz,
            reason: "ignored unless BasisLZ is selected",
        }],
        OptionKey::UastcQuality | OptionKey::UastcRdo => &[Gate {
            pred: Pred::EncodeUastc,
            reason: "ignored unless UASTC is selected",
        }],
        OptionKey::UastcRdoL
        | OptionKey::UastcRdoD
        | OptionKey::UastcRdoB
        | OptionKey::UastcRdoS
        | OptionKey::UastcRdoF
        | OptionKey::UastcRdoM => &[
            Gate {
                pred: Pred::EncodeUastc,
                reason: "ignored unless UASTC is selected",
            },
            Gate {
                pred: Pred::UastcRdo,
                reason: "ignored unless --uastc-rdo is enabled",
            },
        ],
        OptionKey::AstcQuality | OptionKey::AstcPerceptual => &[Gate {
            pred: Pred::AstcFormat,
            reason: "ignored unless an ASTC format is selected",
        }],
        OptionKey::NoSse => &[Gate {
            pred: BASIS_OR_UASTC_ANY,
            reason: "BasisLZ and UASTC only",
        }],
        OptionKey::CompareSsim | OptionKey::ComparePsnr => &[Gate {
            pred: ENCODE_OR_ASTC_ANY,
            reason: "requires BasisLZ, UASTC, or ASTC encoding",
        }],
        OptionKey::OneD => &[Gate {
            pred: NOT_CUBEMAP_OR_DEPTH,
            reason: "not valid with cubemap or 3D",
        }],
        OptionKey::Cubemap => &[Gate {
            pred: NOT_ONE_D_OR_DEPTH,
            reason: "not valid with 1D or 3D",
        }],
        OptionKey::Depth => &[Gate {
            pred: NOT_ONE_D_OR_CUBEMAP,
            reason: "not valid with 1D or cubemap",
        }],
        OptionKey::Width
        | OptionKey::Height
        | OptionKey::Raw
        | OptionKey::Layers
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
        | OptionKey::Threads => &[],
    }
}

fn field_enabled(opts: &CreateOptions, key: OptionKey) -> (bool, Option<String>) {
    for gate in gates(key) {
        if !eval_pred(gate.pred, opts) {
            return (false, Some(gate.reason.to_owned()));
        }
    }
    (true, None)
}

fn eval_pred(pred: Pred, opts: &CreateOptions) -> bool {
    match pred {
        Pred::Raw => opts.container.raw,
        Pred::OneD => opts.container.one_d,
        Pred::Cubemap => opts.container.cubemap,
        Pred::DepthSet => opts.container.depth.is_some(),
        Pred::GenerateMipmap => opts.container.generate_mipmap,
        Pred::RuntimeMipmap => opts.container.runtime_mipmap,
        Pred::ScaleSet => opts.container.scale.is_some(),
        Pred::WidthSet => opts.container.width.is_some(),
        Pred::HeightSet => opts.container.height.is_some(),
        Pred::UastcRdo => opts.uastc_ldr.uastc_rdo,
        Pred::EncodeBasisLz => opts.encode == Some(EncodeCodec::BasisLz),
        Pred::EncodeUastc => opts.encode == Some(EncodeCodec::Uastc),
        Pred::EncodeSet => opts.encode.is_some(),
        Pred::AstcFormat => is_astc_format(opts.format),
        Pred::SintOrUintFormat => is_sint_or_uint(opts.format),
        Pred::AssignPrimariesNone => opts.color.assign_primaries == Some(ColorPrimaries::None),
        Pred::Not(inner) => !eval_pred(*inner, opts),
        Pred::Any(parts) => parts.iter().any(|p| eval_pred(*p, opts)),
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

fn is_srgb_convert_tf(tf: ConvertTransferFunction) -> bool {
    !matches!(tf, ConvertTransferFunction::Linear)
}

fn is_srgb_assign_tf(tf: TransferFunction) -> bool {
    matches!(
        tf,
        TransferFunction::Srgb
            | TransferFunction::SrgbEotf
            | TransferFunction::Scrgb
            | TransferFunction::ScrgbEotf
    )
}

/// `*_SRGB` formats require an sRGB convert-tf; formats with an sRGB twin reject sRGB convert-tf.
fn convert_tf_allowed(format: VkFormat, tf: ConvertTransferFunction) -> bool {
    if is_srgb_format(format) {
        is_srgb_convert_tf(tf)
    } else if has_srgb_twin(format) {
        matches!(tf, ConvertTransferFunction::Linear)
    } else {
        true
    }
}

/// `*_SRGB` formats require an sRGB assign-tf when the flag is set.
fn assign_tf_allowed(format: VkFormat, tf: TransferFunction) -> bool {
    if is_srgb_format(format) {
        is_srgb_assign_tf(tf)
    } else {
        true
    }
}

fn sanitize_convert_tf(
    format: VkFormat,
    tf: Option<ConvertTransferFunction>,
) -> Option<ConvertTransferFunction> {
    tf.filter(|value| convert_tf_allowed(format, *value))
}

fn sanitize_assign_tf(format: VkFormat, tf: Option<TransferFunction>) -> Option<TransferFunction> {
    tf.filter(|value| assign_tf_allowed(format, *value))
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
            ConvertTransferFunction::Linear => TransferFunction::Linear,
            ConvertTransferFunction::Srgb
            | ConvertTransferFunction::SrgbEotf
            | ConvertTransferFunction::Scrgb
            | ConvertTransferFunction::ScrgbEotf => TransferFunction::Srgb,
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
        Some(ConvertTransferFunction::Linear)
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

fn reject_tf_format_mismatch(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "*_SRGB formats require an sRGB output transfer function.
    //  Non-sRGB formats that have an sRGB twin reject an sRGB output transfer function."
    if let Some(tf) = opts.color.convert_tf {
        if !convert_tf_allowed(opts.format, tf) {
            if is_srgb_format(opts.format) {
                errors.push(
                    "--convert-tf must be sRGB for *_SRGB formats (linear is invalid)".to_owned(),
                );
            } else {
                errors.push(
                    "--convert-tf sRGB is invalid for non-sRGB formats that have an sRGB twin"
                        .to_owned(),
                );
            }
        }
    }
    if let Some(tf) = opts.color.assign_tf {
        if !assign_tf_allowed(opts.format, tf) {
            errors.push("--assign-tf must be sRGB for *_SRGB formats".to_owned());
        }
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
        // "Not valid with raw" — soft-disabled; default scale may remain but is not emitted.
        let mut o = opts();
        o.container.raw = true;
        o.container.width = Some(8);
        o.container.height = Some(8);
        assert!(!field(&report(&o), OptionKey::Scale).enabled);
        assert!(validate(&o).is_ok());
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
        // Default scale is 1.0, so filters start enabled; clear scale to test the gate.
        let mut o = opts();
        o.container.scale = None;
        assert!(!field(&report(&o), OptionKey::MipmapFilter).enabled);
        o.container.generate_mipmap = true;
        assert!(field(&report(&o), OptionKey::MipmapFilter).enabled);
        let o = opts();
        assert_eq!(o.container.scale, Some(1.0));
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
    fn srgb_format_hides_linear_convert_tf_and_rejects() {
        // "*_SRGB formats require an sRGB output transfer function"
        let mut o = opts();
        o.format = VkFormat::R8G8B8A8Srgb;
        o.color.convert_tf = Some(ConvertTransferFunction::Linear);
        let r = report(&o);
        let convert_choices: Vec<_> = field(&r, OptionKey::ConvertTf)
            .choices
            .iter()
            .map(|c| c.value.as_str())
            .collect();
        assert!(!convert_choices.contains(&"linear"));
        assert!(convert_choices.contains(&"srgb"));
        assert_eq!(
            r.suggestion,
            Some(OptionSuggestion {
                format: VkFormat::R8G8B8A8Srgb,
                encode: None,
                convert_tf: None,
                assign_tf: None,
            })
        );
        assert!(validate(&o).is_err());
    }

    #[test]
    fn unorm_twin_hides_srgb_convert_tf_and_rejects() {
        // "Non-sRGB formats that have an sRGB twin reject an sRGB output transfer function"
        let mut o = opts();
        o.format = VkFormat::R8G8B8A8Unorm;
        o.color.convert_tf = Some(ConvertTransferFunction::Srgb);
        let r = report(&o);
        let convert_choices: Vec<_> = field(&r, OptionKey::ConvertTf)
            .choices
            .iter()
            .map(|c| c.value.as_str())
            .collect();
        assert!(convert_choices.contains(&"linear"));
        assert!(!convert_choices.contains(&"srgb"));
        assert_eq!(
            r.suggestion,
            Some(OptionSuggestion {
                format: VkFormat::R8G8B8A8Unorm,
                encode: None,
                convert_tf: None,
                assign_tf: None,
            })
        );
        assert!(validate(&o).is_err());
    }

    #[test]
    fn srgb_format_hides_non_srgb_assign_tf() {
        let mut o = opts();
        o.format = VkFormat::R8G8B8A8Srgb;
        o.color.assign_tf = Some(TransferFunction::Linear);
        let r = report(&o);
        let assign_choices: Vec<_> = field(&r, OptionKey::AssignTf)
            .choices
            .iter()
            .map(|c| c.value.as_str())
            .collect();
        assert!(!assign_choices.contains(&"linear"));
        assert!(assign_choices.contains(&"srgb"));
        assert!(validate(&o).is_err());
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
        // Use UNORM (sRGB twin): convert-tf linear is allowed; assign srgb + convert linear warns.
        let mut o = opts();
        o.format = VkFormat::R8G8B8A8Unorm;
        o.color.assign_tf = Some(TransferFunction::Srgb);
        o.color.convert_tf = Some(ConvertTransferFunction::Linear);
        let r = report(&o);
        assert!(r
            .warnings
            .iter()
            .any(|w| w.message.contains("visually lossy")));
        assert!(r.suggestion.is_none());
        assert!(validate(&o).is_ok());
    }

    #[test]
    fn zstd_ok_with_uastc() {
        let mut o = opts();
        o.encode = Some(EncodeCodec::Uastc);
        o.supercompression = Supercompression::Zstd { level: 3 };
        assert!(field(&report(&o), OptionKey::Zstd).enabled);
        assert!(validate(&o).is_ok());
    }

    fn hdr_source() -> SourceInfo {
        SourceInfo {
            path: "a.exr".into(),
            name: "a.exr".into(),
            kind: SourceKind::Hdr,
            width: 64,
            height: 64,
            needs_conversion: false,
        }
    }

    fn ldr_source() -> SourceInfo {
        SourceInfo {
            path: "a.png".into(),
            name: "a.png".into(),
            kind: SourceKind::Ldr,
            width: 64,
            height: 64,
            needs_conversion: false,
        }
    }

    #[test]
    fn hdr_suggests_rgba16_float_and_hides_ldr_formats() {
        let o = opts();
        let r = describe(&o, &[hdr_source()]);
        assert_eq!(r.sources_class, SourcesClass::Hdr);
        assert_eq!(
            r.suggestion,
            Some(OptionSuggestion {
                format: VkFormat::R16G16B16A16Sfloat,
                encode: None,
                convert_tf: None,
                assign_tf: None,
            })
        );
        let format_choices: Vec<_> = field(&r, OptionKey::Format)
            .choices
            .iter()
            .map(|c| c.value.as_str())
            .collect();
        assert!(format_choices.contains(&"R16G16B16A16_SFLOAT"));
        assert!(format_choices.contains(&"R16G16B16_SFLOAT"));
        assert!(!format_choices.iter().any(|v| v.contains("R8")));
        assert!(!format_choices.iter().any(|v| v.starts_with("ASTC_")));
        assert!(!field(&r, OptionKey::Encode).enabled);
        assert!(validate_for_sources(&o, &[hdr_source()]).is_err());
    }

    #[test]
    fn hdr_compatible_options_have_no_suggestion() {
        let mut o = opts();
        o.format = VkFormat::R16G16B16A16Sfloat;
        o.encode = None;
        let r = describe(&o, &[hdr_source()]);
        assert!(r.suggestion.is_none());
        assert!(validate_for_sources(&o, &[hdr_source()]).is_ok());
    }

    #[test]
    fn ldr_hides_float_formats_and_rejects_float_selection() {
        let mut o = opts();
        o.format = VkFormat::R16G16B16A16Sfloat;
        let r = describe(&o, &[ldr_source()]);
        assert_eq!(r.sources_class, SourcesClass::Ldr);
        assert_eq!(
            r.suggestion,
            Some(OptionSuggestion {
                format: VkFormat::R8G8B8A8Srgb,
                encode: None,
                convert_tf: None,
                assign_tf: None,
            })
        );
        let format_choices: Vec<_> = field(&r, OptionKey::Format)
            .choices
            .iter()
            .map(|c| c.value.as_str())
            .collect();
        assert!(!format_choices.iter().any(|v| v.contains("SFLOAT")));
        assert!(validate_for_sources(&o, &[ldr_source()]).is_err());
    }

    #[test]
    fn mixed_sources_warn_and_fail_validate() {
        let o = opts();
        let r = describe(&o, &[ldr_source(), hdr_source()]);
        assert_eq!(r.sources_class, SourcesClass::Mixed);
        assert!(r.suggestion.is_none());
        assert!(r.warnings.iter().any(|w| w.message.contains("all LDR")));
        assert!(validate_for_sources(&o, &[ldr_source(), hdr_source()]).is_err());
    }
}
