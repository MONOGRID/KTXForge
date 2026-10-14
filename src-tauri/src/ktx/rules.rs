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
    let ctx = Context::new(sources, role);
    let sources_class = ctx.class;
    let resolution = resolve(opts, ctx, sources.len().max(1));
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
    let mut warnings = resolution.notes;
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
    let ctx = Context::new(sources, role);
    let effective = resolve(opts, ctx, sources.len()).options;
    reject_source_format_mismatch(&effective, ctx.class, &mut errors);
    reject_raw_missing_size(&effective, &mut errors);
    reject_tf_format_mismatch(&effective, &mut errors);
    reject_role_origins(&effective, role, &mut errors);
    if errors.is_empty() {
        Ok(ResolvedOptions(effective))
    } else {
        Err(errors)
    }
}

/// What the rules know about one output besides its options.
#[derive(Clone, Copy)]
struct Context {
    class: SourcesClass,
    role: InputRole,
    /// Level-0 size of the first input (`None` before any input is added).
    base: Option<(u32, u32)>,
}

impl Context {
    fn new(sources: &[SourceInfo], role: InputRole) -> Self {
        Self {
            class: classify_sources(sources),
            role,
            base: sources.first().map(|s| (s.width, s.height)),
        }
    }
}

struct Resolution {
    options: CreateOptions,
    /// `None` = enabled, `Some(reason)` = hidden in the UI and omitted from argv.
    states: Vec<(OptionKey, Option<String>)>,
    /// Values corrected to fit the source size, shown as warnings.
    notes: Vec<OptionWarning>,
}

/// Apply the role, then fit values and clear every disabled field until nothing
/// changes. Clearing one field can disable another (encode cleared for HDR → BasisLZ settings),
/// hence the loop. Each pass only moves values toward "omitted" or a smaller level count, so it
/// terminates.
fn resolve(opts: &CreateOptions, ctx: Context, input_count: usize) -> Resolution {
    let mut options = opts.clone();
    let mut notes = Vec::new();
    input_role::apply_role(&mut options, ctx.role, input_count);
    loop {
        let mut changed = fit_values(&mut options, ctx, &mut notes);
        let states: Vec<_> = OptionKey::iter()
            .map(|key| (key, disabled_reason(&options, key, ctx)))
            .collect();
        for (key, reason) in &states {
            // Role-owned keys are hidden but carry the value `apply_role` wrote.
            if reason.is_some() && !input_role::hidden_for_role(ctx.role, *key) {
                changed |= clear_field(&mut options, *key);
            }
        }
        if !changed {
            return Resolution {
                options,
                states,
                notes,
            };
        }
    }
}

fn disabled_reason(opts: &CreateOptions, key: OptionKey, ctx: Context) -> Option<String> {
    if input_role::hidden_for_role(ctx.role, key) {
        return Some("set by the input role on the Inputs panel".to_owned());
    }
    if let Some(gate) = gates(key)
        .iter()
        .find(|gate| !eval_pred(gate.pred, opts, ctx))
    {
        return Some(gate.reason.to_owned());
    }
    source_disabled_reason(key, ctx.class).map(str::to_owned)
}

/// Level-0 size `ktx create` resamples the first input to: `--scale`, else `--width` /
/// `--height` (the `--scale` gate keeps them apart).
fn target_size(opts: &CreateOptions, (width, height): (u32, u32)) -> (u32, u32) {
    let c = &opts.container;
    match c.scale {
        Some(scale) if c.width.is_none() && c.height.is_none() => {
            (scaled(width, scale), scaled(height, scale))
        }
        _ => (c.width.unwrap_or(width), c.height.unwrap_or(height)),
    }
}

/// `static_cast<uint32_t>(size * scale)`, as `ktx create` computes it.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn scaled(size: u32, scale: f32) -> u32 {
    (size as f32 * scale) as u32
}

