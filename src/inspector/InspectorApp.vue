<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useTitle } from "@vueuse/core";
import BasisGlobalDataPanel from "./components/report/BasisGlobalDataPanel.vue";
import DfdPanel from "./components/report/DfdPanel.vue";
import HeaderPanel from "./components/report/HeaderPanel.vue";
import IndexPanel from "./components/report/IndexPanel.vue";
import KeyValueTable from "./components/report/KeyValueTable.vue";
import KnownMetadataPanel from "./components/report/KnownMetadataPanel.vue";
import Ktx1TextPanel from "./components/report/Ktx1TextPanel.vue";
import LevelIndexTable from "./components/report/LevelIndexTable.vue";
import StatusPanel from "./components/report/StatusPanel.vue";
import {
  displayValue,
  error,
  initInspectorStore,
  ktx1Header,
  ktx1Text,
  ktx2Info,
  loading,
  path,
  report,
  selection,
} from "./store";

const title = computed((): string => {
  if (path.value === null || path.value.length === 0) {
    return "KTX-GUI Inspector";
  }
  const base = path.value.replace(/\\/g, "/").split("/").pop() ?? path.value;
  return `KTX-GUI Inspector — ${base}`;
});

useTitle(title);

const statusValid = computed((): boolean | null => ktx2Info.value?.valid ?? null);
const statusMessages = computed(() => ktx2Info.value?.messages ?? null);
const previewLabel = computed((): string => {
  const preview = report.value?.preview;
  if (preview === undefined) {
    return displayValue(null);
  }
  if (preview.kind === "unavailable") {
    return `unavailable (${preview.reason})`;
  }
  return preview.kind;
});

onMounted(() => {
  void initInspectorStore();
});
</script>

<template>
  <main class="box-border grid min-h-screen grid-cols-1 gap-4 p-4 font-sans lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
    <section class="flex min-h-0 flex-col gap-3 overflow-auto">
      <header class="panel px-3 py-2">
        <h1 class="m-0 text-lg font-semibold">
          KTX-GUI Inspector
        </h1>
        <p class="m-0 mt-1 break-all font-mono text-xs text-ink-muted">
          {{ displayValue(path) }}
        </p>
        <p class="m-0 mt-1 text-xs text-ink-faint">
          fileSize: {{ displayValue(report?.fileSize) }} · preview: {{ previewLabel }}
        </p>
        <p
          v-if="loading"
          class="m-0 mt-2 text-sm text-ink-muted"
        >
          Loading…
        </p>
        <div
          v-if="error !== null"
          class="mt-2 rounded border border-danger/40 bg-danger/10 p-2 text-sm text-danger"
        >
          {{ error.message }}
        </div>
      </header>

      <StatusPanel
        :valid="statusValid"
        :messages="statusMessages"
      />
      <HeaderPanel
        :ktx2="ktx2Info?.header ?? null"
        :ktx1="ktx1Header"
      />
      <LevelIndexTable :levels="ktx2Info?.index.levels ?? null" />
      <IndexPanel
        :data-format-descriptor="ktx2Info?.index.dataFormatDescriptor ?? null"
        :key-value-data="ktx2Info?.index.keyValueData ?? null"
        :supercompression-global-data="ktx2Info?.index.supercompressionGlobalData ?? null"
      />
      <DfdPanel :descriptor="ktx2Info?.dataFormatDescriptor ?? null" />
      <KnownMetadataPanel :key-value-data="ktx2Info?.keyValueData ?? null" />
      <KeyValueTable :entries="ktx2Info?.keyValueData.other ?? null" />
      <BasisGlobalDataPanel :data="ktx2Info?.supercompressionGlobalData ?? null" />
      <Ktx1TextPanel :text="ktx1Text" />
    </section>

    <!-- T5.6 wires ImageViewer / MipStrip / SubresourceSelectors / FramePlayer here -->
    <section
      class="panel flex min-h-64 flex-col gap-2 p-3"
      aria-label="Image viewer"
    >
      <h2 class="m-0 text-sm font-semibold">
        Image viewer
      </h2>
      <p class="m-0 text-sm text-ink-muted">
        Placeholder until T5.6 (ImageViewer, MipStrip, SubresourceSelectors, FramePlayer).
      </p>
      <p class="m-0 font-mono text-xs text-ink-faint">
        selection: level={{ selection.level }} face={{ selection.face }}
        layer={{ selection.layer }} depthSlice={{ selection.depthSlice }}
      </p>
      <div
        class="mt-2 flex flex-1 items-center justify-center rounded border border-dashed border-line bg-surface-sunken text-sm text-ink-faint"
      >
        No preview component yet
      </div>
    </section>
  </main>
</template>
