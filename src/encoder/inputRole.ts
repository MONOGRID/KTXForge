import type { InputRole } from "../bindings/InputRole";

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

/** Narrow a select value to a pack role without a cast. */
export function parsePackRole(value: string): PackInputRole | null {
  return INPUT_ROLE_OPTIONS.find((opt) => opt.value === value)?.value ?? null;
}

export function roleLabel(role: InputRole): string {
  return (
    INPUT_ROLE_OPTIONS.find((opt) => opt.value === role)?.label ?? "Base 2D"
  );
}
