<script setup lang="ts">
import { useTitle } from "@vueuse/core";
import { computed, onMounted, onUnmounted } from "vue";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Toaster } from "@/components/ui/sonner";
import ModeToggle from "../shared/components/ModeToggle.vue";
import ExportPanel from "./components/ExportPanel.vue";
import BasisGlobalDataPanel from "./components/report/BasisGlobalDataPanel.vue";
import DfdPanel from "./components/report/DfdPanel.vue";
import HeaderPanel from "./components/report/HeaderPanel.vue";
import IndexPanel from "./components/report/IndexPanel.vue";
import KeyValueTable from "./components/report/KeyValueTable.vue";
import KnownMetadataPanel from "./components/report/KnownMetadataPanel.vue";
import Ktx1TextPanel from "./components/report/Ktx1TextPanel.vue";
import LevelIndexTable from "./components/report/LevelIndexTable.vue";
import StatusPanel from "./components/report/StatusPanel.vue";
import FramePlayer from "./components/viewer/FramePlayer.vue";
import ImageViewer from "./components/viewer/ImageViewer.vue";
import MipStrip from "./components/viewer/MipStrip.vue";
import SubresourceSelectors from "./components/viewer/SubresourceSelectors.vue";
import { bindInspectorMenu } from "./menuBridge";
import {
  displayValue,
  initInspectorStore,
  ktx1Header,
  ktx1Text,
  ktx2Info,
  loading,
  path,
  report,
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

let unbindMenu: (() => void) | undefined;

onMounted(() => {
  unbindMenu = bindInspectorMenu();
  void initInspectorStore();
});

onUnmounted(() => {
  unbindMenu?.();
});
</script>

<template>
  <div class="box-border flex h-screen flex-col overflow-hidden font-sans text-ink">
    <Toaster
      rich-colors
      close-button
    />

    <main class="min-h-0 flex-1 p-2">
      <ResizablePanelGroup
        id="inspector-layout"
        auto-save-id="inspector-layout"
        direction="horizontal"
        class="h-full min-h-0"
      >
        <ResizablePanel
          id="inspector-viewer"
          :default-size="58"
          :min-size="35"
          class="min-h-0 min-w-0"
        >
          <section
            class="flex h-full min-h-0 flex-col gap-2 overflow-hidden"
            aria-label="Image viewer"
          >
            <div class="flex shrink-0 flex-col gap-2 px-1">
              <MipStrip />
              <SubresourceSelectors />
              <FramePlayer />
            </div>
            <div class="min-h-0 flex-1">
              <ImageViewer />
            </div>
            <div class="shrink-0">
              <ExportPanel />
            </div>
          </section>
        </ResizablePanel>

        <ResizableHandle
          with-handle
          class="mx-1"
        />

        <ResizablePanel
          id="inspector-info"
          :default-size="42"
          :min-size="22"
          :max-size="60"
          class="min-h-0 min-w-0"
        >
          <section class="flex h-full min-h-0 flex-col gap-2 overflow-hidden">
            <header class="panel shrink-0 px-3 py-2">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <p class="m-0 break-all font-mono text-xs text-ink-muted">
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
                </div>
                <ModeToggle />
              </div>
            </header>

            <ScrollArea class="min-h-0 flex-1">
              <div class="flex flex-col gap-3 pr-3">
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
              </div>
            </ScrollArea>
          </section>
        </ResizablePanel>
      </ResizablePanelGroup>
    </main>
  </div>
</template>
