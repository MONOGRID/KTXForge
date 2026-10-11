import { call } from "../api/commands";
import type { ImageSelection } from "../bindings/ImageSelection";

function selectionKey(filePath: string, sel: ImageSelection): string {
  return `${filePath}\0${String(sel.level)}:${String(sel.face)}:${String(sel.layer)}:${String(sel.depthSlice)}`;
}

const urls = new Map<string, string>();
/** In-flight fetches by key (shared by interactive + prefetch). */
const inflight = new Map<string, Promise<string>>();

/** At most one `ktx extract` / IPC at a time (scrubbing must not stampede). */
type PrefetchItem = ImageSelection & { filePath: string };

let fetchBusy = false;
let interactiveQueued:
  | {
      filePath: string;
      sel: ImageSelection;
      resolve: (url: string) => void;
      reject: (err: unknown) => void;
    }
  | null = null;
const prefetchQueue: PrefetchItem[] = [];

export function peekInspectorImageUrl(
  filePath: string,
  sel: ImageSelection,
): string | null {
  return urls.get(selectionKey(filePath, sel)) ?? null;
}

export function clearInspectorImageCache(): void {
  for (const url of urls.values()) {
    URL.revokeObjectURL(url);
  }
  urls.clear();
  inflight.clear();
  interactiveQueued = null;
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
    urls.set(key, url);
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
  fetchBusy = true;
  void work.finally(() => {
    fetchBusy = false;
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

function pumpFetches(): void {
  if (fetchBusy) {
    return;
  }

  const interactive = interactiveQueued;
  if (interactive !== null) {
    interactiveQueued = null;
    beginFetch(
      runFetch(interactive.filePath, interactive.sel).then(
        interactive.resolve,
        interactive.reject,
      ),
    );
    return;
  }

  const next = takeNextPrefetch();
  if (next === null) {
    return;
  }
  beginFetch(
    runFetch(next.filePath, next).catch(() => {
      // Prefetch failures are ignored; the viewer surfaces real load errors.
    }),
  );
}

/**
 * Resolve an object URL for a subresource.
 * Concurrent interactive requests coalesce to the latest selection only.
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
    // Prefetch work yields to the interactive slider target.
    prefetchQueue.length = 0;
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
 * Enqueue every mip for the current face/layer/depth on the low-priority
 * prefetch queue (interactive slider loads always win and flush this queue).
 */
export function warmInspectorMipChain(
  filePath: string,
  sel: ImageSelection,
  levelCount: number,
): void {
  const base = {
    face: sel.face,
    layer: sel.layer,
    depthSlice: sel.depthSlice,
  };

  for (let level = 0; level < levelCount; level += 1) {
    prefetchInspectorImage(filePath, { ...base, level });
  }
}
