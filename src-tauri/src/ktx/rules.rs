//! Option rules engine: one gate table drives both what the UI shows and what reaches argv.
//!
//! [`describe`] reports every field's enablement for the options panel. [`resolve_for_encode`]
//! runs the same resolution and clears every field the report would hide, so a value the user
//! can no longer see is never sent to `ktx create`. Only combinations that cannot be fixed by
//! omitting a flag (format vs sources, transfer function vs format, raw without a size) are
//! hard errors.

use strum::IntoEnumIterator;

use crate::commands::{SourceInfo, SourceKind};
use crate::ktx::input_role::{self, InputRole};
use crate::ktx::option_help;
use crate::ktx::options::{
    ColorPrimaries, ConvertTransferFunction, CreateOptions, EncodeCodec, FieldState, OptionChoice,
    OptionKey, OptionReport, OptionSuggestion, OptionWarning, SourcesClass, Supercompression,
    TexcoordOrigin, TransferFunction, VkFormat, WarningLevel,
};

/// Options after role application and pruning. Only [`resolve_for_encode`] builds one, so
/// argv can only be emitted from options that passed the rules.
#[derive(Debug, Clone)]
pub struct ResolvedOptions(CreateOptions);

impl ResolvedOptions {
    pub fn options(&self) -> &CreateOptions {
        &self.0
    }
}

/// Return a [`FieldState`] for every [`OptionKey`], plus color / encode warnings.
///
/// Filters format / encode choices from the source set so the UI cannot pick a combination
/// known to fail `ktx create`. When current options are invalid, [`OptionReport::suggestion`]
/// tells Vue what to apply.
pub fn describe(opts: &CreateOptions, sources: &[SourceInfo], role: InputRole) -> OptionReport {
    let sources_class = classify_sources(sources);
    let resolution = resolve(opts, sources_class, role, sources.len().max(1));
    let effective = &resolution.options;
    let fields = resolution
        .states
        .into_iter()
        .map(|(key, reason)| FieldState {
            key,
            enabled: reason.is_none(),
            reason,
            help: option_help::help_for(key).to_owned(),
            choices: filter_choices(
                key,
                effective,
                sources_class,
                role,
                option_help::choices_for(key),
            ),
        })
        .collect();
    let mut warnings = Vec::new();
    warn_nonlinear_mipmap(effective, &mut warnings);
    warn_encode_color_space(effective, &mut warnings);
    warn_srgb_to_linear_lossy(effective, &mut warnings);
    warn_mixed_sources(sources_class, &mut warnings);
    if let Err(msg) = input_role::validate_arity(role, sources.len()) {
        if !sources.is_empty() {
            warnings.push(OptionWarning {
                key: None,
                level: WarningLevel::Warn,
                message: msg,
            });
        }
    }
    OptionReport {
        fields,
        warnings,
        sources_class,
        default_format: default_format_for(sources_class),
        suggestion: suggestion_for(opts, sources_class),
    }
}

/// Resolve options for one output and refuse what cannot be fixed by omitting a flag.
pub fn resolve_for_encode(
    opts: &CreateOptions,
    sources: &[SourceInfo],
    role: InputRole,
) -> Result<ResolvedOptions, Vec<String>> {
    let mut errors = Vec::new();
    if let Err(e) = input_role::validate_arity(role, sources.len()) {
        errors.push(e);
    }
    let class = classify_sources(sources);
    let effective = resolve(opts, class, role, sources.len()).options;
    reject_source_format_mismatch(&effective, class, &mut errors);
    reject_raw_missing_size(&effective, &mut errors);
    reject_tf_format_mismatch(&effective, &mut errors);
    reject_role_origins(&effective, role, &mut errors);
    if errors.is_empty() {
        Ok(ResolvedOptions(effective))
    } else {
        Err(errors)
    }
}

struct Resolution {
    options: CreateOptions,
    /// `None` = enabled, `Some(reason)` = hidden in the UI and omitted from argv.
    states: Vec<(OptionKey, Option<String>)>,
}

