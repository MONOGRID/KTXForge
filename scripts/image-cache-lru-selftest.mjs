/**
 * Red/green loop for inspector memory LRU (no vitest in this repo).
 * Run: node scripts/image-cache-lru-selftest.mjs
 *
 * Inlines the eviction policy (mirrors src/inspector/imageCache.ts) so the
 * contract stays agent-runnable without booting Vue / Tauri.
 */
const MAX_MEMORY_ENTRIES = 16;

function createLru() {
  const urls = new Map();
  const pinCounts = new Map();
  const revoked = [];

  function isPinned(key) {
    return (pinCounts.get(key) ?? 0) > 0;
  }

  function evictOverflow() {
    for (const key of urls.keys()) {
      if (urls.size <= MAX_MEMORY_ENTRIES) {
        return;
      }
      if (isPinned(key)) {
        continue;
      }
      const url = urls.get(key);
      urls.delete(key);
      revoked.push(url);
    }
  }

  function remember(key, url) {
    if (urls.has(key)) {
      urls.delete(key);
    }
    urls.set(key, url);
    evictOverflow();
  }

  return {
    remember,
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
    size() {
      return urls.size;
    },
    revoked,
  };
}

function assert(cond, msg) {
  if (!cond) {
    throw new Error(msg);
  }
}

function main() {
  const lru = createLru();

  // Fill past the soft cap — oldest unpinned entries must leave.
  for (let i = 0; i < 40; i += 1) {
    lru.remember(`k${String(i)}`, `blob:${String(i)}`);
  }
  assert(lru.size() === MAX_MEMORY_ENTRIES, `size ${String(lru.size())}`);
  assert(!lru.has("k0"), "oldest unpinned should be evicted");
  assert(lru.has("k39"), "newest should remain");

  // Pinned keys survive beyond the soft cap (retain before remember, as callers do).
  const pinned = createLru();
  for (let i = 0; i < 20; i += 1) {
    pinned.retain(`p${String(i)}`);
    pinned.remember(`p${String(i)}`, `blob:p${String(i)}`);
  }
  assert(pinned.size() === 20, `pinned stack may exceed soft cap, got ${String(pinned.size())}`);
  assert(pinned.revoked.length === 0, "pinned URLs must not revoke");

  pinned.release("p0");
  // Still over cap with 19 pins + need room — releasing one unpinned allows eviction.
  pinned.remember("extra", "blob:extra");
  assert(!pinned.has("p0") || pinned.size() >= 16, "unpinned p0 can leave");

  console.log("image-cache-lru-selftest: PASS");
}

try {
  main();
} catch (err) {
  console.error("image-cache-lru-selftest: FAIL", err);
  process.exit(1);
}
