//! `CreateOptions` and related enums, frozen to KTX-Software v4.4.2.
//!
//! Deferred (not present in v4.4.2): UASTC HDR codecs/flags, `--premultiply-alpha`.
//! See `docs/ktx-cli/README.md` and decision-context 2026-10-10.

use std::fmt;

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use ts_rs::TS;

/// Validated four-character swizzle from `rgba01`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct Swizzle(String);

impl Swizzle {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        if s.len() != 4 {
            return Err(format!("swizzle must be exactly 4 characters, got {s:?}"));
        }
        if !s
            .chars()
            .all(|c| matches!(c, 'r' | 'g' | 'b' | 'a' | '0' | '1'))
        {
            return Err(format!("swizzle characters must be from rgba01, got {s:?}"));
        }
        Ok(Self(s.to_owned()))
    }
}

impl<'de> Deserialize<'de> for Swizzle {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SwizzleVisitor;
        impl Visitor<'_> for SwizzleVisitor {
            type Value = Swizzle;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a 4-character swizzle from rgba01")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Swizzle::parse(v).map_err(E::custom)
            }
        }
        deserializer.deserialize_str(SwizzleVisitor)
    }
}

macro_rules! cli_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$vmeta:meta])*
                $variant:ident = $cli:literal
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, EnumIter, Display)]
        #[ts(export)]
        #[serde(rename_all = "kebab-case")]
        #[strum(serialize_all = "kebab-case")]
        $vis enum $name {
            $(
                $(#[$vmeta])*
                #[serde(rename = $cli)]
                #[strum(serialize = $cli)]
                $variant,
            )+
        }

        impl $name {
            pub fn cli(self) -> &'static str {
                match self {
                    $(Self::$variant => $cli,)+
                }
            }
        }
    };
}

cli_enum! {
    /// `--encode` values present in v4.4.2 (`basis-lz` | `uastc`).
    pub enum EncodeCodec {
        BasisLz = "basis-lz",
        /// UASTC LDR 4×4. Spec name `uastc-ldr-4x4` is deferred; CLI value is `uastc`.
        Uastc = "uastc",
    }
}

cli_enum! {
    pub enum MipmapFilter {
        Box = "box",
        Tent = "tent",
        Bell = "bell",
        BSpline = "b-spline",
        Mitchell = "mitchell",
        Blackman = "blackman",
        Lanczos3 = "lanczos3",
        Lanczos4 = "lanczos4",
        Lanczos6 = "lanczos6",
        Lanczos12 = "lanczos12",
        Kaiser = "kaiser",
        Gaussian = "gaussian",
        Catmullrom = "catmullrom",
        QuadraticInterp = "quadratic_interp",
        QuadraticApprox = "quadratic_approx",
        QuadraticMix = "quadratic_mix",
    }
}

cli_enum! {
    pub enum MipmapWrap {
        Wrap = "wrap",
        Reflect = "reflect",
        Clamp = "clamp",
    }
}

cli_enum! {
    pub enum TransferFunction {
        Linear = "linear",
        Srgb = "srgb",
        SrgbEotf = "srgb_eotf",
        Scrgb = "scrgb",
        ScrgbEotf = "scrgb_eotf",
        Itu = "itu",
        ItuOetf = "itu_oetf",
        Bt601 = "bt601",
        Bt601Oetf = "bt601_oetf",
        Bt709 = "bt709",
        Bt709Oetf = "bt709_oetf",
        Bt2020 = "bt2020",
        Bt2020Oetf = "bt2020_oetf",
        Smpte170m = "smpte170m",
        Smpte170mOetf = "smpte170m_oetf",
        Smpte170mEotf = "smpte170m_eotf",
        Ntsc = "ntsc",
        NtscEotf = "ntsc_eotf",
        Slog = "slog",
        SlogOetf = "slog_oetf",
        Slog2 = "slog2",
        Slog2Oetf = "slog2_oetf",
        Bt1886 = "bt1886",
        Bt1886Eotf = "bt1886_eotf",
        HlgOetf = "hlg_oetf",
        HlgEotf = "hlg_eotf",
        PqOetf = "pq_oetf",
        PqEotf = "pq_eotf",
        Dcip3 = "dcip3",
        Dcip3Eotf = "dcip3_eotf",
        PalOetf = "pal_oetf",
        Pal625Eotf = "pal625_eotf",
        St240 = "st240",
        St240Oetf = "st240_oetf",
        St240Eotf = "st240_eotf",
        Acescc = "acescc",
        AcesccOetf = "acescc_oetf",
        Acescct = "acescct",
        AcescctOetf = "acescct_oetf",
        Adobergb = "adobergb",
        AdobergbEotf = "adobergb_eotf",
    }
}

