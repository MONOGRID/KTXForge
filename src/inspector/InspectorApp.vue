<script setup lang="ts">
import { useTitle } from "@vueuse/core";
import { computed, onMounted, onUnmounted } from "vue";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Eye } from "@lucide/vue";
import { Switch } from "@/components/ui/switch";
import ModeToggle from "../shared/components/ModeToggle.vue";
import ExportPanel from "./components/ExportPanel.vue";
import InspectorInfoSidebar from "./components/InspectorInfoSidebar.vue";
import FramePlayer from "./components/viewer/FramePlayer.vue";
import ImageViewer from "./components/viewer/ImageViewer.vue";
import MipStrip from "./components/viewer/MipStrip.vue";
import SubresourceSelectors from "./components/viewer/SubresourceSelectors.vue";
import { bindMenuShortcuts } from "../shared/menuShortcuts";
import { bindInspectorMenu } from "./menuBridge";
import {
  displayValue,
  flipY,
  initInspectorStore,
  ktx2Info,
  loading,
  path,
  report,
} from "./store";

const title = computed((): string => {
  if (path.value === null || path.value.length === 0) {
    return "KTXForge Inspector";
  }
  const base = path.value.replace(/\\/g, "/").split("/").pop() ?? path.value;
  return `KTXForge Inspector — ${base}`;
});

useTitle(title);

const statusValid = computed((): boolean | null => ktx2Info.value?.valid ?? null);
const statusMessages = computed(() => ktx2Info.value?.messages ?? null);
const pathLabel = computed((): string => displayValue(path.value));
const fileSizeLabel = computed((): string => displayValue(report.value?.fileSize));
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
const isLoading = computed((): boolean => loading.value);

let unbindMenu: (() => void) | undefined;
let unbindShortcuts: (() => void) | undefined;

onMounted(() => {
  unbindMenu = bindInspectorMenu();
  unbindShortcuts = bindMenuShortcuts();
  void initInspectorStore();
});

onUnmounted(() => {
  unbindMenu?.();
  unbindShortcuts?.();
});
</script>

<template>
  <TooltipProvider :delay-duration="400">
    <div class="box-border flex h-screen flex-col overflow-hidden font-sans text-ink">
      <header
        class="flex shrink-0 items-center justify-between gap-3 border-b border-line/40 bg-surface-raised px-4 py-2 shadow-sm"
      >
        <div class="flex min-w-0 items-center gap-3">
          <div class="flex shrink-0 items-center gap-2">
            <img
              src="/favicon-32.png"
              alt=""
              width="28"
              height="28"
              class="size-7 shrink-0"
            >
            <h1 class="m-0 text-lg font-semibold tracking-tight">
              KTXForge
            </h1>
          </div>
          <div class="min-w-0 border-l border-line/40 pl-3">
            <p
              class="m-0 truncate font-mono text-xs text-ink-muted"
              :title="pathLabel"
            >
              {{ pathLabel }}
            </p>
            <p class="m-0 mt-0.5 text-xs text-ink-faint">
              fileSize: {{ fileSizeLabel }} · preview: {{ previewLabel }}
              <span
                v-if="isLoading"
                class="text-ink-muted"
              > · Loading…</span>
            </p>
          </div>
        </div>
        <ModeToggle />
      </header>

      <main class="flex min-h-0 flex-1 flex-col gap-2 bg-surface p-2.5">
        <ResizablePanelGroup
          id="inspector-layout"
          auto-save-id="inspector-layout"
          direction="horizontal"
          class="min-h-0 flex-1"
        >
          <ResizablePanel
            id="inspector-viewer"
            :default-size="58"
            :min-size="35"
            class="min-h-0 min-w-0"
          >
            <section
              class="panel flex h-full min-h-0 flex-col overflow-hidden"
              aria-label="Image viewer"
            >
              <header class="flex shrink-0 items-center justify-between gap-3 border-b border-line/50 px-3 py-2">
                <h2 class="m-0 flex items-center gap-1.5 text-sm font-semibold">
                  <Eye
                    class="size-4 shrink-0 text-ink-muted"
                    aria-hidden="true"
                  />
                  Viewer
                </h2>
                <label
                  class="flex cursor-pointer items-center gap-2 text-xs text-ink-muted"
                  title="Flip the preview vertically (KTX / OpenGL-style origin)"
                >
                  <span>Flip Y</span>
                  <Switch
                    :checked="flipY"
                    @update:checked="flipY = $event"
                  />
                </label>
              </header>
              <div class="flex min-h-0 flex-1 flex-col gap-2 p-3">
                <div class="flex shrink-0 flex-col gap-2">
                  <MipStrip />
                  <SubresourceSelectors />
                  <FramePlayer />
                </div>
                <div class="min-h-0 flex-1">
                  <ImageViewer />
                </div>
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
            />
          </ResizablePanel>
        </ResizablePanelGroup>

        <ExportPanel class="shrink-0" />
      </main>
    </div>
  </TooltipProvider>
</template>
