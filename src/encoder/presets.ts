import type { CreateOptions } from "../bindings/CreateOptions";
import type { EncodePresetInfo } from "../bindings/EncodePresetInfo";
import type { EncodePresetOwned } from "../bindings/EncodePresetOwned";

/** Product default: Official > Color > UASTC RGBA. */
export const DEFAULT_ENCODE_PRESET_ID = "official.color.uastc-rgba";

export function presetBreadcrumb(preset: EncodePresetInfo): string {
  return preset.path.join(" > ");
}

export function optionsMatchPreset(
  options: CreateOptions,
  owned: EncodePresetOwned,
): boolean {
  return (
    options.format === owned.format &&
    options.encode === owned.encode &&
    options.color.normalize === owned.normalize &&
    options.color.normalMode === owned.normalMode &&
    options.color.assignTf === null &&
    options.color.convertTf === null &&
    JSON.stringify(options.supercompression) ===
      JSON.stringify(owned.supercompression) &&
    JSON.stringify(options.basisLz) === JSON.stringify(owned.basisLz) &&
    JSON.stringify(options.uastcLdr) === JSON.stringify(owned.uastcLdr) &&
    JSON.stringify(options.astc) === JSON.stringify(owned.astc)
  );
}

/** Group preset leaves by category (`Color`, `Data`, `Normal`, `Mask`). */
export function presetsByCategory(
  presets: readonly EncodePresetInfo[],
): { category: string; presets: EncodePresetInfo[] }[] {
  const order: string[] = [];
  const map = new Map<string, EncodePresetInfo[]>();
  for (const preset of presets) {
    const category = preset.path[0] ?? "Other";
    const list = map.get(category);
    if (list === undefined) {
      order.push(category);
      map.set(category, [preset]);
    } else {
      list.push(preset);
    }
  }
  return order.map((category) => ({
    category,
    presets: map.get(category) ?? [],
  }));
}
