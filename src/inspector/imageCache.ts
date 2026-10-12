import { call } from "../api/commands";
import type { ImageSelection } from "../bindings/ImageSelection";

function selectionKey(filePath: string, sel: ImageSelection): string {
  return `${filePath}\0${String(sel.level)}:${String(sel.face)}:${String(sel.layer)}:${String(sel.depthSlice)}`;
}

/**
 * Soft cap on decoded preview blobs in the webview. Disk PNG cache in Rust
 * stays unbounded; revisits rehydrate from disk without keeping every layer hot.
 * ~32 entries covers a mip chain + neighbors without holding a full layer array.
 */
const MAX_MEMORY_ENTRIES = 32;

/** Match Rust `EXTRACT_SLOTS_MAX` — large hosts must not spawn endless decompressors. */
const EXTRACT_SLOTS_MAX = 16;

/**
 * Same formula as Rust `extract_slots_for_cores`: ~80% of logical cores, ≥1, ≤16.
 * Used until `extract_concurrency` confirms the host value.
 */
function extractSlotsForCores(cores: number): number {
  const n = Number.isFinite(cores) && cores > 0 ? Math.trunc(cores) : 1;
  return Math.min(EXTRACT_SLOTS_MAX, Math.max(1, Math.floor((n * 4) / 5)));
}

const urls = new Map<string, string>();
/** In-flight fetches by key (shared by interactive + prefetch). */
const inflight = new Map<string, Promise<string>>();
/** Keys that must not be revoked (active viewport / depth volume planes). */
const pinCounts = new Map<string, number>();

type PrefetchItem = ImageSelection & { filePath: string };

type QueuedLoad = {
  filePath: string;
  sel: ImageSelection;
  resolve: (url: string) => void;
  reject: (err: unknown) => void;
};

let maxInflightFetches = extractSlotsForCores(
  typeof navigator !== "undefined" ? navigator.hardwareConcurrency : 4,
);
let inflightCount = 0;
let interactiveQueued: QueuedLoad | null = null;
/** Non-coalescing loads (depth volume stacks); yield to interactive, outrank prefetch. */
const ensureQueue: QueuedLoad[] = [];
const prefetchQueue: PrefetchItem[] = [];

void call("extract_concurrency", {})
  .then((n) => {
    if (typeof n === "number" && Number.isFinite(n) && n >= 1) {
      maxInflightFetches = Math.min(EXTRACT_SLOTS_MAX, Math.trunc(n));
      pumpFetches();
    }
  })
  .catch(() => {
    // Keep navigator-based fallback when IPC is unavailable (e.g. unit tests).
  });

function isPinned(key: string): boolean {
  return (pinCounts.get(key) ?? 0) > 0;
}

function rememberUrl(key: string, url: string): void {
  // Re-insert so Map iteration order is LRU (oldest = first key).
  if (urls.has(key)) {
    urls.delete(key);
  }
  urls.set(key, url);
  evictOverflow();
}

function evictOverflow(): void {
  for (const key of urls.keys()) {
    if (urls.size <= MAX_MEMORY_ENTRIES) {
      return;
    }
    if (isPinned(key)) {
      continue;
    }
    const url = urls.get(key);
    if (url === undefined) {
      urls.delete(key);
      continue;
    }
    URL.revokeObjectURL(url);
    urls.delete(key);
  }
}

export function peekInspectorImageUrl(
  filePath: string,
  sel: ImageSelection,
): string | null {
  return urls.get(selectionKey(filePath, sel)) ?? null;
}

/** Mark a cached URL as most-recently used (call when binding to the viewport). */
export function touchInspectorImageUrl(
  filePath: string,
  sel: ImageSelection,
): void {
  const key = selectionKey(filePath, sel);
  const url = urls.get(key);
  if (url === undefined) {
    return;
  }
  rememberUrl(key, url);
}

/** Keep an object URL alive across LRU eviction (viewport / volume planes). */
export function retainInspectorImage(
  filePath: string,
  sel: ImageSelection,
): void {
  const key = selectionKey(filePath, sel);
  pinCounts.set(key, (pinCounts.get(key) ?? 0) + 1);
}

