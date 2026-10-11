import type { CreateOptions } from "../bindings/CreateOptions";
import type { InputRole } from "../bindings/InputRole";
import type { TexcoordOrigin } from "../bindings/TexcoordOrigin";

/** Multi-input roles shown in the Inputs panel (Base 2D is implicit for single-input outputs). */
export type PackInputRole = Exclude<InputRole, "base2D">;

/** Labels for the Inputs panel role selector. */
export const INPUT_ROLE_OPTIONS = [
  {
    value: "arrayLayers",
    label: "Array layers",
    help: "List length becomes --layers. Use for arrays and animation frames.",
  },
  {
    value: "cubemapFaces",
    label: "Cubemap faces",
    help: "Exactly six images: +X −X +Y −Y +Z −Z.",
  },
  {
    value: "mipChain",
    label: "Mip chain",
    help: "List length becomes --levels. Generate-mipmap is off.",
  },
  {
    value: "depthSlices",
    label: "Depth slices",
    help: "List length becomes --depth for a 3D texture.",
  },
] as const satisfies ReadonlyArray<{
  value: PackInputRole;
  label: string;
  help: string;
}>;

export function isPackRole(role: InputRole): role is PackInputRole {
  return role !== "base2D";
}

/** Mirror of Rust `input_role::apply_role` — keeps Container options honest in the UI. */
export function applyRoleToOptions(
  options: CreateOptions,
  role: InputRole,
  inputCount: number,
): void {
  const n = inputCount;
  switch (role) {
    case "base2D":
      options.container.layers = null;
      options.container.cubemap = false;
      options.container.depth = null;
      break;
    case "arrayLayers":
      options.container.layers = n > 0 ? n : null;
      options.container.cubemap = false;
      options.container.depth = null;
      break;
    case "cubemapFaces":
      options.container.cubemap = true;
      options.container.layers = null;
      options.container.depth = null;
      options.container.oneD = false;
      ensureTopLeftOrigin(options);
      break;
    case "mipChain":
      options.container.levels = n > 0 ? n : null;
      options.container.generateMipmap = false;
      options.container.runtimeMipmap = false;
      options.container.layers = null;
      options.container.cubemap = false;
      options.container.depth = null;
      break;
    case "depthSlices":
      options.container.depth = n > 0 ? n : null;
      options.container.layers = null;
      options.container.cubemap = false;
      options.container.oneD = false;
      ensureFrontBackOrigin(options);
      break;
  }
}

function ensureTopLeftOrigin(options: CreateOptions): void {
  const current = options.color.assignTexcoordOrigin;
  if (
    current === "top-left" ||
    current === "top-left-front" ||
    current === "top-left-back"
  ) {
    return;
  }
  options.color.assignTexcoordOrigin = "top-left" satisfies TexcoordOrigin;
}

function ensureFrontBackOrigin(options: CreateOptions): void {
  const current = options.color.assignTexcoordOrigin;
  if (
    current === "top-left-front" ||
    current === "top-left-back" ||
    current === "bottom-left-front" ||
    current === "bottom-left-back"
  ) {
    return;
  }
  options.color.assignTexcoordOrigin = "top-left-front" satisfies TexcoordOrigin;
}

export function validateArity(
  role: InputRole,
  inputCount: number,
): string | null {
  if (inputCount === 0) {
    return "each KTX output needs at least one input image";
  }
  if (role === "base2D" && inputCount !== 1) {
    return `Base 2D expects exactly 1 input image, got ${String(inputCount)}`;
  }
  if (role === "cubemapFaces" && inputCount !== 6) {
    return `Cubemap expects exactly 6 face images, got ${String(inputCount)}`;
  }
  return null;
}

/** Max inputs the bottom panel should accept for this role (`null` = unlimited). */
export function maxInputsForRole(role: InputRole): number | null {
  if (role === "base2D") {
    return 1;
  }
  if (role === "cubemapFaces") {
    return 6;
  }
  return null;
}