cli_enum! {
    pub enum ConvertTransferFunction {
        Linear = "linear",
        Srgb = "srgb",
        SrgbEotf = "srgb_eotf",
        Scrgb = "scrgb",
        ScrgbEotf = "scrgb_eotf",
    }
}

cli_enum! {
    pub enum ColorPrimaries {
        None = "none",
        Bt709 = "bt709",
        Srgb = "srgb",
        Bt601Ebu = "bt601_ebu",
        Bt601Smpte = "bt601_smpte",
        Bt2020 = "bt2020",
        Ciexyz = "ciexyz",
        Aces = "aces",
        Acescc = "acescc",
        Ntsc1953 = "ntsc1953",
        Pal525 = "pal525",
        Displayp3 = "displayp3",
        Adobergb = "adobergb",
    }
}

cli_enum! {
    pub enum ConvertPrimaries {
        Bt709 = "bt709",
        Srgb = "srgb",
        Bt601Ebu = "bt601_ebu",
        Bt601Smpte = "bt601_smpte",
        Bt2020 = "bt2020",
        Ciexyz = "ciexyz",
        Aces = "aces",
        Acescc = "acescc",
        Ntsc1953 = "ntsc1953",
        Pal525 = "pal525",
        Displayp3 = "displayp3",
        Adobergb = "adobergb",
    }
}

cli_enum! {
    pub enum TexcoordOrigin {
        TopLeft = "top-left",
        BottomLeft = "bottom-left",
        TopLeftFront = "top-left-front",
        TopLeftBack = "top-left-back",
        BottomLeftFront = "bottom-left-front",
        BottomLeftBack = "bottom-left-back",
    }
}

cli_enum! {
    pub enum AstcQuality {
        Fastest = "fastest",
        Fast = "fast",
        Medium = "medium",
        Thorough = "thorough",
        Exhaustive = "exhaustive",
    }
}

cli_enum! {
    /// Vulkan format names without `VK_FORMAT_`. ASTC uses the `_BLOCK` suffix required by v4.4.2.
    pub enum VkFormat {
        R8Unorm = "R8_UNORM",
        R8Srgb = "R8_SRGB",
        R8G8Unorm = "R8G8_UNORM",
        R8G8Srgb = "R8G8_SRGB",
        R8G8B8Unorm = "R8G8B8_UNORM",
        R8G8B8Srgb = "R8G8B8_SRGB",
        R8G8B8A8Unorm = "R8G8B8A8_UNORM",
        R8G8B8A8Srgb = "R8G8B8A8_SRGB",
        R16G16B16Sfloat = "R16G16B16_SFLOAT",
        R16G16B16A16Sfloat = "R16G16B16A16_SFLOAT",
        Astc4x4UnormBlock = "ASTC_4x4_UNORM_BLOCK",
        Astc4x4SrgbBlock = "ASTC_4x4_SRGB_BLOCK",
        Astc5x4UnormBlock = "ASTC_5x4_UNORM_BLOCK",
        Astc5x4SrgbBlock = "ASTC_5x4_SRGB_BLOCK",
        Astc5x5UnormBlock = "ASTC_5x5_UNORM_BLOCK",
        Astc5x5SrgbBlock = "ASTC_5x5_SRGB_BLOCK",
        Astc6x5UnormBlock = "ASTC_6x5_UNORM_BLOCK",
        Astc6x5SrgbBlock = "ASTC_6x5_SRGB_BLOCK",
        Astc6x6UnormBlock = "ASTC_6x6_UNORM_BLOCK",
        Astc6x6SrgbBlock = "ASTC_6x6_SRGB_BLOCK",
        Astc8x5UnormBlock = "ASTC_8x5_UNORM_BLOCK",
        Astc8x5SrgbBlock = "ASTC_8x5_SRGB_BLOCK",
        Astc8x6UnormBlock = "ASTC_8x6_UNORM_BLOCK",
        Astc8x6SrgbBlock = "ASTC_8x6_SRGB_BLOCK",
        Astc8x8UnormBlock = "ASTC_8x8_UNORM_BLOCK",
        Astc8x8SrgbBlock = "ASTC_8x8_SRGB_BLOCK",
        Astc10x5UnormBlock = "ASTC_10x5_UNORM_BLOCK",
        Astc10x5SrgbBlock = "ASTC_10x5_SRGB_BLOCK",
        Astc10x6UnormBlock = "ASTC_10x6_UNORM_BLOCK",
        Astc10x6SrgbBlock = "ASTC_10x6_SRGB_BLOCK",
        Astc10x8UnormBlock = "ASTC_10x8_UNORM_BLOCK",
        Astc10x8SrgbBlock = "ASTC_10x8_SRGB_BLOCK",
        Astc10x10UnormBlock = "ASTC_10x10_UNORM_BLOCK",
        Astc10x10SrgbBlock = "ASTC_10x10_SRGB_BLOCK",
        Astc12x10UnormBlock = "ASTC_12x10_UNORM_BLOCK",
        Astc12x10SrgbBlock = "ASTC_12x10_SRGB_BLOCK",
        Astc12x12UnormBlock = "ASTC_12x12_UNORM_BLOCK",
        Astc12x12SrgbBlock = "ASTC_12x12_SRGB_BLOCK",
    }
}

