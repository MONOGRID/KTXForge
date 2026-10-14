//! Official encode presets (Handbrake-style recipes).
//!
//! Presets own format / encode / color TF+normal flags / codec blocks /
//! supercompression. They never touch `container`, `filter`, or `encoder`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::options::{
    AstcOptions, BasisLzOptions, CreateOptions, EncodeCodec, Supercompression, TransferFunction,
    UastcLdrOptions, VkFormat,
};

/// Product default: Official > Color > UASTC RGBA.
pub const DEFAULT_ENCODE_PRESET_ID: &str = "official.color.uastc-rgba";

const ZSTD_DEFAULT_LEVEL: u8 = 18;
const NORMAL_UASTC_RDO_L: f32 = 0.5;

/// Owned encode fields written by an Official preset (for catalog + Custom detect).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EncodePresetOwned {
    pub format: VkFormat,
    pub encode: Option<EncodeCodec>,
    pub normalize: bool,
    pub normal_mode: bool,
    /// `linear` for UNORM presets: the bytes are data, so an 8-bit PNG that
    /// `ktx create` would read as sRGB must not be converted to linear.
    pub assign_tf: Option<TransferFunction>,
    pub basis_lz: BasisLzOptions,
    pub uastc_ldr: UastcLdrOptions,
    pub astc: AstcOptions,
    pub supercompression: Supercompression,
}

/// One Official preset leaf for the frontend menu.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EncodePresetInfo {
    pub id: String,
    /// Breadcrumb segments, e.g. `["Color", "UASTC RGBA"]`.
    pub path: Vec<String>,
    /// Pros/cons tip for the preset menu item.
    pub help: String,
    pub owned: EncodePresetOwned,
}

#[derive(Clone)]
struct Recipe {
    id: &'static str,
    path: &'static [&'static str],
    help: &'static str,
    format: VkFormat,
    encode: EncodeCodec,
    normalize: bool,
    normal_mode: bool,
    uastc_rdo: bool,
    supercompression: Supercompression,
}

fn zstd_default() -> Supercompression {
    Supercompression::Zstd {
        level: ZSTD_DEFAULT_LEVEL,
    }
}

fn recipes() -> [Recipe; 12] {
    [
        // Color (sRGB)
        Recipe {
            id: "official.color.uastc-rgba",
            path: &["Color", "UASTC RGBA"],
            help: "Product default for sRGB color with alpha (albedo, decals, UI). Pros: high quality UASTC, keeps alpha, zstd shrinks the file. Cons: larger than BasisLZ.",
            format: VkFormat::R8G8B8A8Srgb,
            encode: EncodeCodec::Uastc,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: zstd_default(),
        },
        Recipe {
            id: "official.color.uastc-rgb",
            path: &["Color", "UASTC RGB"],
            help: "Opaque sRGB color (no alpha channel). Pros: same high quality as RGBA UASTC without unused alpha. Cons: cannot store transparency; larger than BasisLZ.",
            format: VkFormat::R8G8B8Srgb,
            encode: EncodeCodec::Uastc,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: zstd_default(),
        },
        Recipe {
            id: "official.color.basislz-rgba",
            path: &["Color", "BasisLZ RGBA"],
            help: "Smaller sRGB color with alpha when size matters more than peak quality. Pros: much smaller files than UASTC; alpha kept. Cons: more compression artifacts; BasisLZ cannot use zstd.",
            format: VkFormat::R8G8B8A8Srgb,
            encode: EncodeCodec::BasisLz,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: Supercompression::None,
        },
        Recipe {
            id: "official.color.basislz-rgb",
            path: &["Color", "BasisLZ RGB"],
            help: "Smaller opaque sRGB color. Pros: smallest typical color encode. Cons: no alpha; more artifacts than UASTC; BasisLZ cannot use zstd.",
            format: VkFormat::R8G8B8Srgb,
            encode: EncodeCodec::BasisLz,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: Supercompression::None,
        },
        // Data (linear UNORM)
        Recipe {
            id: "official.data.uastc-rgba",
            path: &["Data", "UASTC RGBA"],
            help: "Linear data with alpha (ORM packs, non-color channels). Pros: UNORM (not sRGB), high quality UASTC. Cons: wrong for display albedo (use Color); larger than BasisLZ.",
            format: VkFormat::R8G8B8A8Unorm,
            encode: EncodeCodec::Uastc,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: zstd_default(),
        },
        Recipe {
            id: "official.data.uastc-rgb",
            path: &["Data", "UASTC RGB"],
            help: "Linear opaque data (no alpha). Pros: UNORM UASTC quality without unused alpha. Cons: not for sRGB albedo; larger than BasisLZ.",
            format: VkFormat::R8G8B8Unorm,
            encode: EncodeCodec::Uastc,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: zstd_default(),
        },
        Recipe {
            id: "official.data.basislz-rgba",
            path: &["Data", "BasisLZ RGBA"],
            help: "Smaller linear data with alpha. Pros: much smaller than UASTC data; keeps alpha. Cons: more artifacts on packed channels; not for sRGB albedo; BasisLZ cannot use zstd.",
            format: VkFormat::R8G8B8A8Unorm,
            encode: EncodeCodec::BasisLz,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: Supercompression::None,
        },
        Recipe {
            id: "official.data.basislz-rgb",
            path: &["Data", "BasisLZ RGB"],
            help: "Smaller linear opaque data. Pros: compact UNORM RGB. Cons: no alpha; more artifacts than UASTC; not for sRGB albedo; BasisLZ cannot use zstd.",
            format: VkFormat::R8G8B8Unorm,
            encode: EncodeCodec::BasisLz,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: Supercompression::None,
        },
        // Normal (linear, normal-mode)
        Recipe {
            id: "official.normal.uastc",
            path: &["Normal", "UASTC"],
            help: "High-quality tangent-space normals. Pros: normalize + normal-mode, UASTC RDO tuned for normals, zstd shrinks the file. Cons: larger than BasisLZ normals; not for color/albedo.",
            format: VkFormat::R8G8B8A8Unorm,
            encode: EncodeCodec::Uastc,
            normalize: true,
            normal_mode: true,
            uastc_rdo: true,
            supercompression: zstd_default(),
        },
        Recipe {
            id: "official.normal.basis-lz",
            path: &["Normal", "BasisLZ"],
            help: "Smaller tangent-space normals. Pros: normalize + normal-mode with a much smaller file. Cons: more artifacts on subtle slopes than UASTC; BasisLZ cannot use zstd; not for color/albedo.",
            format: VkFormat::R8G8B8A8Unorm,
            encode: EncodeCodec::BasisLz,
            normalize: true,
            normal_mode: true,
            uastc_rdo: false,
            supercompression: Supercompression::None,
        },
        // Mask (R8 linear)
        Recipe {
            id: "official.mask.uastc-r8",
            path: &["Mask", "UASTC R8"],
            help: "Single-channel mask (AO, opacity mask, scalar map). Pros: R8 only, high quality UASTC. Cons: one channel only; larger than BasisLZ mask.",
            format: VkFormat::R8Unorm,
            encode: EncodeCodec::Uastc,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: zstd_default(),
        },
        Recipe {
            id: "official.mask.basislz-r8",
            path: &["Mask", "BasisLZ R8"],
            help: "Smaller single-channel mask. Pros: typically the smallest mask encode. Cons: more banding/artifacts possible; one channel only; BasisLZ cannot use zstd.",
            format: VkFormat::R8Unorm,
            encode: EncodeCodec::BasisLz,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: Supercompression::None,
        },
    ]
}