export function releaseInspectorImage(
  filePath: string,
  sel: ImageSelection,
): void {
  const key = selectionKey(filePath, sel);
  const prev = pinCounts.get(key) ?? 0;
  if (prev <= 1) {
    pinCounts.delete(key);
  } else {
    pinCounts.set(key, prev - 1);
  }
  evictOverflow();
}

export function clearInspectorImageCache(): void {
  for (const url of urls.values()) {
    URL.revokeObjectURL(url);
  }
  urls.clear();
  inflight.clear();
  pinCounts.clear();
  if (interactiveQueued !== null) {
    interactiveQueued.reject(new DOMException("Cleared", "AbortError"));
    interactiveQueued = null;
  }
  for (const item of ensureQueue) {
    item.reject(new DOMException("Cleared", "AbortError"));
  }
  ensureQueue.length = 0;
  prefetchQueue.length = 0;
}

async function runFetch(
  filePath: string,
  sel: ImageSelection,
): Promise<string> {
  const key = selectionKey(filePath, sel);
  const cached = urls.get(key);
  if (cached !== undefined) {
    return cached;
  }

  const pending = inflight.get(key);
  if (pending !== undefined) {
    return pending;
  }

  const promise = (async (): Promise<string> => {
    const bytes = await call("inspector_image", {
      path: filePath,
      selection: {
        level: sel.level,
        face: sel.face,
        layer: sel.layer,
        depthSlice: sel.depthSlice,
      },
    });
    const url = URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
    rememberUrl(key, url);
    return url;
  })();

  inflight.set(key, promise);
  try {
    return await promise;
  } finally {
    inflight.delete(key);
  }
}

function beginFetch(work: Promise<unknown>): void {
  inflightCount += 1;
  void work.finally(() => {
    inflightCount -= 1;
    pumpFetches();
  });
}

function takeNextPrefetch(): PrefetchItem | null {
  while (prefetchQueue.length > 0) {
    const next = prefetchQueue.shift();
    if (next === undefined) {
      return null;
    }
    if (peekInspectorImageUrl(next.filePath, next) !== null) {
      continue;
    }
    if (inflight.has(selectionKey(next.filePath, next))) {
      continue;
    }
    return next;
  }
  return null;
}

function startQueuedLoad(item: QueuedLoad): void {
  beginFetch(
    runFetch(item.filePath, item.sel).then(item.resolve, item.reject),
  );
}

function takeInteractive(): QueuedLoad | null {
  const item = interactiveQueued;
  interactiveQueued = null;
  return item;
}

function startPrefetchIfAny(): boolean {
  const next = takeNextPrefetch();
  if (next === null) {
    return false;
  }
  beginFetch(
    runFetch(next.filePath, next).catch(() => {
      // Prefetch failures are ignored; the viewer surfaces real load errors.
    }),
  );
  return true;
}

function pumpFetches(): void {
  while (inflightCount < maxInflightFetches) {
    const interactive = takeInteractive();
    if (interactive !== null) {
      startQueuedLoad(interactive);
      continue;
    }

    const ensure = ensureQueue.shift();
    if (ensure !== undefined) {
      startQueuedLoad(ensure);
      continue;
    }

    if (!startPrefetchIfAny()) {
      return;
    }
  }
}

/**
 * Resolve an object URL for a subresource.
 * Concurrent interactive requests coalesce to the latest selection only;
 * up to `maxInflightFetches` IPC/extracts may run (prefetch fills spare slots).
 */
export function loadInspectorImageUrl(
  filePath: string,
  sel: ImageSelection,
): Promise<string> {
  const cached = peekInspectorImageUrl(filePath, sel);
  if (cached !== null) {
    return Promise.resolve(cached);
  }

  const pending = inflight.get(selectionKey(filePath, sel));
  if (pending !== undefined) {
    return pending;
  }

  return new Promise<string>((resolve, reject) => {
    if (interactiveQueued !== null) {
      // Drop superseded scrub targets so we never extract every intermediate layer.
      interactiveQueued.reject(new DOMException("Superseded", "AbortError"));
    }
    interactiveQueued = { filePath, sel, resolve, reject };
    // Interactive takes the next free slot; leave the warm queue running in
    // spare extract slots so background pre-extract can finish while scrubbing.
    pumpFetches();
  });
}

