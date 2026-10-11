<script setup lang="ts">
import { useTitle } from "@vueuse/core";
import { computed, onMounted, onUnmounted } from "vue";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { Toaster } from "@/components/ui/sonner";
import ExportPanel from "./components/ExportPanel.vue";
import InspectorInfoSidebar from "./components/InspectorInfoSidebar.vue";
import FramePlayer from "./components/viewer/FramePlayer.vue";
import ImageViewer from "./components/viewer/ImageViewer.vue";
import MipStrip from "./components/viewer/MipStrip.vue";
import SubresourceSelectors from "./components/viewer/SubresourceSelectors.vue";
import { bindInspectorMenu } from "./menuBridge";
import {
  displayValue,
  initInspectorStore,
  ktx2Info,
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
          <InspectorInfoSidebar
            :status-valid="statusValid"
            :status-messages="statusMessages"
            :preview-label="previewLabel"
          />
        </ResizablePanel>
      </ResizablePanelGroup>
    </main>
  </div>
</template>
