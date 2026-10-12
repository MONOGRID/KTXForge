/**
 * Soft-cap LRU for inspector preview object URLs.
 * Pinned keys (active viewport / volume planes) survive past the cap.
 */

export const MAX_MEMORY_ENTRIES = 32;

/** Match Rust `EXTRACT_SLOTS_MAX` — large hosts must not spawn endless decompressors. */
export const EXTRACT_SLOTS_MAX = 16;

/**
 * Same formula as Rust `extract_slots_for_cores`: ~80% of logical cores, ≥1, ≤16.
 */
export function extractSlotsForCores(cores: number): number {
  const n = Number.isFinite(cores) && cores > 0 ? Math.trunc(cores) : 1;
  return Math.min(EXTRACT_SLOTS_MAX, Math.max(1, Math.floor((n * 4) / 5)));
}

export type ImageMemoryLru = {
  remember: (key: string, url: string) => void;
  retain: (key: string) => void;
  release: (key: string) => void;
  has: (key: string) => boolean;
  get: (key: string) => string | undefined;
  size: () => number;
  clear: (revokeAll?: boolean) => void;
  /** URLs passed to `revoke` during eviction (for tests). */
  revoked: string[];
};

export function createImageMemoryLru(
  maxEntries: number = MAX_MEMORY_ENTRIES,
  revoke: (url: string) => void = (url) => {
    URL.revokeObjectURL(url);
  },
): ImageMemoryLru {
  const urls = new Map<string, string>();
  const pinCounts = new Map<string, number>();
  const revoked: string[] = [];

  function isPinned(key: string): boolean {
    return (pinCounts.get(key) ?? 0) > 0;
  }

  function evictOverflow(): void {
    for (const key of urls.keys()) {
      if (urls.size <= maxEntries) {
        return;
      }
      if (isPinned(key)) {
        continue;
      }
      const url = urls.get(key);
      urls.delete(key);
      if (url === undefined) {
        continue;
      }
      revoke(url);
      revoked.push(url);
    }
  }

  return {
    remember(key, url) {
      if (urls.has(key)) {
        urls.delete(key);
      }
      urls.set(key, url);
      evictOverflow();
    },
    retain(key) {
      pinCounts.set(key, (pinCounts.get(key) ?? 0) + 1);
    },
    release(key) {
      const prev = pinCounts.get(key) ?? 0;
      if (prev <= 1) {
        pinCounts.delete(key);
      } else {
        pinCounts.set(key, prev - 1);
      }
      evictOverflow();
    },
    has(key) {
      return urls.has(key);
    },
    get(key) {
      return urls.get(key);
    },
    size() {
      return urls.size;
    },
    clear(revokeAll = true) {
      if (revokeAll) {
        for (const url of urls.values()) {
          revoke(url);
          revoked.push(url);
        }
      }
      urls.clear();
      pinCounts.clear();
    },
    revoked,
  };
}
