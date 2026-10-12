//! Official encode presets (Handbrake-style recipes).
//!
//! Presets own format / encode / color TF+normal flags / codec blocks /
//! supercompression. They never touch `container`, `filter`, or `encoder`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::options::{
    AstcOptions, BasisLzOptions, CreateOptions, EncodeCodec, Supercompression, UastcLdrOptions,
    VkFormat,
};

/// Product default: Official > Color > UASTC RGBA.
pub const DEFAULT_ENCODE_PRESET_ID: &str = "official.color.uastc-rgba";

const ZSTD_DEFAULT_LEVEL: u8 = 22;
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

fn zstd22() -> Supercompression {
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
            help: "Product default for sRGB color with alpha (albedo, decals, UI). Pros: high quality UASTC, keeps alpha, zstd shrinks the file. Cons: larger than BasisLZ; zstd 22 uses much more encode memory.",
            format: VkFormat::R8G8B8A8Srgb,
            encode: EncodeCodec::Uastc,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: zstd22(),
        },
        Recipe {
            id: "official.color.uastc-rgb",
            path: &["Color", "UASTC RGB"],
            help: "Opaque sRGB color (no alpha channel). Pros: same high quality as RGBA UASTC without unused alpha. Cons: cannot store transparency; larger than BasisLZ; zstd 22 uses much more encode memory.",
            format: VkFormat::R8G8B8Srgb,
            encode: EncodeCodec::Uastc,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: zstd22(),
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
            help: "Linear data with alpha (ORM packs, non-color channels). Pros: UNORM (not sRGB), high quality UASTC. Cons: wrong for display albedo (use Color); larger than BasisLZ; zstd 22 uses much more encode memory.",
            format: VkFormat::R8G8B8A8Unorm,
            encode: EncodeCodec::Uastc,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: zstd22(),
        },
        Recipe {
            id: "official.data.uastc-rgb",
            path: &["Data", "UASTC RGB"],
            help: "Linear opaque data (no alpha). Pros: UNORM UASTC quality without unused alpha. Cons: not for sRGB albedo; larger than BasisLZ; zstd 22 uses much more encode memory.",
            format: VkFormat::R8G8B8Unorm,
            encode: EncodeCodec::Uastc,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: zstd22(),
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
            help: "High-quality tangent-space normals. Pros: normalize + normal-mode, UASTC RDO tuned for normals, zstd shrinks the file. Cons: larger than BasisLZ normals; zstd 22 uses much more encode memory; not for color/albedo.",
            format: VkFormat::R8G8B8A8Unorm,
            encode: EncodeCodec::Uastc,
            normalize: true,
            normal_mode: true,
            uastc_rdo: true,
            supercompression: zstd22(),
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
            help: "Single-channel mask (AO, opacity mask, scalar map). Pros: R8 only, high quality UASTC. Cons: one channel only; larger than BasisLZ mask; zstd 22 uses much more encode memory.",
            format: VkFormat::R8Unorm,
            encode: EncodeCodec::Uastc,
            normalize: false,
            normal_mode: false,
            uastc_rdo: false,
            supercompression: zstd22(),
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
    // Linear vs sRGB is carried by the format; clear leftover TF flags.
    next.color.assign_tf = None;
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
        && options.color.assign_tf.is_none()
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

/// Product `CreateOptions::default` payload (Color UASTC RGBA + zstd 22).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ktx::options::FilterOptions;

    #[test]
    fn catalog_has_twelve_official_leaves() {
        let list = list_presets();
        assert_eq!(list.len(), 12);
        assert_eq!(list[0].id, DEFAULT_ENCODE_PRESET_ID);
        assert_eq!(list[0].path, vec!["Color", "UASTC RGBA"]);
        assert!(list.iter().all(|p| !p.help.is_empty()));
        let categories: Vec<&str> = list
            .iter()
            .map(|p| p.path[0].as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        assert_eq!(categories, vec!["Color", "Data", "Mask", "Normal"]);
    }

    #[test]
    fn every_leaf_payload_matches_recipe_intent() {
        for info in list_presets() {
            let applied = apply_preset(&product_default_options(), &info.id).unwrap();
            assert!(
                matches_preset(&applied, &info.id),
                "{} should match after apply",
                info.id
            );
            match info.id.as_str() {
                id if id.contains("basislz") || id.contains("basis-lz") => {
                    assert_eq!(applied.encode, Some(EncodeCodec::BasisLz));
                    assert!(matches!(applied.supercompression, Supercompression::None));
                }
                id if id.contains("uastc") => {
                    assert_eq!(applied.encode, Some(EncodeCodec::Uastc));
                    assert!(matches!(
                        applied.supercompression,
                        Supercompression::Zstd { level: 22 }
                    ));
                }
                other => panic!("unexpected preset id {other}"),
            }
            if info.path[0] == "Normal" {
                assert!(applied.color.normal_mode);
                assert!(applied.color.normalize);
            } else {
                assert!(!applied.color.normal_mode);
                assert!(!applied.color.normalize);
            }
            if info.path[0] == "Mask" {
                assert_eq!(applied.format, VkFormat::R8Unorm);
            }
            if info.path[0] == "Color" {
                assert!(applied.format.is_srgb());
            }
            if info.path[0] == "Data" {
                assert!(!applied.format.is_srgb());
            }
        }
    }

    #[test]
    fn product_default_is_color_uastc_rgba() {
        let opts = product_default_options();
        assert_eq!(opts.format, VkFormat::R8G8B8A8Srgb);
        assert_eq!(opts.encode, Some(EncodeCodec::Uastc));
        assert!(matches!(
            opts.supercompression,
            Supercompression::Zstd { level: 22 }
        ));
        assert!(!opts.color.normal_mode);
        assert!(!opts.color.normalize);
        assert!(matches_preset(&opts, DEFAULT_ENCODE_PRESET_ID));
    }

    #[test]
    fn apply_preserves_container_filter_encoder() {
        let mut current = product_default_options();
        current.container.generate_mipmap = true;
        current.container.cubemap = true;
        current.filter.mipmap_filter_scale = Some(2.0);
        current.encoder.compare_ssim = true;
        current.encoder.threads = Some(4);

        let next = apply_preset(&current, "official.normal.basis-lz").expect("preset");
        assert!(next.container.generate_mipmap);
        assert!(next.container.cubemap);
        assert_eq!(next.filter.mipmap_filter_scale, Some(2.0));
        assert!(next.encoder.compare_ssim);
        assert_eq!(next.encoder.threads, Some(4));
        assert_eq!(next.encode, Some(EncodeCodec::BasisLz));
        assert!(next.color.normal_mode);
        assert!(next.color.normalize);
        assert!(matches!(next.supercompression, Supercompression::None));
    }

    #[test]
    fn apply_normal_uastc_sets_rdo_lambda() {
        let next = apply_preset(&product_default_options(), "official.normal.uastc").unwrap();
        assert!(next.uastc_ldr.uastc_rdo);
        assert_eq!(next.uastc_ldr.uastc_rdo_l, Some(0.5));
        assert!(matches_preset(&next, "official.normal.uastc"));
    }

    #[test]
    fn mutate_owned_field_breaks_match() {
        let mut opts = product_default_options();
        assert!(matches_preset(&opts, DEFAULT_ENCODE_PRESET_ID));
        opts.supercompression = Supercompression::Zstd { level: 3 };
        assert!(!matches_preset(&opts, DEFAULT_ENCODE_PRESET_ID));
    }

    #[test]
    fn mutate_filter_keeps_match() {
        let mut opts = product_default_options();
        opts.filter = FilterOptions {
            mipmap_filter_scale: Some(3.0),
            ..FilterOptions::default()
        };
        assert!(matches_preset(&opts, DEFAULT_ENCODE_PRESET_ID));
    }

    #[test]
    fn switch_from_normal_clears_rdo_and_normal_mode() {
        let normal = apply_preset(&product_default_options(), "official.normal.uastc").unwrap();
        let color = apply_preset(&normal, DEFAULT_ENCODE_PRESET_ID).unwrap();
        assert!(!color.color.normal_mode);
        assert!(!color.uastc_ldr.uastc_rdo);
        assert!(matches_preset(&color, DEFAULT_ENCODE_PRESET_ID));
    }
}