/// Apply the role, then clear every disabled field until nothing changes. Clearing one field
/// can disable another (encode cleared for HDR → BasisLZ settings), hence the loop. Each pass
/// only moves values toward "omitted", so it terminates.
fn resolve(
    opts: &CreateOptions,
    class: SourcesClass,
    role: InputRole,
    input_count: usize,
) -> Resolution {
    let mut options = opts.clone();
    input_role::apply_role(&mut options, role, input_count);
    loop {
        let states: Vec<_> = OptionKey::iter()
            .map(|key| (key, disabled_reason(&options, key, class, role)))
            .collect();
        let mut changed = false;
        for (key, reason) in &states {
            // Role-owned keys are hidden but carry the value `apply_role` wrote.
            if reason.is_some() && !input_role::hidden_for_role(role, *key) {
                changed |= clear_field(&mut options, *key);
            }
        }
        if !changed {
            return Resolution { options, states };
        }
    }
}

fn disabled_reason(
    opts: &CreateOptions,
    key: OptionKey,
    class: SourcesClass,
    role: InputRole,
) -> Option<String> {
    if input_role::hidden_for_role(role, key) {
        return Some("set by the input role on the Inputs panel".to_owned());
    }
    if let Some(gate) = gates(key).iter().find(|gate| !eval_pred(gate.pred, opts)) {
        return Some(gate.reason.to_owned());
    }
    source_disabled_reason(key, class).map(str::to_owned)
}

fn source_disabled_reason(key: OptionKey, class: SourcesClass) -> Option<&'static str> {
    const MIXED: &str = "cannot encode mixed LDR and HDR sources together";
    match (key, class) {
        (OptionKey::Encode, SourcesClass::Hdr) => Some(
            "BasisLZ and UASTC LDR require 8-bit input; HDR sources need an uncompressed float format",
        ),
        (OptionKey::AstcQuality | OptionKey::AstcPerceptual, SourcesClass::Hdr) => {
            Some("ASTC is LDR-only; HDR sources need an uncompressed float format")
        }
        (OptionKey::CompareSsim | OptionKey::ComparePsnr, SourcesClass::Hdr) => {
            Some("compare metrics require BasisLZ, UASTC, or ASTC")
        }
        (
            OptionKey::Encode
            | OptionKey::AstcQuality
            | OptionKey::AstcPerceptual
            | OptionKey::CompareSsim
            | OptionKey::ComparePsnr,
            SourcesClass::Mixed,
        ) => Some(MIXED),
        _ => None,
    }
}

