//! Input list role: how many images map to which KTX dimension.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::commands::{SourceInfo, SourceKind};
use crate::ktx::options::{CreateOptions, TexcoordOrigin};
use crate::ktx::rules;

/// What the per-output input list means for `ktx create` arity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
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

impl InputRole {
    pub fn cli_label(self) -> &'static str {
        match self {
            Self::Base2D => "base2d",
            Self::ArrayLayers => "arrayLayers",
            Self::CubemapFaces => "cubemapFaces",
            Self::MipChain => "mipChain",
            Self::DepthSlices => "depthSlices",
        }
    }
}

/// Apply role-driven container fields from `input_count`.
///
/// Clears arity fields the role does not own. Does not touch width/height/scale/raw/1d.
pub fn apply_role(opts: &mut CreateOptions, role: InputRole, input_count: usize) {
    let n = input_count as u32;
    match role {
        InputRole::Base2D => {
            opts.container.layers = None;
            opts.container.cubemap = false;
            opts.container.depth = None;
        }
        InputRole::ArrayLayers => {
            opts.container.layers = if n > 0 { Some(n) } else { None };
            opts.container.cubemap = false;
            opts.container.depth = None;
        }
        InputRole::CubemapFaces => {
            opts.container.cubemap = true;
            opts.container.layers = None;
            opts.container.depth = None;
            opts.container.one_d = false;
            ensure_top_left_origin(opts);
        }
        InputRole::MipChain => {
            opts.container.levels = if n > 0 { Some(n) } else { None };
            opts.container.generate_mipmap = false;
            opts.container.runtime_mipmap = false;
            opts.container.layers = None;
            opts.container.cubemap = false;
            opts.container.depth = None;
        }
        InputRole::DepthSlices => {
            opts.container.depth = if n > 0 { Some(n) } else { None };
            opts.container.layers = None;
            opts.container.cubemap = false;
            opts.container.one_d = false;
            ensure_front_back_origin(opts);
        }
    }
}

fn ensure_top_left_origin(opts: &mut CreateOptions) {
    match opts.color.assign_texcoord_origin {
        Some(TexcoordOrigin::TopLeft)
        | Some(TexcoordOrigin::TopLeftFront)
        | Some(TexcoordOrigin::TopLeftBack) => {}
        _ => {
            opts.color.assign_texcoord_origin = Some(TexcoordOrigin::TopLeft);
        }
    }
}

fn ensure_front_back_origin(opts: &mut CreateOptions) {
    match opts.color.assign_texcoord_origin {
        Some(TexcoordOrigin::TopLeftFront)
        | Some(TexcoordOrigin::TopLeftBack)
        | Some(TexcoordOrigin::BottomLeftFront)
        | Some(TexcoordOrigin::BottomLeftBack) => {}
        _ => {
            opts.color.assign_texcoord_origin = Some(TexcoordOrigin::TopLeftFront);
        }
    }
}

/// Validate that `inputs` satisfy the role (count only).
pub fn validate_arity(role: InputRole, input_count: usize) -> Result<(), String> {
    if input_count == 0 {
        return Err("each KTX output needs at least one input image".to_owned());
    }
    match role {
        InputRole::Base2D if input_count != 1 => Err(format!(
            "Base 2D expects exactly 1 input image, got {input_count}"
        )),
        InputRole::CubemapFaces if input_count != 6 => Err(format!(
            "Cubemap expects exactly 6 face images (+X −X +Y −Y +Z −Z), got {input_count}"
        )),
        InputRole::ArrayLayers | InputRole::MipChain | InputRole::DepthSlices => Ok(()),
        _ => Ok(()),
    }
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

/// Keys always owned by the input-role panel (never shown as free Container fields).
pub fn always_role_driven(key: crate::ktx::options::OptionKey) -> bool {
    use crate::ktx::options::OptionKey;
    matches!(
        key,
        OptionKey::Layers | OptionKey::Cubemap | OptionKey::Depth
    )
}

/// Extra keys hidden for the active role (beyond [`always_role_driven`]).
pub fn hidden_for_role(role: InputRole, key: crate::ktx::options::OptionKey) -> bool {
    use crate::ktx::options::OptionKey;
    if always_role_driven(key) {
        return true;
    }
    match role {
        InputRole::MipChain => matches!(
            key,
            OptionKey::Levels | OptionKey::GenerateMipmap | OptionKey::RuntimeMipmap
        ),
        // generate-mipmap is invalid for 3D; depth count comes from the list.
        InputRole::DepthSlices => matches!(
            key,
            OptionKey::GenerateMipmap | OptionKey::RuntimeMipmap | OptionKey::Levels
        ),
        InputRole::Base2D | InputRole::ArrayLayers | InputRole::CubemapFaces => false,
    }
}

/// Full validate for one output slot before encode.
pub fn validate_output(
    opts: &CreateOptions,
    role: InputRole,
    sources: &[SourceInfo],
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if let Err(e) = validate_arity(role, sources.len()) {
        errors.push(e);
    }
    if let Err(opt_errors) = rules::validate_for_sources(opts, sources) {
        errors.extend(opt_errors);
    }
    // Animation requires array layers ≥ 2 — checked by caller when animation is Some.
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
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

    #[test]
    fn apply_array_layers_sets_layers() {
        let mut o = CreateOptions::default();
        apply_role(&mut o, InputRole::ArrayLayers, 4);
        assert_eq!(o.container.layers, Some(4));
        assert!(!o.container.cubemap);
    }

    #[test]
    fn apply_cubemap_sets_flag_and_origin() {
        let mut o = CreateOptions::default();
        apply_role(&mut o, InputRole::CubemapFaces, 6);
        assert!(o.container.cubemap);
        assert_eq!(
            o.color.assign_texcoord_origin,
            Some(TexcoordOrigin::TopLeft)
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
    fn compatible_same_kind() {
        assert!(compatible_with_anchor(&src(SourceKind::Ldr), &src(SourceKind::Ldr)).is_ok());
        assert!(compatible_with_anchor(&src(SourceKind::Ldr), &src(SourceKind::Hdr)).is_err());
    }
}
