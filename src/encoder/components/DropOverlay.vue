<script setup lang="ts">
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { onMounted, onUnmounted, ref } from "vue";

const active = ref(false);
let unlisten: (() => void) | undefined;

onMounted(() => {
  void getCurrentWebview()
    .onDragDropEvent((event) => {
      const type = event.payload.type;
      if (type === "enter" || type === "over") {
        active.value = true;
        return;
      }
      // leave / drop — reset highlight; Rust owns path handling on drop
      active.value = false;
    })
    .then((fn) => {
      unlisten = fn;
    });
});

onUnmounted(() => {
  unlisten?.();
});
</script>

<template>
  <div
    class="pointer-events-none fixed inset-0 z-50 border-4 border-transparent transition-colors"
    :class="{ 'border-accent bg-accent/10': active }"
    aria-hidden="true"
  />
</template>