/// Correct values `ktx create` rejects or fails on, toward what it accepts, and note each
/// correction. Returns whether anything changed.
fn fit_values(opts: &mut CreateOptions, ctx: Context, notes: &mut Vec<OptionWarning>) -> bool {
    // The resampler can end up with no samples for a pixel below 1 (box fails even at 0.9).
    if let Some(scale) = opts.filter.mipmap_filter_scale.filter(|s| *s < 1.0) {
        opts.filter.mipmap_filter_scale = Some(1.0);
        notes.push(note(
            OptionKey::MipmapFilterScale,
            format!("filter scale {scale} can leave the resampler without samples; using 1"),
        ));
        return true;
    }
    let Some(base) = ctx.base else {
        return false;
    };
    let (width, height) = target_size(opts, base);
    if let Some(scale) = opts.container.scale.filter(|_| width == 0 || height == 0) {
        opts.container.scale = None;
        notes.push(note(
            OptionKey::Scale,
            format!(
                "--scale {scale} shrinks {}×{} to nothing; encoding at the source size",
                base.0, base.1
            ),
        ));
        return true;
    }
    let max = input_role::max_mip_levels(width, height);
    let c = &mut opts.container;
    match c.levels {
        // Without a resize, a chain longer than its base is a pack error, not a correction.
        Some(levels)
            if levels > max
                && ctx.role == InputRole::MipChain
                && (c.width.is_some() || c.height.is_some() || c.scale.is_some()) =>
        {
            c.width = None;
            c.height = None;
            c.scale = None;
            notes.push(note(
                OptionKey::Scale,
                format!("a {levels}-level mip chain cannot be resampled to {width}×{height} ({max} levels at most); encoding at the source size"),
            ));
            true
        }
        Some(levels) if levels > max && c.generate_mipmap => {
            c.levels = Some(max);
            notes.push(note(
                OptionKey::Levels,
                format!("{width}×{height} has at most {max} mip levels; generating {max}"),
            ));
            true
        }
        _ => false,
    }
}