impl VkFormat {
    /// Uncompressed formats; every other variant is an ASTC block format.
    pub fn is_astc(self) -> bool {
        !matches!(
            self,
            Self::R8Unorm
                | Self::R8Srgb
                | Self::R8G8Unorm
                | Self::R8G8Srgb
                | Self::R8G8B8Unorm
                | Self::R8G8B8Srgb
                | Self::R8G8B8A8Unorm
                | Self::R8G8B8A8Srgb
                | Self::R16G16B16Sfloat
                | Self::R16G16B16A16Sfloat
        )
    }

    /// Half-float formats, the only ones that accept HDR (EXR / Radiance) input.
    pub fn is_hdr_float(self) -> bool {
        matches!(self, Self::R16G16B16Sfloat | Self::R16G16B16A16Sfloat)
    }

    pub fn is_srgb(self) -> bool {
        matches!(
            self,
            Self::R8Srgb
                | Self::R8G8Srgb
                | Self::R8G8B8Srgb
                | Self::R8G8B8A8Srgb
                | Self::Astc4x4SrgbBlock
                | Self::Astc5x4SrgbBlock
                | Self::Astc5x5SrgbBlock
                | Self::Astc6x5SrgbBlock
                | Self::Astc6x6SrgbBlock
                | Self::Astc8x5SrgbBlock
                | Self::Astc8x6SrgbBlock
                | Self::Astc8x8SrgbBlock
                | Self::Astc10x5SrgbBlock
                | Self::Astc10x6SrgbBlock
                | Self::Astc10x8SrgbBlock
                | Self::Astc10x10SrgbBlock
                | Self::Astc12x10SrgbBlock
                | Self::Astc12x12SrgbBlock
        )
    }

    /// UNORM formats with a `*_SRGB` counterpart (every 8-bit and ASTC UNORM format).
    pub fn has_srgb_twin(self) -> bool {
        !self.is_srgb() && !self.is_hdr_float()
    }
}

