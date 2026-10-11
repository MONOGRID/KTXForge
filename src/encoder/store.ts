import { useDebounceFn } from "@vueuse/core";
import { reactive, watch } from "vue";
import { call } from "../api/commands";
import type { AnimationOptions } from "../bindings/AnimationOptions";
import type { CreateOptions } from "../bindings/CreateOptions";
import type { EncodeResult } from "../bindings/EncodeResult";
import type { EncodeStage } from "../bindings/EncodeStage";
import type { FieldState } from "../bindings/FieldState";
import type { KtxVersion } from "../bindings/KtxVersion";
import type { OptionKey } from "../bindings/OptionKey";
import type { OptionReport } from "../bindings/OptionReport";
import type { SourceInfo } from "../bindings/SourceInfo";

export type JobStatus = "idle" | "running" | "done" | "error";

export type JobState = {
  status: JobStatus;
  jobId: string | null;
  stage: EncodeStage | null;
  stderrLines: string[];
  result: EncodeResult | null;
};

function emptyJob(): JobState {
  return {
    status: "idle",
    jobId: null,
    stage: null,
    stderrLines: [],
    result: null,
  };
}

function emptyReport(): OptionReport {
  return { fields: [], warnings: [] };
}

/** Matches Rust `CreateOptions::default` until `initStore` replaces it. */
function interimOptions(): CreateOptions {
  return {
    format: "R8G8B8A8_SRGB",
    encode: null,
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
      normalize: false,
      normalMode: false,
    },
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
    astc: {
      astcQuality: "medium",
      astcPerceptual: false,
    },
    encoder: {
      threads: 1,
      noSse: false,
      compareSsim: false,
      comparePsnr: false,
    },
    supercompression: { kind: "none" },
  } satisfies CreateOptions;
}

export const store = reactive({
  sources: [] as SourceInfo[],
  options: interimOptions(),
  report: emptyReport(),
  job: emptyJob(),
  animation: null as AnimationOptions | null,
  ktxVersion: null as KtxVersion | null,
  ready: false,
});

const refreshReport = useDebounceFn(async () => {
  if (!store.ready) {
    return;
  }
  try {
    store.report = await call("describe_options", {
      options: store.options,
      sources: store.sources,
    });
  } catch {
    // Keep the last successful report; option sections still render.
  }
}, 80);

watch(
  () => [store.sources, store.options] as const,
  () => {
    void refreshReport();
  },
  { deep: true },
);

/** Rust `describe_options` is the only source of enablement. Missing → hidden. */
export function fieldState(key: OptionKey): FieldState {
  const found = store.report.fields.find((field) => field.key === key);
  if (found !== undefined) {
    return found;
  }
  return { key, enabled: false, reason: "waiting for option rules" };
}

export function isOptionEnabled(key: OptionKey): boolean {
  return fieldState(key).enabled;
}

export function addSources(items: readonly SourceInfo[]): void {
  store.sources.push(...items);
}

export function removeSource(index: number): void {
  if (index < 0 || index >= store.sources.length) {
    return;
  }
  store.sources.splice(index, 1);
}

export function moveSource(index: number, delta: -1 | 1): void {
  const next = index + delta;
  if (
    index < 0 ||
    next < 0 ||
    index >= store.sources.length ||
    next >= store.sources.length
  ) {
    return;
  }
  const sources = store.sources;
  const current = sources[index];
  sources[index] = sources[next];
  sources[next] = current;
}

export async function initStore(): Promise<void> {
  const [options, ktxVersion] = await Promise.all([
    call("default_create_options", {}),
    call("ktx_version", {}),
  ]);
  store.options = options;
  store.ktxVersion = ktxVersion;
  store.ready = true;
  store.report = await call("describe_options", {
    options: store.options,
    sources: store.sources,
  });
}
