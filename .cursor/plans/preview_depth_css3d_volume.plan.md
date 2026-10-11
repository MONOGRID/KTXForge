---
name: Preview depth CSS3D volume
overview: Follow-up to role-aware Preview — visualize DepthSlices as an oblique CSS 3D stack of slice planes (preserve-3d), still driven by Rust PNG extracts / source_preview.
todos:
  - id: depth-volume-view
    content: CSS3D oblique stack component for DepthSlices (source + optional encoded wipe of whole stack or active slice)
    status: completed
  - id: depth-wire-preview
    content: Wire PreviewPane DepthSlices mode to volume view; keep 2D slice slider as fallback/control
    status: completed
  - id: depth-docs
    content: requirements.md + decision-context.md for depth volume preview
    status: completed
isProject: false
---

# Preview: CSS3D depth volume (follow-up)

Deferred from [Preview role shapes](preview_role_shapes_1b0ea1db.plan.md).

## Goal

For `InputRole::DepthSlices`, show the depth texture as an **oblique CSS 3D stack**: each slice is a textured plane in a `perspective` / `transform-style: preserve-3d` scene, rotated so the volume is readable at a glance.

## Constraints

- Vue/HTML/CSS visualization only — slices still come from `source_preview` (before encode) and `encoded_preview` + `ImageSelection.depthSlice` (after encode).
- No WebGL / custom shaders in this pass.
- Keep the existing **Depth** slider to choose a highlighted slice (and to drive wipe compare); the volume is the spatial overview.
- Wipe strategy (pick one when implementing): either wipe the **highlighted** slice in a side panel, or duplicate the stack (source vs encoded) with one clip — prefer the simpler highlighted-slice wipe first.

## Out of scope

- Cube arrays / compound shapes.
- Raymarched volume rendering.
