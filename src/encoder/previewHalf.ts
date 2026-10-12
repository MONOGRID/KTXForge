import { ref, type Ref } from "vue";
import type { InputRole } from "../bindings/InputRole";
import { createScrubGate, type ScrubGate } from "../shared/scrubGate";

function revoke(url: string | null): void {
  if (url !== null) {
    URL.revokeObjectURL(url);
  }
}

function revokeList(urls: ReadonlyArray<string | null>): void {
  for (const url of urls) {
    revoke(url);
  }
}

function emptyFaces(): Array<string | null> {
  return [null, null, null, null, null, null];
}

export function replaceUrl(
  target: { value: string | null },
  next: string,
): void {
  const prev = target.value;
  target.value = next;
  if (prev !== null && prev !== next) {
    revoke(prev);
  }
}

export function replaceUrlList(
  target: { value: Array<string | null> },
  next: Array<string | null>,
): void {
  const prev = target.value;
  target.value = next;
  revokeList(prev);
}

export type PreviewHalf = {
  url: Ref<string | null>;
  faceUrls: Ref<Array<string | null>>;
  depthUrls: Ref<Array<string | null>>;
  loading: Ref<boolean>;
  gate: ScrubGate;
  /** Invalidate in-flight work and clear surfaces; returns the new generation. */
  beginReplace: () => number;
  /** Current generation (scrub keeps surfaces; does not bump). */
  generation: () => number;
  isCurrent: (gen: number) => boolean;
  clearSurfaces: () => void;
  wipe: () => void;
  needsLoading: (role: InputRole) => boolean;
  revokeIfStale: (gen: number, urls: ReadonlyArray<string | null>) => boolean;
  revokeUrlIfStale: (gen: number, url: string) => boolean;
};

/** One side of the source/encoded preview (blob URLs, scrub gate, generation). */
export function createPreviewHalf(): PreviewHalf {
  const url = ref<string | null>(null);
  const faceUrls = ref<Array<string | null>>(emptyFaces());
  const depthUrls = ref<Array<string | null>>([]);
  const loading = ref(false);
  const gate = createScrubGate();
  let gen = 0;

  function clearSurfaces(): void {
    revoke(url.value);
    url.value = null;
    revokeList(faceUrls.value);
    faceUrls.value = emptyFaces();
    revokeList(depthUrls.value);
    depthUrls.value = [];
  }

  function beginReplace(): number {
    gen += 1;
    gate.clearPending();
    clearSurfaces();
    return gen;
  }

  function wipe(): void {
    beginReplace();
    loading.value = false;
  }

  function needsLoading(role: InputRole): boolean {
    if (role === "depthSlices") {
      return !depthUrls.value.some((u) => u !== null);
    }
    return url.value === null;
  }

  function isCurrent(candidate: number): boolean {
    return candidate === gen;
  }

  function revokeIfStale(
    candidate: number,
    urls: ReadonlyArray<string | null>,
  ): boolean {
    if (isCurrent(candidate)) {
      return false;
    }
    revokeList(urls);
    return true;
  }

  function revokeUrlIfStale(candidate: number, blobUrl: string): boolean {
    if (isCurrent(candidate)) {
      return false;
    }
    revoke(blobUrl);
    return true;
  }

  return {
    url,
    faceUrls,
    depthUrls,
    loading,
    gate,
    beginReplace,
    generation: () => gen,
    isCurrent,
    clearSurfaces,
    wipe,
    needsLoading,
    revokeIfStale,
    revokeUrlIfStale,
  };
}

function depthStackReady(
  urls: ReadonlyArray<string | null>,
  expected: number,
): boolean {
  return urls.length === expected && urls.some((u) => u !== null);
}

export function skipDepthScrub(
  mode: "replace" | "scrub",
  role: InputRole,
  urls: ReadonlyArray<string | null>,
  expected: number,
): boolean {
  return (
    mode === "scrub" &&
    role === "depthSlices" &&
    depthStackReady(urls, expected)
  );
}

export function pickListUrl(
  urls: ReadonlyArray<string | null>,
  index: number,
): string | null {
  if (index < 0 || index >= urls.length) {
    return null;
  }
  return urls[index] ?? null;
}

export function bytesToUrl(bytes: ArrayBuffer): string {
  return URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
}
