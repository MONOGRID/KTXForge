import { expect, test } from "@playwright/test";
import type { EncodePresetOwned } from "../../src/bindings/EncodePresetOwned";
import {
  DEFAULT_ENCODE_PRESET_ID,
  presetsByCategory,
  presetBreadcrumb,
} from "../../src/encoder/presets";
import type { EncodePresetInfo } from "../../src/bindings/EncodePresetInfo";

function colorOwned(): EncodePresetOwned {
  return {
    format: "R8G8B8A8_SRGB",
    encode: "uastc",
    normalize: false,
    normalMode: false,
    basisLz: {
      qlevel: 128,
      clevel: 1,
      maxEndpoints: null,
      maxSelectors: null,
      endpointRdoThreshold: 1.25,
      selectorRdoThreshold: 1.25,
      noEndpointRdo: false,
      noSelectorRdo: false,
    },
    uastcLdr: {
      uastcQuality: 1,
      uastcRdo: false,
      uastcRdoL: 1,
      uastcRdoD: 4096,
      uastcRdoB: 10,
      uastcRdoS: 18,
      uastcRdoF: false,
      uastcRdoM: false,
    },
    astc: { astcQuality: "medium", astcPerceptual: false },
    assignTf: null,
    supercompression: { kind: "zstd", level: 18 },
  };
}

test.describe("encode presets helpers", () => {
  test("keeps the stable default id", () => {
    expect(DEFAULT_ENCODE_PRESET_ID).toBe("official.color.uastc-rgba");
  });

  test("groups by category and builds breadcrumbs", () => {
    const presets = [
      {
        id: "official.color.uastc-rgba",
        path: ["Color", "UASTC RGBA"],
        help: "color tip",
        owned: colorOwned(),
      },
      {
        id: "official.data.uastc-rgba",
        path: ["Data", "UASTC RGBA"],
        help: "data tip",
        owned: colorOwned(),
      },
    ] satisfies EncodePresetInfo[];
    expect(presetBreadcrumb(presets[0])).toBe("Color > UASTC RGBA");
    expect(presetsByCategory(presets).map((g) => g.category)).toEqual([
      "Color",
      "Data",
    ]);
  });
});
