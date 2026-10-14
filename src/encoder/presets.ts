import type { EncodePresetInfo } from "../bindings/EncodePresetInfo";

/** Product default: Official > Color > UASTC RGBA. */
export const DEFAULT_ENCODE_PRESET_ID = "official.color.uastc-rgba";

export function presetBreadcrumb(preset: EncodePresetInfo): string {
  return preset.path.join(" > ");
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