/// Reset `key` to its "flag omitted" value. Returns whether anything changed.
fn clear_field(opts: &mut CreateOptions, key: OptionKey) -> bool {
    fn off(flag: &mut bool) -> bool {
        std::mem::replace(flag, false)
    }
    fn none<T>(value: &mut Option<T>) -> bool {
        value.take().is_some()
    }
    let c = &mut opts.container;
    let f = &mut opts.filter;
    let color = &mut opts.color;
    let b = &mut opts.basis_lz;
    let u = &mut opts.uastc_ldr;
    let e = &mut opts.encoder;
    match key {
        OptionKey::OneD => off(&mut c.one_d),
        OptionKey::Cubemap => off(&mut c.cubemap),
        OptionKey::Layers => none(&mut c.layers),
        OptionKey::Depth => none(&mut c.depth),
        OptionKey::Levels => none(&mut c.levels),
        OptionKey::GenerateMipmap => off(&mut c.generate_mipmap),
        OptionKey::RuntimeMipmap => off(&mut c.runtime_mipmap),
        OptionKey::Width => none(&mut c.width),
        OptionKey::Height => none(&mut c.height),
        OptionKey::Scale => none(&mut c.scale),
        OptionKey::Raw => off(&mut c.raw),
        OptionKey::MipmapFilter => none(&mut f.mipmap_filter),
        OptionKey::MipmapFilterScale => none(&mut f.mipmap_filter_scale),
        OptionKey::MipmapWrap => none(&mut f.mipmap_wrap),
        OptionKey::AssignTf => none(&mut color.assign_tf),
        OptionKey::ConvertTf => none(&mut color.convert_tf),
        OptionKey::AssignPrimaries => none(&mut color.assign_primaries),
        OptionKey::ConvertPrimaries => none(&mut color.convert_primaries),
        OptionKey::AssignTexcoordOrigin => none(&mut color.assign_texcoord_origin),
        OptionKey::ConvertTexcoordOrigin => none(&mut color.convert_texcoord_origin),
        OptionKey::FailOnColorConversions => off(&mut color.fail_on_color_conversions),
        OptionKey::WarnOnColorConversions => off(&mut color.warn_on_color_conversions),
        OptionKey::NoWarnOnColorConversions => off(&mut color.no_warn_on_color_conversions),
        OptionKey::FailOnOriginChanges => off(&mut color.fail_on_origin_changes),
        OptionKey::WarnOnOriginChanges => off(&mut color.warn_on_origin_changes),
        OptionKey::Swizzle => none(&mut color.swizzle),
        OptionKey::InputSwizzle => none(&mut color.input_swizzle),
        OptionKey::Normalize => off(&mut color.normalize),
        OptionKey::NormalMode => off(&mut color.normal_mode),
        // `--format` is always required; incompatible formats go through the suggestion.
        OptionKey::Format => false,
        OptionKey::Encode => none(&mut opts.encode),
        OptionKey::Qlevel => none(&mut b.qlevel),
        OptionKey::Clevel => none(&mut b.clevel),
        OptionKey::MaxEndpoints => none(&mut b.max_endpoints),
        OptionKey::MaxSelectors => none(&mut b.max_selectors),
        OptionKey::EndpointRdoThreshold => none(&mut b.endpoint_rdo_threshold),
        OptionKey::SelectorRdoThreshold => none(&mut b.selector_rdo_threshold),
        OptionKey::NoEndpointRdo => off(&mut b.no_endpoint_rdo),
        OptionKey::NoSelectorRdo => off(&mut b.no_selector_rdo),
        OptionKey::UastcQuality => none(&mut u.uastc_quality),
        OptionKey::UastcRdo => off(&mut u.uastc_rdo),
        OptionKey::UastcRdoL => none(&mut u.uastc_rdo_l),
        OptionKey::UastcRdoD => none(&mut u.uastc_rdo_d),
        OptionKey::UastcRdoB => none(&mut u.uastc_rdo_b),
        OptionKey::UastcRdoS => none(&mut u.uastc_rdo_s),
        OptionKey::UastcRdoF => off(&mut u.uastc_rdo_f),
        OptionKey::UastcRdoM => off(&mut u.uastc_rdo_m),
        OptionKey::AstcQuality => none(&mut opts.astc.astc_quality),
        OptionKey::AstcPerceptual => off(&mut opts.astc.astc_perceptual),
        OptionKey::Threads => none(&mut e.threads),
        OptionKey::NoSse => off(&mut e.no_sse),
        OptionKey::CompareSsim => off(&mut e.compare_ssim),
        OptionKey::ComparePsnr => off(&mut e.compare_psnr),
        OptionKey::Zstd => clear_scheme(&mut opts.supercompression, |s| {
            matches!(s, Supercompression::Zstd { .. })
        }),
        OptionKey::Zlib => clear_scheme(&mut opts.supercompression, |s| {
            matches!(s, Supercompression::Zlib { .. })
        }),
    }
}

fn clear_scheme(value: &mut Supercompression, is_scheme: fn(&Supercompression) -> bool) -> bool {
    if is_scheme(value) {
        *value = Supercompression::None;
        true
    } else {
        false
    }
}

pub fn classify_sources(sources: &[SourceInfo]) -> SourcesClass {
    let saw_ldr = sources.iter().any(|s| s.kind == SourceKind::Ldr);
    let saw_hdr = sources.iter().any(|s| s.kind == SourceKind::Hdr);
    match (saw_ldr, saw_hdr) {
        (true, true) => SourcesClass::Mixed,
        (false, true) => SourcesClass::Hdr,
        (true, false) => SourcesClass::Ldr,
        (false, false) => SourcesClass::Empty,
    }
}

fn format_allowed_for_class(format: VkFormat, class: SourcesClass) -> bool {
    match class {
        SourcesClass::Empty | SourcesClass::Mixed => true,
        SourcesClass::Hdr => format.is_hdr_float(),
        SourcesClass::Ldr => !format.is_hdr_float(),
    }
}

/// Source-aware default format: what a fresh form or an incompatible format resets to.
fn default_format_for(class: SourcesClass) -> VkFormat {
    match class {
        SourcesClass::Hdr => VkFormat::R16G16B16A16Sfloat,
        SourcesClass::Empty | SourcesClass::Ldr | SourcesClass::Mixed => VkFormat::R8G8B8A8Srgb,
    }
}

