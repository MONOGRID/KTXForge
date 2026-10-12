//! Input list role: how many images map to which KTX dimension.

use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use ts_rs::TS;

use crate::commands::{SourceInfo, SourceKind};
use crate::ktx::options::{CreateOptions, OptionKey, TexcoordOrigin};

/// What the per-output input list means for `ktx create` arity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS, EnumIter)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum InputRole {
    /// Single 2D base image (default batch item).
    #[default]
    #[serde(rename = "base2D")]
    #[ts(rename = "base2D")]
    Base2D,
    /// Array layers / animation frames → `--layers = count`.
    ArrayLayers,
    /// Cubemap faces in +X −X +Y −Y +Z −Z order → `--cubemap`, count must be 6.
    CubemapFaces,
    /// Explicit mip chain → `--levels = count`, no generate-mipmap.
    MipChain,
    /// 3D depth slices → `--depth = count`.
    DepthSlices,
}

/// How many inputs a role accepts. `max: None` means unbounded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RoleArity {
    pub role: InputRole,
    pub min: u32,
    pub max: Option<u32>,
}

impl InputRole {
    fn label(self) -> &'static str {
        match self {
            Self::Base2D => "Base 2D",
            Self::ArrayLayers => "Array layers",
            Self::CubemapFaces => "Cubemap (+X −X +Y −Y +Z −Z)",
            Self::MipChain => "Mip chain",
            Self::DepthSlices => "Depth slices",
        }
    }

    pub fn arity(self) -> RoleArity {
        let (min, max) = match self {
            Self::Base2D => (1, Some(1)),
            Self::CubemapFaces => (6, Some(6)),
            Self::ArrayLayers | Self::MipChain | Self::DepthSlices => (1, None),
        };
        RoleArity {
            role: self,
            min,
            max,
        }
    }
}

/// Arity for every role, so the frontend can size the Inputs list without its own copy.
pub fn all_arities() -> Vec<RoleArity> {
    InputRole::iter().map(InputRole::arity).collect()
}

/// Write the container fields the role owns, derived from `input_count`.
///
/// Every key [`hidden_for_role`] reports for a role is fully determined here, so a value the
/// user set before switching roles never leaks into the encode. Does not touch
/// width / height / scale / raw.
pub fn apply_role(opts: &mut CreateOptions, role: InputRole, input_count: usize) {
    let n = u32::try_from(input_count).ok().filter(|n| *n > 0);
    let c = &mut opts.container;
    c.layers = None;
    c.cubemap = false;
    c.depth = None;
    match role {
        InputRole::Base2D => {}
        InputRole::ArrayLayers => c.layers = n,
        InputRole::CubemapFaces => {
            c.cubemap = true;
            c.one_d = false;
        }
        InputRole::MipChain => {
            c.levels = n;
            c.generate_mipmap = false;
            c.runtime_mipmap = false;
        }
        InputRole::DepthSlices => {
            c.depth = n;
            c.one_d = false;
            c.levels = None;
            c.generate_mipmap = false;
            c.runtime_mipmap = false;
        }
    }
    constrain_origins(opts, role);
}

/// Texture-coordinate origins `ktx create` accepts for the role's texture shape.
pub fn origin_allowed(role: InputRole, origin: TexcoordOrigin) -> bool {
    match role {
        // "Cubemaps must stay top-left"
        InputRole::CubemapFaces => matches!(
            origin,
            TexcoordOrigin::TopLeft | TexcoordOrigin::TopLeftFront | TexcoordOrigin::TopLeftBack
        ),
        // "--depth … Requires a front/back texture-coordinate origin"
        InputRole::DepthSlices => matches!(
            origin,
            TexcoordOrigin::TopLeftFront
                | TexcoordOrigin::TopLeftBack
                | TexcoordOrigin::BottomLeftFront
                | TexcoordOrigin::BottomLeftBack
        ),
        InputRole::Base2D | InputRole::ArrayLayers | InputRole::MipChain => true,
    }
}

fn constrain_origins(opts: &mut CreateOptions, role: InputRole) {
    let color = &mut opts.color;
    if !color
        .assign_texcoord_origin
        .is_some_and(|o| origin_allowed(role, o))
    {
        color.assign_texcoord_origin = match role {
            InputRole::CubemapFaces => Some(TexcoordOrigin::TopLeft),
            InputRole::DepthSlices => Some(TexcoordOrigin::TopLeftFront),
            InputRole::Base2D | InputRole::ArrayLayers | InputRole::MipChain => {
                color.assign_texcoord_origin
            }
        };
    }
    color.convert_texcoord_origin = color
        .convert_texcoord_origin
        .filter(|o| origin_allowed(role, *o));
}

/// Validate that `inputs` satisfy the role (count only).
pub fn validate_arity(role: InputRole, input_count: usize) -> Result<(), String> {
    if input_count == 0 {
        return Err("each KTX output needs at least one input image".to_owned());
    }
    let arity = role.arity();
    let count = u32::try_from(input_count).unwrap_or(u32::MAX);
    if count >= arity.min && arity.max.is_none_or(|max| count <= max) {
        return Ok(());
    }
    let expected = match arity.max {
        Some(max) if max == arity.min => format!("exactly {max}"),
        Some(max) => format!("{}–{max}", arity.min),
        None => format!("at least {}", arity.min),
    };
    Err(format!(
        "{} expects {expected} input image(s), got {input_count}",
        role.label()
    ))
}

