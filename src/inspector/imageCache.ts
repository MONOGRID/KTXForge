import { call } from "../api/commands";
import type { ImageSelection } from "../bindings/ImageSelection";
import {
  createImageMemoryLru,
  EXTRACT_SLOTS_MAX,
  extractSlotsForCores,
} from "./imageMemoryLru";

function selectionKey(filePath: string, sel: ImageSelection): string {
  return `${filePath}\0${String(sel.level)}:${String(sel.face)}:${String(sel.layer)}:${String(sel.depthSlice)}`;
}

const memory = createImageMemoryLru();

/** In-flight fetches by key (shared by interactive + prefetch). */
const inflight = new Map<string, Promise<string>>();

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

function rememberUrl(key: string, url: string): void {
  memory.remember(key, url);
}

export function peekInspectorImageUrl(
  filePath: string,
  sel: ImageSelection,
): string | null {
  return memory.get(selectionKey(filePath, sel)) ?? null;
}

/** Mark a cached URL as most-recently used (call when binding to the viewport). */
export function touchInspectorImageUrl(
  filePath: string,
  sel: ImageSelection,
): void {
  const key = selectionKey(filePath, sel);
  const url = memory.get(key);
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
  memory.retain(selectionKey(filePath, sel));
}

export function releaseInspectorImage(
  filePath: string,
  sel: ImageSelection,
): void {
  memory.release(selectionKey(filePath, sel));
}

export function clearInspectorImageCache(): void {
  memory.clear(true);
  inflight.clear();
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
  const cached = memory.get(key);
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
function prefetchInspectorImage(
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