fn suggestion_for(opts: &CreateOptions, class: SourcesClass) -> Option<OptionSuggestion> {
    let format = if format_allowed_for_class(opts.format, class) {
        opts.format
    } else {
        default_format_for(class)
    };
    let encode = if class == SourcesClass::Hdr {
        None
    } else {
        opts.encode
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

fn filter_choices(
    key: OptionKey,
    opts: &CreateOptions,
    class: SourcesClass,
    role: InputRole,
    choices: Vec<OptionChoice>,
) -> Vec<OptionChoice> {
    let keep = |choice: &OptionChoice| -> bool {
        let value = choice.value.as_str();
        match key {
            OptionKey::Format => {
                VkFormat::iter().any(|f| f.cli() == value && format_allowed_for_class(f, class))
            }
            OptionKey::Encode => source_disabled_reason(key, class).is_none(),
            OptionKey::ConvertTf => ConvertTransferFunction::iter()
                .any(|tf| tf.cli() == value && convert_tf_allowed(opts.format, tf)),
            OptionKey::AssignTf => TransferFunction::iter()
                .any(|tf| tf.cli() == value && assign_tf_allowed(opts.format, tf)),
            OptionKey::AssignTexcoordOrigin | OptionKey::ConvertTexcoordOrigin => {
                TexcoordOrigin::iter()
                    .any(|o| o.cli() == value && input_role::origin_allowed(role, o))
            }
            _ => true,
        }
    };
    choices.into_iter().filter(keep).collect()
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
const NOT_DEPTH_SET: Pred = Pred::Not(&Pred::DepthSet);
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

/// SINT / UINT formats (also invalid with `--generate-mipmap`) are not in [`VkFormat`].
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
        Pred::AstcFormat => opts.format.is_astc(),
        Pred::AssignPrimariesNone => opts.color.assign_primaries == Some(ColorPrimaries::None),
        Pred::Not(inner) => !eval_pred(*inner, opts),
        Pred::Any(parts) => parts.iter().any(|p| eval_pred(*p, opts)),
    }
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
    if format.is_srgb() {
        is_srgb_convert_tf(tf)
    } else if format.has_srgb_twin() {
        matches!(tf, ConvertTransferFunction::Linear)
    } else {
        true
    }
}

/// `*_SRGB` formats require an sRGB assign-tf when the flag is set.
fn assign_tf_allowed(format: VkFormat, tf: TransferFunction) -> bool {
    !format.is_srgb() || is_srgb_assign_tf(tf)
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
    !matches!(tf, TransferFunction::Linear) && !is_srgb_assign_tf(tf)
}

fn effective_output_tf(opts: &CreateOptions) -> Option<TransferFunction> {
    if let Some(tf) = opts.color.convert_tf {
        return Some(if is_srgb_convert_tf(tf) {
            TransferFunction::Srgb
        } else {
            TransferFunction::Linear
        });
    }
    opts.color.assign_tf
}

// "Mip generation is avoided when the output transfer function is non-linear and is not sRGB."
fn warn_nonlinear_mipmap(opts: &CreateOptions, warnings: &mut Vec<OptionWarning>) {
    if !opts.container.generate_mipmap {
        return;
    }
    if effective_output_tf(opts).is_some_and(is_nonlinear_non_srgb) {
        warnings.push(OptionWarning {
            key: Some(OptionKey::GenerateMipmap),
            level: WarningLevel::Warn,
            message: "mip generation is avoided when the output transfer function is non-linear and is not sRGB; the resampler only decodes sRGB".to_owned(),
        });
    }
}

// "Encoding to ASTC, BasisLZ, or UASTC is discouraged unless the data is linear or sRGB."
fn warn_encode_color_space(opts: &CreateOptions, warnings: &mut Vec<OptionWarning>) {
    let encoding = opts.encode.is_some() || opts.format.is_astc();
    if encoding && effective_output_tf(opts).is_some_and(is_nonlinear_non_srgb) {
        warnings.push(OptionWarning {
            key: Some(OptionKey::Encode),
            level: WarningLevel::Warn,
            message: "encoding to ASTC, BasisLZ, or UASTC is discouraged unless the data is linear or sRGB".to_owned(),
        });
    }
}

// "sRGB to linear is treated as visually lossy."
fn warn_srgb_to_linear_lossy(opts: &CreateOptions, warnings: &mut Vec<OptionWarning>) {
    let from_srgb = opts.color.assign_tf.is_some_and(is_srgb_assign_tf);
    let to_linear = matches!(opts.color.convert_tf, Some(ConvertTransferFunction::Linear));
    if from_srgb && to_linear {
        warnings.push(OptionWarning {
            key: Some(OptionKey::ConvertTf),
            level: WarningLevel::Info,
            message: "sRGB to linear is treated as visually lossy".to_owned(),
        });
    }
}

// --- hard rejects: cannot be fixed by omitting a flag ---

fn reject_source_format_mismatch(
    opts: &CreateOptions,
    class: SourcesClass,
    errors: &mut Vec<String>,
) {
    match class {
        SourcesClass::Empty => {}
        SourcesClass::Mixed => errors.push("sources must be all LDR or all HDR".to_owned()),
        SourcesClass::Hdr if !opts.format.is_hdr_float() => {
            errors.push("HDR sources require R16G16B16_SFLOAT or R16G16B16A16_SFLOAT".to_owned());
        }
        SourcesClass::Ldr if opts.format.is_hdr_float() => errors.push(
            "LDR sources cannot use half-float formats; use an 8-bit or ASTC format".to_owned(),
        ),
        SourcesClass::Hdr | SourcesClass::Ldr => {}
    }
}

fn reject_raw_missing_size(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "Width and height required" for raw
    if opts.container.raw && (opts.container.width.is_none() || opts.container.height.is_none()) {
        errors.push("--width and --height are required for --raw".to_owned());
    }
}

fn reject_tf_format_mismatch(opts: &CreateOptions, errors: &mut Vec<String>) {
    // "*_SRGB formats require an sRGB output transfer function.
    //  Non-sRGB formats that have an sRGB twin reject an sRGB output transfer function."
    if let Some(tf) = opts.color.convert_tf {
        if !convert_tf_allowed(opts.format, tf) {
            errors.push(if opts.format.is_srgb() {
                "--convert-tf must be sRGB for *_SRGB formats (linear is invalid)".to_owned()
            } else {
                "--convert-tf sRGB is invalid for non-sRGB formats that have an sRGB twin"
                    .to_owned()
            });
        }
    }
    if let Some(tf) = opts.color.assign_tf {
        if !assign_tf_allowed(opts.format, tf) {
            errors.push("--assign-tf must be sRGB for *_SRGB formats".to_owned());
        }
    }
}

/// Backstop: [`input_role::apply_role`] already constrains origins for cubemaps and 3D.
fn reject_role_origins(opts: &CreateOptions, role: InputRole, errors: &mut Vec<String>) {
    let color = &opts.color;
    let bad = [color.assign_texcoord_origin, color.convert_texcoord_origin]
        .into_iter()
        .flatten()
        .any(|origin| !input_role::origin_allowed(role, origin));
    let depth_unset = role == InputRole::DepthSlices
        && color.assign_texcoord_origin.is_none()
        && color.convert_texcoord_origin.is_none();
    if bad || depth_unset {
        errors.push(match role {
            InputRole::DepthSlices => {
                "--depth requires a front/back texture-coordinate origin".to_owned()
            }
            _ => "cubemap texture-coordinate origin must be top-left".to_owned(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ktx::options::*;

    /// Baseline without the product Color UASTC default (tests that assert “no encode”).
    fn opts() -> CreateOptions {
        CreateOptions {
            encode: None,
            supercompression: Supercompression::None,
            ..CreateOptions::default()
        }
    }

    fn source(kind: SourceKind) -> SourceInfo {
        let name = match kind {
            SourceKind::Ldr => "a.png",
            SourceKind::Hdr => "a.exr",
        };
        SourceInfo {
            path: name.into(),
            name: name.into(),
            kind,
            width: 64,
            height: 64,
            needs_conversion: false,
        }
    }

    fn ldr_sources(n: usize) -> Vec<SourceInfo> {
        vec![source(SourceKind::Ldr); n]
    }

    fn report(o: &CreateOptions) -> OptionReport {
        describe(o, &[], InputRole::Base2D)
    }

    fn report_role(o: &CreateOptions, role: InputRole) -> OptionReport {
        describe(o, &[], role)
    }

    /// Resolve a single LDR Base 2D output.
    fn check(o: &CreateOptions) -> Result<CreateOptions, Vec<String>> {
        check_role(o, InputRole::Base2D, 1)
    }

    fn check_role(
        o: &CreateOptions,
        role: InputRole,
        n: usize,
    ) -> Result<CreateOptions, Vec<String>> {
        resolve_for_encode(o, &ldr_sources(n), role).map(|r| r.0)
    }

    fn field(report: &OptionReport, key: OptionKey) -> &FieldState {
        report
            .fields
            .iter()
            .find(|f| f.key == key)
            .expect("field present")
    }

    fn raw_8x8() -> CreateOptions {
        let mut o = opts();
        o.container.raw = true;
        o.container.width = Some(8);
        o.container.height = Some(8);
        o
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

    /// The core invariant: a field the report hides is never emitted.
    #[test]
    fn every_hidden_field_is_omitted_on_encode() {
        let mut everything = opts();
        everything.encode = Some(EncodeCodec::BasisLz);
        everything.supercompression = Supercompression::Zstd { level: 3 };
        everything.encoder.compare_ssim = true;
        everything.encoder.no_sse = true;
        everything.uastc_ldr.uastc_rdo = true;
        everything.uastc_ldr.uastc_rdo_f = true;
        everything.astc.astc_perceptual = true;
        everything.container.generate_mipmap = true;
        everything.container.one_d = true;
        everything.color.normalize = true;
        everything.color.convert_primaries = Some(ConvertPrimaries::Srgb);
        everything.color.assign_primaries = Some(ColorPrimaries::None);

        let mut raw = everything.clone();
        raw.container.raw = true;
        raw.container.width = Some(8);
        raw.container.height = Some(8);
        let mut uastc = everything.clone();
        uastc.encode = Some(EncodeCodec::Uastc);

        let cases = [
            (everything.clone(), InputRole::Base2D, 1),
            (raw, InputRole::Base2D, 1),
            (uastc, InputRole::ArrayLayers, 3),
            (everything.clone(), InputRole::CubemapFaces, 6),
            (everything.clone(), InputRole::MipChain, 4),
            (everything, InputRole::DepthSlices, 4),
        ];
        for (o, role, n) in cases {
            let sources = ldr_sources(n);
            let r = describe(&o, &sources, role);
            let mut resolved = resolve_for_encode(&o, &sources, role)
                .unwrap_or_else(|e| panic!("{role:?}: {e:?}"))
                .0;
            for f in r.fields.iter().filter(|f| !f.enabled) {
                if input_role::hidden_for_role(role, f.key) {
                    continue;
                }
                assert!(
                    !clear_field(&mut resolved, f.key),
                    "{role:?}: hidden {:?} still set",
                    f.key
                );
            }
        }
    }

    #[test]
    fn compare_metrics_dropped_without_encoding() {
        // "--compare-ssim can only be used with BasisLZ, UASTC or ASTC encoding"
        let mut o = opts();
        o.encoder.compare_ssim = true;
        o.encoder.compare_psnr = true;
        assert!(!field(&report(&o), OptionKey::CompareSsim).enabled);
        let resolved = check(&o).unwrap();
        assert!(!resolved.encoder.compare_ssim);
        assert!(!resolved.encoder.compare_psnr);
        o.encode = Some(EncodeCodec::Uastc);
        assert!(field(&report(&o), OptionKey::CompareSsim).enabled);
        assert!(check(&o).unwrap().encoder.compare_ssim);
    }

    #[test]
    fn zstd_dropped_with_basis_lz() {
        // "--zstd … not valid with BasisLZ"
        let mut o = opts();
        o.encode = Some(EncodeCodec::BasisLz);
        assert!(!field(&report(&o), OptionKey::Zstd).enabled);
        o.supercompression = Supercompression::Zstd { level: 3 };
        assert_eq!(check(&o).unwrap().supercompression, Supercompression::None);
    }

    #[test]
    fn zlib_dropped_with_basis_lz() {
        // "--zlib … not valid with BasisLZ"
        let mut o = opts();
        o.encode = Some(EncodeCodec::BasisLz);
        assert!(!field(&report(&o), OptionKey::Zlib).enabled);
        o.supercompression = Supercompression::Zlib { level: 6 };
        assert_eq!(check(&o).unwrap().supercompression, Supercompression::None);
    }

    #[test]
    fn zstd_kept_with_uastc() {
        let mut o = opts();
        o.encode = Some(EncodeCodec::Uastc);
        o.supercompression = Supercompression::Zstd { level: 3 };
        assert!(field(&report(&o), OptionKey::Zstd).enabled);
        assert_eq!(
            check(&o).unwrap().supercompression,
            Supercompression::Zstd { level: 3 }
        );
    }

    #[test]
    fn generate_mipmap_dropped_with_raw() {
        // "Not valid for … or raw"
        let mut o = raw_8x8();
        assert!(!field(&report(&o), OptionKey::GenerateMipmap).enabled);
        o.container.generate_mipmap = true;
        assert!(!check(&o).unwrap().container.generate_mipmap);
    }

    #[test]
    fn generate_mipmap_dropped_for_3d() {
        // "Not valid for … 3D …" — depth is owned by InputRole::DepthSlices
        let mut o = opts();
        o.container.generate_mipmap = true;
        let r = describe(&o, &ldr_sources(4), InputRole::DepthSlices);
        assert!(!field(&r, OptionKey::GenerateMipmap).enabled);
        let resolved = check_role(&o, InputRole::DepthSlices, 4).unwrap();
        assert!(!resolved.container.generate_mipmap);
        assert_eq!(resolved.container.depth, Some(4));
    }

    #[test]
    fn runtime_and_generate_never_both_emitted() {
        // "Mutually exclusive with --generate-mipmap"
        let mut o = opts();
        o.container.generate_mipmap = true;
        assert!(!field(&report(&o), OptionKey::RuntimeMipmap).enabled);
        o.container.runtime_mipmap = true;
        let c = check(&o).unwrap().container;
        assert!(!(c.generate_mipmap && c.runtime_mipmap));
    }

    #[test]
    fn scale_and_filters_dropped_with_raw() {
        let o = raw_8x8();
        assert!(!field(&report(&o), OptionKey::Scale).enabled);
        let resolved = check(&o).unwrap();
        assert_eq!(resolved.container.scale, None);
        assert_eq!(resolved.filter.mipmap_filter, None);
    }

    #[test]
    fn raw_requires_width_height() {
        // "Width and height required"
        let mut o = opts();
        o.container.raw = true;
        assert!(check(&o).is_err());
        assert!(check(&raw_8x8()).is_ok());
    }

    #[test]
    fn color_conversions_dropped_with_raw() {
        // "Not valid with raw"
        let mut o = raw_8x8();
        for key in [
            OptionKey::ConvertTf,
            OptionKey::ConvertPrimaries,
            OptionKey::ConvertTexcoordOrigin,
            OptionKey::Normalize,
        ] {
            assert!(!field(&report(&o), key).enabled, "{key:?}");
        }
        o.format = VkFormat::R8G8B8A8Unorm;
        o.color.convert_tf = Some(ConvertTransferFunction::Linear);
        o.color.convert_primaries = Some(ConvertPrimaries::Srgb);
        o.color.convert_texcoord_origin = Some(TexcoordOrigin::BottomLeft);
        o.color.normalize = true;
        let color = check(&o).unwrap().color;
        assert_eq!(color.convert_tf, None);
        assert_eq!(color.convert_primaries, None);
        assert_eq!(color.convert_texcoord_origin, None);
        assert!(!color.normalize);
    }

    #[test]
    fn convert_primaries_dropped_when_assign_none() {
        // "not valid when assigned primaries are none"
        let mut o = opts();
        o.color.assign_primaries = Some(ColorPrimaries::None);
        assert!(!field(&report(&o), OptionKey::ConvertPrimaries).enabled);
        o.color.convert_primaries = Some(ConvertPrimaries::Srgb);
        assert_eq!(check(&o).unwrap().color.convert_primaries, None);
    }

    #[test]
    fn depth_gets_front_back_origin() {
        // "Requires a front/back texture-coordinate origin"
        let mut o = opts();
        o.color.assign_texcoord_origin = Some(TexcoordOrigin::BottomLeft);
        let resolved = check_role(&o, InputRole::DepthSlices, 4).unwrap();
        assert_eq!(
            resolved.color.assign_texcoord_origin,
            Some(TexcoordOrigin::TopLeftFront)
        );
        let r = describe(&o, &ldr_sources(4), InputRole::DepthSlices);
        let origins: Vec<_> = field(&r, OptionKey::AssignTexcoordOrigin)
            .choices
            .iter()
            .map(|c| c.value.as_str())
            .collect();
        assert!(!origins.contains(&"top-left"));
        assert!(origins.contains(&"bottom-left-back"));
    }

    #[test]
    fn cubemap_origin_must_be_top_left() {
        // "Cubemaps must stay top-left"
        let mut o = opts();
        o.color.assign_texcoord_origin = Some(TexcoordOrigin::BottomLeft);
        o.color.convert_texcoord_origin = Some(TexcoordOrigin::BottomLeft);
        let color = check_role(&o, InputRole::CubemapFaces, 6).unwrap().color;
        assert_eq!(color.assign_texcoord_origin, Some(TexcoordOrigin::TopLeft));
        assert_eq!(color.convert_texcoord_origin, None);
        assert!(check_role(&o, InputRole::CubemapFaces, 5).is_err());
    }

    #[test]
    fn levels_stay_user_owned_outside_mip_chain() {
        // Leaving the mip-chain role must not keep its derived --levels.
        let o = opts();
        assert_eq!(
            check_role(&o, InputRole::MipChain, 3)
                .unwrap()
                .container
                .levels,
            Some(3)
        );
        assert_eq!(check(&o).unwrap().container.levels, None);
    }

    #[test]
    fn mipmap_filters_ignored_unless_resampling() {
        // "Ignored unless --generate-mipmap, --scale, --width or --height"
        // Default scale is 1.0, so filters start enabled; clear scale to test the gate.
        let mut o = opts();
        o.container.scale = None;
        assert!(!field(&report(&o), OptionKey::MipmapFilter).enabled);
        assert_eq!(check(&o).unwrap().filter.mipmap_filter, None);
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
        assert_eq!(check(&o).unwrap().basis_lz.qlevel, None);
        let mut o = opts();
        o.encode = Some(EncodeCodec::BasisLz);
        assert!(field(&report(&o), OptionKey::Qlevel).enabled);
        assert_eq!(check(&o).unwrap().basis_lz.qlevel, Some(128));
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
        assert_eq!(check(&o).unwrap().uastc_ldr.uastc_rdo_l, None);
        o.uastc_ldr.uastc_rdo = true;
        assert!(field(&report(&o), OptionKey::UastcRdoL).enabled);
        assert_eq!(check(&o).unwrap().uastc_ldr.uastc_rdo_l, Some(1.0));
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
    fn one_d_not_with_cubemap() {
        let mut o = opts();
        o.container.one_d = true;
        assert!(!field(&report_role(&o, InputRole::CubemapFaces), OptionKey::OneD).enabled);
        assert!(
            !check_role(&o, InputRole::CubemapFaces, 6)
                .unwrap()
                .container
                .one_d
        );
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
        assert!(check(&o).is_err());
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
        assert!(check(&o).is_err());
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
        assert!(check(&o).is_err());
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
        assert!(check(&o).is_ok());
    }

    #[test]
    fn hdr_suggests_rgba16_float_and_hides_ldr_formats() {
        let o = opts();
        let hdr = [source(SourceKind::Hdr)];
        let r = describe(&o, &hdr, InputRole::Base2D);
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
        assert!(field(&r, OptionKey::Encode).choices.is_empty());
        assert!(resolve_for_encode(&o, &hdr, InputRole::Base2D).is_err());
    }

    #[test]
    fn hdr_drops_encode_and_its_settings() {
        // Clearing encode disables the BasisLZ fields too (resolution loops to a fixed point).
        let mut o = opts();
        o.format = VkFormat::R16G16B16A16Sfloat;
        o.encode = Some(EncodeCodec::BasisLz);
        let hdr = [source(SourceKind::Hdr)];
        let resolved = resolve_for_encode(&o, &hdr, InputRole::Base2D).unwrap().0;
        assert_eq!(resolved.encode, None);
        assert_eq!(resolved.basis_lz.qlevel, None);
        let r = describe(&o, &hdr, InputRole::Base2D);
        assert!(!field(&r, OptionKey::Qlevel).enabled);
    }

    #[test]
    fn hdr_compatible_options_have_no_suggestion() {
        let mut o = opts();
        o.format = VkFormat::R16G16B16A16Sfloat;
        let hdr = [source(SourceKind::Hdr)];
        let r = describe(&o, &hdr, InputRole::Base2D);
        assert!(r.suggestion.is_none());
        assert!(resolve_for_encode(&o, &hdr, InputRole::Base2D).is_ok());
    }

    #[test]
    fn ldr_hides_float_formats_and_rejects_float_selection() {
        let mut o = opts();
        o.format = VkFormat::R16G16B16A16Sfloat;
        let r = describe(&o, &ldr_sources(1), InputRole::Base2D);
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
        assert!(check(&o).is_err());
    }

    #[test]
    fn mixed_sources_warn_and_fail_resolve() {
        let o = opts();
        let mixed = [source(SourceKind::Ldr), source(SourceKind::Hdr)];
        let r = describe(&o, &mixed, InputRole::ArrayLayers);
        assert_eq!(r.sources_class, SourcesClass::Mixed);
        assert!(r.suggestion.is_none());
        assert!(r.warnings.iter().any(|w| w.message.contains("all LDR")));
        assert!(resolve_for_encode(&o, &mixed, InputRole::ArrayLayers).is_err());
    }
}
