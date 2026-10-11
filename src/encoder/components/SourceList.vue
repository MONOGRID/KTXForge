<script setup lang="ts">
import { useTimeoutFn } from "@vueuse/core";
import { onMounted, onUnmounted, reactive, watch } from "vue";
import { ScrollArea } from "@/components/ui/scroll-area";
import { call } from "../../api/commands";
import { on } from "../../api/events";
import type { SourceInfo } from "../../bindings/SourceInfo";
import { openSources } from "../actions";
import {
  addSources,
  moveSource,
  removeSource,
  store,
} from "../store";

const thumbnails = reactive(new Map<string, string>());
const notice = reactive<{ message: string | null }>({ message: null });

const { start: startNoticeTimer, stop: stopNoticeTimer } = useTimeoutFn(
  () => {
    notice.message = null;
  },
  6000,
  { immediate: false },
);

function showNotice(message: string): void {
  notice.message = message;
  stopNoticeTimer();
  startNoticeTimer();
}

function revokeThumbnail(path: string): void {
  const url = thumbnails.get(path);
  if (url !== undefined) {
    URL.revokeObjectURL(url);
    thumbnails.delete(path);
  }
}

async function loadThumbnail(source: SourceInfo): Promise<void> {
  if (thumbnails.has(source.path)) {
    return;
  }
  try {
    const bytes = await call("source_thumbnail", { path: source.path });
    const url = URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
    thumbnails.set(source.path, url);
  } catch {
    // Leave the slot empty; list still shows name and size.
  }
}

watch(
  () => store.sources.map((s) => s.path),
  (paths, prev) => {
    const next = new Set(paths);
    for (const path of prev ?? []) {
      if (!next.has(path)) {
        revokeThumbnail(path);
      }
    }
    for (const source of store.sources) {
      void loadThumbnail(source);
    }
  },
  { immediate: true },
);

const unlisteners: Array<() => void> = [];

onMounted(() => {
  void on("encoder://sources-added", (sources) => {
    addSources(sources);
  }).then((unlisten) => {
    unlisteners.push(unlisten);
  });

  void on("encoder://drop-rejected", (payload) => {
    const paths =
      payload.paths.length > 0 ? ` (${payload.paths.join(", ")})` : "";
    showNotice(`${payload.reason}${paths}`);
  }).then((unlisten) => {
    unlisteners.push(unlisten);
  });
});

onUnmounted(() => {
  for (const unlisten of unlisteners) {
    unlisten();
  }
  for (const path of [...thumbnails.keys()]) {
    revokeThumbnail(path);
  }
  stopNoticeTimer();
});

function onAddFiles(): void {
  void openSources();
}
</script>

<template>
  <section class="panel flex h-full min-h-0 flex-col overflow-hidden">
    <header class="flex items-center justify-between gap-2 border-b border-line px-3 py-2">
      <h2 class="m-0 text-sm font-semibold">
        Sources
      </h2>
      <button
        type="button"
        class="btn"
        @click="onAddFiles"
      >
        Add files…
      </button>
    </header>

    <p
      v-if="notice.message"
      class="m-0 border-b border-danger/40 bg-danger/10 px-3 py-2 text-xs text-danger"
      role="status"
    >
      {{ notice.message }}
    </p>

    <ScrollArea
      v-if="store.sources.length > 0"
      class="min-h-0 flex-1"
    >
      <ul class="m-0 list-none p-2 pr-3">
        <li
          v-for="(source, index) in store.sources"
          :key="source.path"
          class="mb-1 flex items-center gap-2 rounded border border-line bg-surface px-2 py-1.5 last:mb-0"
        >
          <div class="h-10 w-10 shrink-0 overflow-hidden rounded bg-surface-sunken">
            <img
              v-if="thumbnails.get(source.path)"
              :src="thumbnails.get(source.path)"
              :alt="source.name"
              class="h-full w-full object-contain"
            >
          </div>
          <div class="min-w-0 flex-1">
            <div class="truncate text-sm font-medium">
              {{ source.name }}
            </div>
            <div class="hint">
              {{ source.width }}×{{ source.height }} · {{ source.kind }}
              <span v-if="source.needsConversion"> · convert</span>
            </div>
          </div>
          <div class="flex shrink-0 flex-col gap-0.5">
            <button
              type="button"
              class="btn px-1.5 py-0.5 text-xs"
              :disabled="index === 0"
              aria-label="Move up"
              @click="moveSource(index, -1)"
            >
              ↑
            </button>
            <button
              type="button"
              class="btn px-1.5 py-0.5 text-xs"
              :disabled="index === store.sources.length - 1"
              aria-label="Move down"
              @click="moveSource(index, 1)"
            >
              ↓
            </button>
          </div>
          <button
            type="button"
            class="btn shrink-0 px-1.5 py-0.5 text-xs"
            aria-label="Remove"
            @click="removeSource(index)"
          >
            ✕
          </button>
        </li>
      </ul>
    </ScrollArea>

    <p
      v-else
      class="m-0 flex flex-1 items-center justify-center px-4 text-center text-sm text-ink-muted"
    >
      Drop images here or use Add files…
    </p>
  </section>
</template>
