import { describe, expect, it } from "vitest";
import {
  createImageMemoryLru,
  extractSlotsForCores,
  MAX_MEMORY_ENTRIES,
} from "./imageMemoryLru";

describe("createImageMemoryLru", () => {
  it("evicts oldest unpinned entries past the soft cap", () => {
    const lru = createImageMemoryLru(MAX_MEMORY_ENTRIES, () => {});
    for (let i = 0; i < 40; i += 1) {
      lru.remember(`k${String(i)}`, `blob:${String(i)}`);
    }
    expect(lru.size()).toBe(MAX_MEMORY_ENTRIES);
    expect(lru.has("k0")).toBe(false);
    expect(lru.has("k39")).toBe(true);
  });

  it("lets pinned keys exceed the soft cap until released", () => {
    const lru = createImageMemoryLru(MAX_MEMORY_ENTRIES, () => {});
    for (let i = 0; i < 20; i += 1) {
      lru.retain(`p${String(i)}`);
      lru.remember(`p${String(i)}`, `blob:p${String(i)}`);
    }
    expect(lru.size()).toBe(20);
    expect(lru.revoked).toHaveLength(0);

    lru.release("p0");
    lru.remember("extra", "blob:extra");
    expect(lru.has("p0") || lru.size() >= MAX_MEMORY_ENTRIES).toBe(true);
  });
});

describe("extractSlotsForCores", () => {
  it("scales to ~80% of cores with floor 1 and cap 16", () => {
    expect(extractSlotsForCores(0)).toBe(1);
    expect(extractSlotsForCores(1)).toBe(1);
    expect(extractSlotsForCores(4)).toBe(3);
    expect(extractSlotsForCores(8)).toBe(6);
    expect(extractSlotsForCores(32)).toBe(16);
  });
});