fn note(key: OptionKey, message: String) -> OptionWarning {
    OptionWarning {
        key: Some(key),
        level: WarningLevel::Info,
        message,
    }
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
    // ASTC formats are compressed by the ASTC encoder; `--encode` needs an 8-bit format.
    let encode = if class == SourcesClass::Hdr || format.is_astc() {
        None
    } else {
        opts.encode
    };
    let convert_tf = sanitize_convert_tf(format, opts.color.convert_tf);
    let assign_tf = sanitize_assign_tf(format, convert_tf, opts.color.assign_tf);
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
            OptionKey::AssignTf => TransferFunction::iter().any(|tf| {
                tf.cli() == value && assign_tf_allowed(opts.format, opts.color.convert_tf, tf)
            }),
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
    /// R8 formats: one channel.
    SingleChannelFormat,
    AssignPrimariesNone,
    AssignPrimariesSet,
    AssignTfSet,
    ConvertTfSet,
    ConvertPrimariesSet,
    ConvertOriginSet,
    /// `--assign-tf` unset or one `ktx create` can convert from.
    AssignTfDecodable,
    /// Output transfer function is linear or sRGB, the only ones `ktx create` encodes to.
    OutputTfEncodable,
    OutputLinear,
    FailOnColorConversions,
    FailOnOriginChanges,
    NoEndpointRdo,
    NoSelectorRdo,
    HdrSources,
    /// The first input resamples to 1 px tall (true while there is no input).
    TargetHeightOne,
    /// Every input level stays taller than 1 px (true while there is no input).
    LevelsTallerThanOne,
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
const NOT_ASTC_FORMAT: Pred = Pred::Not(&Pred::AstcFormat);
const NOT_NO_ENDPOINT_RDO: Pred = Pred::Not(&Pred::NoEndpointRdo);
const NOT_NO_SELECTOR_RDO: Pred = Pred::Not(&Pred::NoSelectorRdo);
const NOT_CONVERT_TF_SET: Pred = Pred::Not(&Pred::ConvertTfSet);
const NOT_CONVERT_PRIMARIES_SET: Pred = Pred::Not(&Pred::ConvertPrimariesSet);
const NOT_SIZE_SET: Pred = Pred::Not(&Pred::Any(&[Pred::WidthSet, Pred::HeightSet]));
/// Cubemap faces must stay square, so only `--scale` resizes them (raw needs a size).
const NOT_CUBEMAP_OR_RAW: Pred = Pred::Any(&[Pred::Not(&Pred::Cubemap), Pred::Raw]);
/// EXR inputs carry no primaries; converting them needs `--assign-primaries`.
const PRIMARIES_KNOWN: Pred = Pred::Any(&[Pred::Not(&Pred::HdrSources), Pred::AssignPrimariesSet]);

const BASIS_LZ_GATE: Gate = Gate {
    pred: Pred::EncodeBasisLz,
    reason: "ignored unless BasisLZ is selected",
};
const NOT_RAW_GATE: Gate = Gate {
    pred: NOT_RAW,
    reason: "not valid with raw",
};
const DECODABLE_GATE: Gate = Gate {
    pred: Pred::AssignTfDecodable,
    reason: "ktx cannot convert from the assigned transfer function",
};

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
        OptionKey::Scale => &[
            Gate {
                pred: NOT_RAW,
                reason: "cannot be used with raw",
            },
            Gate {
                pred: NOT_SIZE_SET,
                reason: "cannot be used with --width or --height",
            },
        ],
        OptionKey::Width | OptionKey::Height => &[Gate {
            pred: NOT_CUBEMAP_OR_RAW,
            reason: "cubemap faces must stay square; resize them with --scale",
        }],
        OptionKey::Levels => &[Gate {
            pred: Pred::GenerateMipmap,
            reason: "needs --generate-mipmap; an explicit mip chain is an Inputs role",
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
        OptionKey::ConvertTexcoordOrigin => &[
            NOT_RAW_GATE,
            Gate {
                pred: Pred::LevelsTallerThanOne,
                reason: "ktx crashes flipping 1-pixel-tall images (1D or a mip chain down to 1×1)",
            },
        ],
        OptionKey::ConvertTf => &[NOT_RAW_GATE, DECODABLE_GATE],
        OptionKey::Normalize => &[
            NOT_RAW_GATE,
            Gate {
                pred: Pred::OutputLinear,
                reason: "needs a linear output: not with sRGB formats or an sRGB transfer function",
            },
        ],
        OptionKey::ConvertPrimaries => &[
            NOT_RAW_GATE,
            Gate {
                pred: NOT_ASSIGN_PRIMARIES_NONE,
                reason: "not valid when assigned primaries are none",
            },
            DECODABLE_GATE,
            Gate {
                pred: Pred::OutputTfEncodable,
                reason: "needs a linear or sRGB output transfer function",
            },
            Gate {
                pred: PRIMARIES_KNOWN,
                reason: "HDR inputs carry no primaries; set --assign-primaries first",
            },
        ],
        OptionKey::FailOnColorConversions => &[
            Gate {
                pred: Pred::AssignTfSet,
                reason: "needs --assign-tf; otherwise the input's own transfer function may be converted",
            },
            Gate {
                pred: NOT_CONVERT_TF_SET,
                reason: "--convert-tf converts the input",
            },
            Gate {
                pred: NOT_CONVERT_PRIMARIES_SET,
                reason: "--convert-primaries converts the input",
            },
        ],
        OptionKey::WarnOnColorConversions => &[Gate {
            pred: Pred::Not(&Pred::FailOnColorConversions),
            reason: "mutually exclusive with --fail-on-color-conversions",
        }],
        OptionKey::FailOnOriginChanges => &[Gate {
            pred: Pred::Not(&Pred::ConvertOriginSet),
            reason: "--convert-texcoord-origin changes the origin",
        }],
        OptionKey::WarnOnOriginChanges => &[Gate {
            pred: Pred::Not(&Pred::FailOnOriginChanges),
            reason: "mutually exclusive with --fail-on-origin-changes",
        }],
        OptionKey::NormalMode => &[Gate {
            pred: Pred::Not(&Pred::SingleChannelFormat),
            reason: "needs a format with two or more channels",
        }],
        OptionKey::Encode => &[Gate {
            pred: NOT_ASTC_FORMAT,
            reason: "ASTC formats are compressed by the ASTC encoder",
        }],
        OptionKey::Qlevel | OptionKey::Clevel | OptionKey::NoEndpointRdo | OptionKey::NoSelectorRdo => {
            &[BASIS_LZ_GATE]
        }
        OptionKey::MaxEndpoints | OptionKey::EndpointRdoThreshold => &[
            BASIS_LZ_GATE,
            Gate {
                pred: NOT_NO_ENDPOINT_RDO,
                reason: "not valid while endpoint RDO is disabled",
            },
        ],
        OptionKey::MaxSelectors | OptionKey::SelectorRdoThreshold => &[
            BASIS_LZ_GATE,
            Gate {
                pred: NOT_NO_SELECTOR_RDO,
                reason: "not valid while selector RDO is disabled",
            },
        ],
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
        OptionKey::CompareSsim | OptionKey::ComparePsnr => &[
            Gate {
                pred: ENCODE_OR_ASTC_ANY,
                reason: "requires BasisLZ, UASTC, or ASTC encoding",
            },
            Gate {
                pred: Pred::Any(&[NOT_ASTC_FORMAT, NOT_DEPTH_SET]),
                reason: "ktx cannot measure ASTC 3D textures",
            },
        ],
        OptionKey::OneD => &[
            Gate {
                pred: NOT_CUBEMAP_OR_DEPTH,
                reason: "not valid with cubemap or 3D",
            },
            Gate {
                pred: NOT_ENCODE_OR_ASTC,
                reason: "BasisLZ, UASTC and ASTC files cannot be 1D",
            },
            Gate {
                pred: Pred::TargetHeightOne,
                reason: "needs 1-pixel-tall images (or --height 1)",
            },
        ],
        OptionKey::Cubemap => &[Gate {
            pred: NOT_ONE_D_OR_DEPTH,
            reason: "not valid with 1D or 3D",
        }],
        OptionKey::Depth => &[Gate {
            pred: NOT_ONE_D_OR_CUBEMAP,
            reason: "not valid with 1D or cubemap",
        }],
        OptionKey::Raw
        | OptionKey::Layers
        | OptionKey::AssignTf
        | OptionKey::AssignPrimaries
        | OptionKey::AssignTexcoordOrigin
        | OptionKey::NoWarnOnColorConversions
        | OptionKey::Swizzle
        | OptionKey::InputSwizzle
        | OptionKey::Format
        | OptionKey::Threads => &[],
    }
}

fn eval_pred(pred: Pred, opts: &CreateOptions, ctx: Context) -> bool {
    let color = &opts.color;
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
        Pred::SingleChannelFormat => matches!(opts.format, VkFormat::R8Unorm | VkFormat::R8Srgb),
        Pred::AssignPrimariesNone => color.assign_primaries == Some(ColorPrimaries::None),
        Pred::AssignPrimariesSet => color.assign_primaries.is_some(),
        Pred::AssignTfSet => color.assign_tf.is_some(),
        Pred::ConvertTfSet => color.convert_tf.is_some(),
        Pred::ConvertPrimariesSet => color.convert_primaries.is_some(),
        Pred::ConvertOriginSet => color.convert_texcoord_origin.is_some(),
        Pred::AssignTfDecodable => color.assign_tf.is_none_or(is_decodable_tf),
        Pred::OutputTfEncodable => {
            color.convert_tf.is_some()
                || color
                    .assign_tf
                    .is_none_or(|tf| tf == TransferFunction::Linear || is_srgb_assign_tf(tf))
        }
        Pred::OutputLinear => output_is_linear(opts),
        Pred::FailOnColorConversions => color.fail_on_color_conversions,
        Pred::FailOnOriginChanges => color.fail_on_origin_changes,
        Pred::NoEndpointRdo => opts.basis_lz.no_endpoint_rdo,
        Pred::NoSelectorRdo => opts.basis_lz.no_selector_rdo,
        Pred::HdrSources => ctx.class == SourcesClass::Hdr,
        Pred::TargetHeightOne => ctx.base.is_none_or(|base| target_size(opts, base).1 == 1),
        Pred::LevelsTallerThanOne => ctx.base.is_none_or(|base| {
            // Only a mip chain brings its own smaller levels; generated mips are not flipped.
            let levels = match ctx.role {
                InputRole::MipChain => opts.container.levels.unwrap_or(1),
                _ => 1,
            };
            let height = target_size(opts, base).1;
            height.checked_shr(levels.saturating_sub(1)).unwrap_or(0) > 1
        }),
        Pred::Not(inner) => !eval_pred(*inner, opts, ctx),
        Pred::Any(parts) => parts.iter().any(|p| eval_pred(*p, opts, ctx)),
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

/// `*_SRGB` formats require an sRGB assign-tf when the flag is set. With `--convert-tf` the
/// input is converted from the assigned function, so it must be decodable. Otherwise formats
/// with an sRGB twin reject an sRGB assign-tf and the ASTC encoder accepts only a linear one.
fn assign_tf_allowed(
    format: VkFormat,
    convert_tf: Option<ConvertTransferFunction>,
    tf: TransferFunction,
) -> bool {
    if format.is_srgb() {
        return is_srgb_assign_tf(tf);
    }
    if convert_tf.is_some() {
        return is_decodable_tf(tf);
    }
    if format.is_astc() {
        return tf == TransferFunction::Linear;
    }
    !(format.has_srgb_twin() && is_srgb_assign_tf(tf))
}

/// Transfer functions `ktx create` can decode when converting from them.
fn is_decodable_tf(tf: TransferFunction) -> bool {
    matches!(
        tf,
        TransferFunction::Linear
            | TransferFunction::Itu
            | TransferFunction::ItuOetf
            | TransferFunction::Bt601
            | TransferFunction::Bt601Oetf
            | TransferFunction::Bt709
            | TransferFunction::Bt709Oetf
            | TransferFunction::Bt2020
            | TransferFunction::Bt2020Oetf
            | TransferFunction::Smpte170m
            | TransferFunction::Smpte170mOetf
            | TransferFunction::Smpte170mEotf
            | TransferFunction::PqEotf
    ) || is_srgb_assign_tf(tf)
}

/// Output transfer function: `--convert-tf`, else `--assign-tf`, else the format's own.
fn output_is_linear(opts: &CreateOptions) -> bool {
    match (opts.color.convert_tf, opts.color.assign_tf) {
        (Some(tf), _) => !is_srgb_convert_tf(tf),
        (None, Some(tf)) => tf == TransferFunction::Linear,
        (None, None) => !opts.format.is_srgb(),
    }
}

fn sanitize_convert_tf(
    format: VkFormat,
    tf: Option<ConvertTransferFunction>,
) -> Option<ConvertTransferFunction> {
    tf.filter(|value| convert_tf_allowed(format, *value))
}

fn sanitize_assign_tf(
    format: VkFormat,
    convert_tf: Option<ConvertTransferFunction>,
    tf: Option<TransferFunction>,
) -> Option<TransferFunction> {
    tf.filter(|value| assign_tf_allowed(format, convert_tf, *value))
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
        if !assign_tf_allowed(opts.format, opts.color.convert_tf, tf) {
            errors.push(if opts.format.is_srgb() {
                "--assign-tf must be sRGB for *_SRGB formats".to_owned()
            } else {
                "--assign-tf sRGB is invalid for non-sRGB formats that have an sRGB twin".to_owned()
            });
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