/// Whether a new primary source may join a queue pinned by `anchor` (first output).
pub fn compatible_with_anchor(anchor: &SourceInfo, candidate: &SourceInfo) -> Result<(), String> {
    if anchor.kind != candidate.kind {
        let a = kind_label(anchor.kind);
        let b = kind_label(candidate.kind);
        return Err(format!(
            "incompatible with the first KTX output ({a}); cannot add {b} source"
        ));
    }
    Ok(())
}

fn kind_label(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::Ldr => "LDR",
        SourceKind::Hdr => "HDR",
    }
}

/// Keys owned by the input role: never shown as free Container fields, always written by
/// [`apply_role`].
pub fn hidden_for_role(role: InputRole, key: OptionKey) -> bool {
    if matches!(
        key,
        OptionKey::Layers | OptionKey::Cubemap | OptionKey::Depth
    ) {
        return true;
    }
    match role {
        InputRole::MipChain | InputRole::DepthSlices => matches!(
            key,
            OptionKey::Levels | OptionKey::GenerateMipmap | OptionKey::RuntimeMipmap
        ),
        InputRole::Base2D | InputRole::ArrayLayers | InputRole::CubemapFaces => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::SourceKind;

    fn src(kind: SourceKind) -> SourceInfo {
        SourceInfo {
            path: "a.png".into(),
            name: "a.png".into(),
            kind,
            width: 4,
            height: 4,
            needs_conversion: false,
        }
    }

    /// Options with every role-owned field set to something no role would pick.
    fn cluttered() -> CreateOptions {
        let mut o = CreateOptions::default();
        o.container.layers = Some(7);
        o.container.cubemap = true;
        o.container.depth = Some(5);
        o.container.levels = Some(9);
        o.container.generate_mipmap = true;
        o.container.runtime_mipmap = true;
        o.container.one_d = true;
        o.color.assign_texcoord_origin = Some(TexcoordOrigin::BottomLeft);
        o.color.convert_texcoord_origin = Some(TexcoordOrigin::BottomLeft);
        o
    }

    #[test]
    fn hidden_keys_do_not_depend_on_prior_values() {
        for role in InputRole::iter() {
            let mut a = CreateOptions::default();
            let mut b = cluttered();
            apply_role(&mut a, role, 3);
            apply_role(&mut b, role, 3);
            let (a, b) = (&a.container, &b.container);
            assert_eq!(a.layers, b.layers, "{role:?} layers");
            assert_eq!(a.cubemap, b.cubemap, "{role:?} cubemap");
            assert_eq!(a.depth, b.depth, "{role:?} depth");
            if hidden_for_role(role, OptionKey::Levels) {
                assert_eq!(a.levels, b.levels, "{role:?} levels");
                assert_eq!(a.generate_mipmap, b.generate_mipmap, "{role:?} generate");
                assert_eq!(a.runtime_mipmap, b.runtime_mipmap, "{role:?} runtime");
            }
        }
    }

    #[test]
    fn base2d_keeps_user_levels_and_mipmaps() {
        let mut o = CreateOptions::default();
        o.container.levels = Some(4);
        o.container.generate_mipmap = true;
        apply_role(&mut o, InputRole::Base2D, 1);
        assert_eq!(o.container.levels, Some(4));
        assert!(o.container.generate_mipmap);
    }

    #[test]
    fn apply_array_layers_sets_layers() {
        let mut o = CreateOptions::default();
        apply_role(&mut o, InputRole::ArrayLayers, 4);
        assert_eq!(o.container.layers, Some(4));
        assert!(!o.container.cubemap);
    }

    #[test]
    fn apply_cubemap_sets_flag_and_origin() {
        let mut o = cluttered();
        apply_role(&mut o, InputRole::CubemapFaces, 6);
        assert!(o.container.cubemap);
        assert!(!o.container.one_d);
        assert_eq!(
            o.color.assign_texcoord_origin,
            Some(TexcoordOrigin::TopLeft)
        );
        assert_eq!(o.color.convert_texcoord_origin, None);
    }

    #[test]
    fn apply_depth_sets_front_back_origin() {
        let mut o = cluttered();
        apply_role(&mut o, InputRole::DepthSlices, 4);
        assert_eq!(o.container.depth, Some(4));
        assert_eq!(
            o.color.assign_texcoord_origin,
            Some(TexcoordOrigin::TopLeftFront)
        );
    }

    #[test]
    fn apply_mip_chain_clears_generate() {
        let mut o = CreateOptions::default();
        o.container.generate_mipmap = true;
        apply_role(&mut o, InputRole::MipChain, 3);
        assert_eq!(o.container.levels, Some(3));
        assert!(!o.container.generate_mipmap);
    }

    #[test]
    fn arity_cubemap() {
        assert!(validate_arity(InputRole::CubemapFaces, 6).is_ok());
        assert!(validate_arity(InputRole::CubemapFaces, 5).is_err());
    }

    #[test]
    fn arity_base2d_and_packs() {
        assert!(validate_arity(InputRole::Base2D, 1).is_ok());
        assert!(validate_arity(InputRole::Base2D, 2).is_err());
        assert!(validate_arity(InputRole::ArrayLayers, 0).is_err());
        assert!(validate_arity(InputRole::ArrayLayers, 40).is_ok());
    }

    #[test]
    fn compatible_same_kind() {
        assert!(compatible_with_anchor(&src(SourceKind::Ldr), &src(SourceKind::Ldr)).is_ok());
        assert!(compatible_with_anchor(&src(SourceKind::Ldr), &src(SourceKind::Hdr)).is_err());
    }
}