fn owned_from_recipe(recipe: &Recipe) -> EncodePresetOwned {
    let mut uastc_ldr = UastcLdrOptions::default();
    if recipe.uastc_rdo {
        uastc_ldr.uastc_rdo = true;
        uastc_ldr.uastc_rdo_l = Some(NORMAL_UASTC_RDO_L);
    }
    EncodePresetOwned {
        format: recipe.format,
        encode: Some(recipe.encode),
        normalize: recipe.normalize,
        normal_mode: recipe.normal_mode,
        assign_tf: (!recipe.format.is_srgb()).then_some(TransferFunction::Linear),
        basis_lz: BasisLzOptions::default(),
        uastc_ldr,
        astc: AstcOptions::default(),
        supercompression: recipe.supercompression.clone(),
    }
}

/// Full Official catalog for the preset menu.
pub fn list_presets() -> Vec<EncodePresetInfo> {
    recipes()
        .iter()
        .map(|recipe| {
            let owned = owned_from_recipe(recipe);
            EncodePresetInfo {
                id: recipe.id.to_owned(),
                path: recipe.path.iter().map(|s| (*s).to_owned()).collect(),
                help: recipe.help.to_owned(),
                owned,
            }
        })
        .collect()
}

fn find_recipe(id: &str) -> Option<Recipe> {
    recipes().into_iter().find(|r| r.id == id)
}

/// Merge a preset onto `current`, keeping container / filter / encoder.
pub fn apply_preset(current: &CreateOptions, id: &str) -> Option<CreateOptions> {
    let recipe = find_recipe(id)?;
    let owned = owned_from_recipe(&recipe);
    let mut next = current.clone();
    next.format = owned.format;
    next.encode = owned.encode;
    next.color.normalize = owned.normalize;
    next.color.normal_mode = owned.normal_mode;
    // sRGB presets let the format carry the transfer; linear ones assign it
    // so the input bytes pass through unconverted. Clear any conversion.
    next.color.assign_tf = owned.assign_tf;
    next.color.convert_tf = None;
    next.basis_lz = owned.basis_lz;
    next.uastc_ldr = owned.uastc_ldr;
    next.astc = owned.astc;
    next.supercompression = owned.supercompression;
    Some(next)
}

fn owned_equal(options: &CreateOptions, owned: &EncodePresetOwned) -> bool {
    options.format == owned.format
        && options.encode == owned.encode
        && options.color.normalize == owned.normalize
        && options.color.normal_mode == owned.normal_mode
        && options.color.assign_tf == owned.assign_tf
        && options.color.convert_tf.is_none()
        && options.supercompression == owned.supercompression
        && options.basis_lz == owned.basis_lz
        && options.uastc_ldr == owned.uastc_ldr
        && options.astc == owned.astc
}

/// Whether `options` still match the named Official preset’s owned fields.
pub fn matches_preset(options: &CreateOptions, id: &str) -> bool {
    let Some(recipe) = find_recipe(id) else {
        return false;
    };
    owned_equal(options, &owned_from_recipe(&recipe))
}

/// Product `CreateOptions::default` payload (Color UASTC RGBA + zstd 18).
pub fn product_default_options() -> CreateOptions {
    let baseline = CreateOptions {
        format: VkFormat::R8G8B8A8Srgb,
        encode: None,
        container: Default::default(),
        filter: Default::default(),
        color: Default::default(),
        basis_lz: Default::default(),
        uastc_ldr: Default::default(),
        astc: Default::default(),
        encoder: Default::default(),
        supercompression: Supercompression::None,
    };
    apply_preset(&baseline, DEFAULT_ENCODE_PRESET_ID).expect("default encode preset must exist")
}
