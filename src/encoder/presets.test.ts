import { describe, expect, it } from "vitest";
import type { CreateOptions } from "../bindings/CreateOptions";
import type { EncodePresetOwned } from "../bindings/EncodePresetOwned";
import {
  DEFAULT_ENCODE_PRESET_ID,
  optionsMatchPreset,
  presetsByCategory,
  presetBreadcrumb,
} from "./presets";
import type { EncodePresetInfo } from "../bindings/EncodePresetInfo";

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
    supercompression: { kind: "zstd", level: 22 },
  };
}

function optionsFromOwned(owned: EncodePresetOwned): CreateOptions {
  return {
    format: owned.format,
    encode: owned.encode,
    container: {
      oneD: false,
      cubemap: false,
      layers: null,
      depth: null,
      levels: null,
      generateMipmap: false,
      runtimeMipmap: false,
      width: null,
      height: null,
      scale: 1,
      raw: false,
    },
    filter: {
      mipmapFilter: "lanczos4",
      mipmapFilterScale: 1,
      mipmapWrap: "clamp",
    },
    color: {
      assignTf: null,
      convertTf: null,
      assignPrimaries: null,
      convertPrimaries: null,
      assignTexcoordOrigin: null,
      convertTexcoordOrigin: null,
      failOnColorConversions: false,
      warnOnColorConversions: false,
      noWarnOnColorConversions: false,
      failOnOriginChanges: false,
      warnOnOriginChanges: false,
      swizzle: null,
      inputSwizzle: null,
      normalize: owned.normalize,
      normalMode: owned.normalMode,
    },
    basisLz: owned.basisLz,
    uastcLdr: owned.uastcLdr,
    astc: owned.astc,
    encoder: {
      threads: 1,
      noSse: false,
      compareSsim: false,
      comparePsnr: false,
    },
    supercompression: owned.supercompression,
  };
}

describe("encode presets helpers", () => {
  it("keeps the stable default id", () => {
    expect(DEFAULT_ENCODE_PRESET_ID).toBe("official.color.uastc-rgba");
  });

  it("matches owned fields and ignores filter edits", () => {
    const owned = colorOwned();
    const options = optionsFromOwned(owned);
    expect(optionsMatchPreset(options, owned)).toBe(true);
    options.filter.mipmapFilterScale = 3;
    expect(optionsMatchPreset(options, owned)).toBe(true);
    options.supercompression = { kind: "zstd", level: 3 };
    expect(optionsMatchPreset(options, owned)).toBe(false);
  });

  it("groups by category and builds breadcrumbs", () => {
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