/// One variant per v4.4.2 `ktx create` UI flag (serde name = flag without `--`).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, EnumIter, Display,
)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum OptionKey {
    #[serde(rename = "1d")]
    #[strum(serialize = "1d")]
    OneD,
    Cubemap,
    Layers,
    Depth,
    Levels,
    GenerateMipmap,
    RuntimeMipmap,
    Width,
    Height,
    Scale,
    Raw,
    MipmapFilter,
    MipmapFilterScale,
    MipmapWrap,
    AssignTf,
    ConvertTf,
    AssignPrimaries,
    ConvertPrimaries,
    AssignTexcoordOrigin,
    ConvertTexcoordOrigin,
    FailOnColorConversions,
    WarnOnColorConversions,
    NoWarnOnColorConversions,
    FailOnOriginChanges,
    WarnOnOriginChanges,
    Swizzle,
    InputSwizzle,
    Normalize,
    NormalMode,
    Format,
    Encode,
    Qlevel,
    Clevel,
    MaxEndpoints,
    MaxSelectors,
    EndpointRdoThreshold,
    SelectorRdoThreshold,
    NoEndpointRdo,
    NoSelectorRdo,
    UastcQuality,
    UastcRdo,
    #[serde(rename = "uastc-rdo-l")]
    #[strum(serialize = "uastc-rdo-l")]
    UastcRdoL,
    #[serde(rename = "uastc-rdo-d")]
    #[strum(serialize = "uastc-rdo-d")]
    UastcRdoD,
    #[serde(rename = "uastc-rdo-b")]
    #[strum(serialize = "uastc-rdo-b")]
    UastcRdoB,
    #[serde(rename = "uastc-rdo-s")]
    #[strum(serialize = "uastc-rdo-s")]
    UastcRdoS,
    #[serde(rename = "uastc-rdo-f")]
    #[strum(serialize = "uastc-rdo-f")]
    UastcRdoF,
    #[serde(rename = "uastc-rdo-m")]
    #[strum(serialize = "uastc-rdo-m")]
    UastcRdoM,
    AstcQuality,
    AstcPerceptual,
    Threads,
    NoSse,
    CompareSsim,
    ComparePsnr,
    Zstd,
    Zlib,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ContainerOptions {
    pub one_d: bool,
    pub cubemap: bool,
    pub layers: Option<u32>,
    pub depth: Option<u32>,
    pub levels: Option<u32>,
    pub generate_mipmap: bool,
    pub runtime_mipmap: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub scale: Option<f32>,
    pub raw: bool,
}

impl Default for ContainerOptions {
    fn default() -> Self {
        Self {
            one_d: false,
            cubemap: false,
            layers: None,
            depth: None,
            levels: None,
            generate_mipmap: false,
            runtime_mipmap: false,
            width: None,
            height: None,
            scale: Some(1.0),
            raw: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FilterOptions {
    pub mipmap_filter: Option<MipmapFilter>,
    pub mipmap_filter_scale: Option<f32>,
    pub mipmap_wrap: Option<MipmapWrap>,
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            mipmap_filter: Some(MipmapFilter::Lanczos4),
            mipmap_filter_scale: Some(1.0),
            mipmap_wrap: Some(MipmapWrap::Clamp),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ColorOptions {
    pub assign_tf: Option<TransferFunction>,
    pub convert_tf: Option<ConvertTransferFunction>,
    pub assign_primaries: Option<ColorPrimaries>,
    pub convert_primaries: Option<ConvertPrimaries>,
    pub assign_texcoord_origin: Option<TexcoordOrigin>,
    pub convert_texcoord_origin: Option<TexcoordOrigin>,
    pub fail_on_color_conversions: bool,
    pub warn_on_color_conversions: bool,
    pub no_warn_on_color_conversions: bool,
    pub fail_on_origin_changes: bool,
    pub warn_on_origin_changes: bool,
    pub swizzle: Option<Swizzle>,
    pub input_swizzle: Option<Swizzle>,
    pub normalize: bool,
    pub normal_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BasisLzOptions {
    pub qlevel: Option<u8>,
    pub clevel: Option<u8>,
    pub max_endpoints: Option<u16>,
    pub max_selectors: Option<u16>,
    pub endpoint_rdo_threshold: Option<f32>,
    pub selector_rdo_threshold: Option<f32>,
    pub no_endpoint_rdo: bool,
    pub no_selector_rdo: bool,
}

impl Default for BasisLzOptions {
    fn default() -> Self {
        Self {
            qlevel: Some(128),
            clevel: Some(1),
            max_endpoints: None,
            max_selectors: None,
            endpoint_rdo_threshold: Some(1.25),
            selector_rdo_threshold: Some(1.25),
            no_endpoint_rdo: false,
            no_selector_rdo: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct UastcLdrOptions {
    pub uastc_quality: Option<u8>,
    pub uastc_rdo: bool,
    pub uastc_rdo_l: Option<f32>,
    pub uastc_rdo_d: Option<u32>,
    pub uastc_rdo_b: Option<f32>,
    pub uastc_rdo_s: Option<f32>,
    pub uastc_rdo_f: bool,
    pub uastc_rdo_m: bool,
}

impl Default for UastcLdrOptions {
    fn default() -> Self {
        Self {
            uastc_quality: Some(1),
            uastc_rdo: false,
            uastc_rdo_l: Some(1.0),
            uastc_rdo_d: Some(4096),
            uastc_rdo_b: Some(10.0),
            uastc_rdo_s: Some(18.0),
            uastc_rdo_f: false,
            uastc_rdo_m: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AstcOptions {
    pub astc_quality: Option<AstcQuality>,
    pub astc_perceptual: bool,
}

impl Default for AstcOptions {
    fn default() -> Self {
        Self {
            astc_quality: Some(AstcQuality::Medium),
            astc_perceptual: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EncoderOptions {
    pub threads: Option<u32>,
    pub no_sse: bool,
    pub compare_ssim: bool,
    pub compare_psnr: bool,
}

fn default_thread_count() -> u32 {
    std::thread::available_parallelism()
        .map(|n| u32::try_from(n.get()).unwrap_or(1))
        .unwrap_or(1)
}

impl Default for EncoderOptions {
    fn default() -> Self {
        Self {
            threads: Some(default_thread_count()),
            no_sse: false,
            compare_ssim: false,
            compare_psnr: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Supercompression {
    #[default]
    None,
    Zstd {
        level: u8,
    },
    Zlib {
        level: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CreateOptions {
    pub format: VkFormat,
    pub encode: Option<EncodeCodec>,
    pub container: ContainerOptions,
    pub filter: FilterOptions,
    pub color: ColorOptions,
    pub basis_lz: BasisLzOptions,
    pub uastc_ldr: UastcLdrOptions,
    pub astc: AstcOptions,
    pub encoder: EncoderOptions,
    pub supercompression: Supercompression,
}

impl Default for CreateOptions {
    fn default() -> Self {
        // Product default = Official > Color > UASTC RGBA (+ zstd 18).
        crate::ktx::presets::product_default_options()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AnimationOptions {
    pub duration: u32,
    pub timescale: u32,
    pub loop_count: u32,
}

impl Default for AnimationOptions {
    fn default() -> Self {
        Self {
            duration: 1,
            timescale: 10,
            loop_count: 0,
        }
    }
}

/// Warning / info attached to an option field by the rules engine.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct OptionWarning {
    pub key: Option<OptionKey>,
    pub level: WarningLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum WarningLevel {
    Info,
    Warn,
}

/// One selectable value under an enum option, with help for the select item.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct OptionChoice {
    pub value: String,
    pub help: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FieldState {
    pub key: OptionKey,
    pub enabled: bool,
    pub reason: Option<String>,
    /// What this option does (label tooltip).
    pub help: String,
    /// Per-value help for enum selects (empty when not an enum).
    pub choices: Vec<OptionChoice>,
}

/// Homogeneous source set for format / encode enablement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum SourcesClass {
    Empty,
    Ldr,
    Hdr,
    /// Both LDR and HDR inputs in one encode — not allowed.
    Mixed,
}

/// Patches Vue must apply when current options would fail `ktx create`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct OptionSuggestion {
    pub format: VkFormat,
    pub encode: Option<EncodeCodec>,
    /// Cleared when incompatible with `format` (e.g. `linear` on `*_SRGB`).
    pub convert_tf: Option<ConvertTransferFunction>,
    /// Cleared when incompatible with `format` (non-sRGB assign on `*_SRGB`).
    pub assign_tf: Option<TransferFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct OptionReport {
    pub fields: Vec<FieldState>,
    pub warnings: Vec<OptionWarning>,
    pub sources_class: SourcesClass,
    /// Uncompressed format to fall back to for these sources (fresh form, mode switch).
    pub default_format: VkFormat,
    /// When set, the current options are known-invalid for these sources; apply before encode.
    pub suggestion: Option<OptionSuggestion>,
}
