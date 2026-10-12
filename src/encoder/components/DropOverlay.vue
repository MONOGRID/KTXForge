<script setup lang="ts">
import { ImageUp } from "@lucide/vue";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { onMounted, onUnmounted } from "vue";
import {
  commitDropZone,
  dropHintLabel,
  isFileDragActive,
  setDropZone,
  setFileDragActive,
  type DropZone,
} from "../dropTarget";

let disposed = false;
let unlisten: (() => void) | undefined;

function zoneFromPoint(x: number, y: number): DropZone | null {
  const el = document.elementFromPoint(x, y);
  const zoneEl = el?.closest("[data-drop-zone]");
  const zone = zoneEl?.getAttribute("data-drop-zone");
  if (zone === "outputs" || zone === "inputs" || zone === "center") {
    return zone;
  }
  return null;
}

onMounted(() => {
  void getCurrentWebview()
    .onDragDropEvent((event) => {
      const type = event.payload.type;
      if (type === "enter" || type === "over") {
        setFileDragActive(true);
        const pos = event.payload.position;
        setDropZone(zoneFromPoint(pos.x, pos.y));
        return;
      }
      if (type === "drop") {
        const pos = event.payload.position;
        setDropZone(zoneFromPoint(pos.x, pos.y));
        commitDropZone();
      }
      setFileDragActive(false);
    })
    .then((fn) => {
      if (disposed) {
        fn();
        return;
      }
      unlisten = fn;
    });
});

onUnmounted(() => {
  disposed = true;
  unlisten?.();
});
</script>

<template>
  <div
    v-if="isFileDragActive"
    class="pointer-events-none fixed inset-0 z-50"
    aria-hidden="true"
  >
    <div
      class="absolute bottom-4 left-1/2 flex -translate-x-1/2 items-center gap-1.5 rounded bg-surface-raised px-3 py-1.5 text-xs font-medium shadow"
    >
      <ImageUp
        class="size-3.5 shrink-0"
        aria-hidden="true"
      />
      Drop → {{ dropHintLabel }}
    </div>
  </div>
</template>
