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
    errors.extend(input_role::pack_validation_errors(
        &input_role::validate_pack_inputs(role, sources),
    ));
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
/// libktx refuses CompressBasis / UASTC / ASTC when the texture is marked for runtime mipmaps.
const NOT_ENCODE_OR_ASTC: Pred = Pred::Not(&ENCODE_OR_ASTC_ANY);

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
        OptionKey::RuntimeMipmap => &[
            Gate {
                pred: NOT_GENERATE_MIPMAP,
                reason: "mutually exclusive with --generate-mipmap",
            },
            Gate {
                pred: NOT_ENCODE_OR_ASTC,
                reason: "not valid with BasisLZ, UASTC, or ASTC encoding",
            },
        ],
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