/**
 * Resolve an object URL without scrub coalescing — for loading every depth
 * slice of a 3D volume. Yields to interactive loads; shares the cache/inflight map.
 */
export function ensureInspectorImageUrl(
  filePath: string,
  sel: ImageSelection,
): Promise<string> {
  const cached = peekInspectorImageUrl(filePath, sel);
  if (cached !== null) {
    return Promise.resolve(cached);
  }

  const pending = inflight.get(selectionKey(filePath, sel));
  if (pending !== undefined) {
    return pending;
  }

  // Duplicate queue entries for the same key are fine: later pumps hit cache/inflight.
  return new Promise<string>((resolve, reject) => {
    ensureQueue.push({ filePath, sel, resolve, reject });
    pumpFetches();
  });
}

/** Best-effort warm of a subresource (no throw); never outruns interactive loads. */
export function prefetchInspectorImage(
  filePath: string,
  sel: ImageSelection,
): void {
  if (peekInspectorImageUrl(filePath, sel) !== null) {
    return;
  }
  if (inflight.has(selectionKey(filePath, sel))) {
    return;
  }
  const alreadyQueued = prefetchQueue.some(
    (item) =>
      item.filePath === filePath &&
      item.level === sel.level &&
      item.face === sel.face &&
      item.layer === sel.layer &&
      item.depthSlice === sel.depthSlice,
  );
  if (alreadyQueued) {
    return;
  }
  prefetchQueue.push({ filePath, ...sel });
  pumpFetches();
}

export type SubresourceDims = {
  levelCount: number;
  faceCount: number;
  layerCount: number;
  depthCount: number;
};

/** Prefetch immediate neighbors on each selection axis. */
export function prefetchInspectorNeighbors(
  filePath: string,
  sel: ImageSelection,
  dims: SubresourceDims,
): void {
  const axes = [
    { key: "level" as const, max: dims.levelCount - 1 },
    { key: "face" as const, max: dims.faceCount - 1 },
    { key: "layer" as const, max: dims.layerCount - 1 },
    { key: "depthSlice" as const, max: dims.depthCount - 1 },
  ];

  for (const axis of axes) {
    for (const step of [-1, 1] as const) {
      const next = sel[axis.key] + step;
      if (next >= 0 && next <= axis.max) {
        prefetchInspectorImage(filePath, { ...sel, [axis.key]: next });
      }
    }
  }
}

/**
 * Enqueue selections nearest-first on the low-priority prefetch queue.
 * Interactive loads still win free slots first via `pumpFetches` priority.
 */
function warmAxisNearestFirst(
  filePath: string,
  sel: ImageSelection,
  axis: "level" | "layer",
  count: number,
): void {
  if (count <= 1) {
    return;
  }
  const origin = sel[axis];
  prefetchInspectorImage(filePath, sel);
  for (let dist = 1; dist < count; dist += 1) {
    const hi = origin + dist;
    const lo = origin - dist;
    if (hi < count) {
      prefetchInspectorImage(filePath, { ...sel, [axis]: hi });
    }
    if (lo >= 0) {
      prefetchInspectorImage(filePath, { ...sel, [axis]: lo });
    }
  }
}

/**
 * Enqueue every mip for the current face/layer/depth (nearest level first).
 */
export function warmInspectorMipChain(
  filePath: string,
  sel: ImageSelection,
  levelCount: number,
): void {
  warmAxisNearestFirst(filePath, sel, "level", levelCount);
}

/**
 * Enqueue every array layer / anim frame for the current level/face/depth
 * (nearest layer first) so idle time fills the disk cache ahead of scrubbing.
 */
export function warmInspectorLayerStack(
  filePath: string,
  sel: ImageSelection,
  layerCount: number,
): void {
  warmAxisNearestFirst(filePath, sel, "layer", layerCount);
}
