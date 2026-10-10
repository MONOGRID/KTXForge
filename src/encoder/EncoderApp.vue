<script setup lang="ts">
import { useTitle } from "@vueuse/core";
import { computed, onMounted, ref } from "vue";
import UpdateBanner from "../shared/components/UpdateBanner.vue";
import DropOverlay from "./components/DropOverlay.vue";
import EncodeBar from "./components/EncodeBar.vue";
import PreviewPane from "./components/PreviewPane.vue";
import SourceList from "./components/SourceList.vue";
import OptionsPanel from "./components/options/OptionsPanel.vue";
import { initStore, store } from "./store";

useTitle("KTX-GUI");

const initError = ref<string | null>(null);

const versionLabel = computed((): string => {
  const version = store.ktxVersion;
  if (version === null) {
    return "…";
  }
  return version.pinned;
});

onMounted(() => {
  void initStore().catch((err: unknown) => {
    initError.value =
      err instanceof Error ? err.message : "Failed to initialize encoder";
  });
});
</script>

<template>
  <div class="box-border flex h-screen flex-col overflow-hidden font-sans text-ink">
    <DropOverlay />

    <header class="flex shrink-0 items-center justify-between gap-3 border-b border-line bg-surface-raised px-4 py-2">
      <h1 class="m-0 text-lg font-semibold tracking-tight">
        KTX-GUI
      </h1>
      <span class="badge font-mono">
        ktx {{ versionLabel }}
      </span>
    </header>

    <UpdateBanner />

    <p
      v-if="initError"
      class="m-0 border-b border-danger/40 bg-danger/10 px-4 py-2 text-sm text-danger"
      role="alert"
    >
      {{ initError }}
    </p>

    <main class="grid min-h-0 flex-1 grid-cols-[minmax(220px,280px)_minmax(0,1fr)_minmax(220px,300px)] gap-3 p-3">
      <SourceList class="min-h-0" />

      <section class="flex min-h-0 flex-col gap-2 overflow-hidden">
        <div class="min-h-0 flex-1 overflow-auto">
          <OptionsPanel />
        </div>
        <EncodeBar />
      </section>

      <PreviewPane class="min-h-0" />
    </main>
  </div>
</template>
