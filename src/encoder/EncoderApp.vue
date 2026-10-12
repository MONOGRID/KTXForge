<script setup lang="ts">
import { useTitle } from "@vueuse/core";
import { computed, onMounted, onUnmounted } from "vue";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { ScrollArea } from "@/components/ui/scroll-area";
import { TooltipProvider } from "@/components/ui/tooltip";
import { on } from "../api/events";
import { toAppError } from "../api/guards";
import {
  alertAppError,
  alertInvalidInput,
} from "../shared/alertAppError";
import ModeToggle from "../shared/components/ModeToggle.vue";
import UpdateBanner from "../shared/components/UpdateBanner.vue";
import { handleDroppedSources } from "./actions";
import DropIntentDialog from "./components/DropIntentDialog.vue";
import DropOverlay from "./components/DropOverlay.vue";
import EncodeBar from "./components/EncodeBar.vue";
import InputsPanel from "./components/InputsPanel.vue";
import OutputsQueue from "./components/OutputsQueue.vue";
import PreviewPane from "./components/PreviewPane.vue";
import QueueModal from "./components/QueueModal.vue";
import OptionsPanel from "./components/options/OptionsPanel.vue";
import { consumeCommittedDropZone } from "./dropTarget";
import { bindWindowsMenuShortcuts } from "../shared/menuShortcuts";
import { bindEncoderMenu } from "./menuBridge";
import { initStore, showInputsPanel, store } from "./store";

useTitle("KTXForge");

const versionLabel = computed((): string => {
  const version = store.ktxVersion;
  if (version === null) {
    return "…";
  }
  return version.pinned;
});

let unbindMenu: (() => void) | undefined;
let unbindShortcuts: (() => void) | undefined;
const dropUnlisteners: Array<() => void> = [];

onMounted(() => {
  unbindMenu = bindEncoderMenu();
  unbindShortcuts = bindWindowsMenuShortcuts();
  void initStore().catch((err: unknown) => {
    void alertAppError(toAppError(err));
  });
  void on("encoder://sources-added", (sources) => {
    void handleDroppedSources(sources, consumeCommittedDropZone());
  }).then((unlisten) => {
    dropUnlisteners.push(unlisten);
  });
  void on("encoder://drop-rejected", (payload) => {
    const paths =
      payload.paths.length > 0 ? ` (${payload.paths.join(", ")})` : "";
    void alertInvalidInput(`${payload.reason}${paths}`);
  }).then((unlisten) => {
    dropUnlisteners.push(unlisten);
  });
});

onUnmounted(() => {
  unbindMenu?.();
  unbindShortcuts?.();
  for (const unlisten of dropUnlisteners) {
    unlisten();
  }
});
</script>

<template>
  <TooltipProvider :delay-duration="400">
    <div class="box-border flex h-screen flex-col overflow-hidden font-sans text-ink">
      <DropOverlay />
      <DropIntentDialog />
      <QueueModal />

      <header class="flex shrink-0 items-center justify-between gap-3 border-b border-line bg-surface-raised px-4 py-2">
        <div class="flex min-w-0 items-center gap-2">
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
        <div class="flex items-center gap-2">
          <span class="badge font-mono">
            ktx {{ versionLabel }}
          </span>
          <ModeToggle />
        </div>
      </header>

      <UpdateBanner />

      <main class="min-h-0 flex-1 p-2">
        <ResizablePanelGroup
          id="encoder-layout"
          auto-save-id="encoder-layout"
          direction="horizontal"
          class="h-full min-h-0"
        >
          <ResizablePanel
            id="encoder-sources"
            :default-size="22"
            :min-size="14"
            :max-size="40"
            class="min-h-0 min-w-0"
          >
            <ResizablePanelGroup
              v-if="showInputsPanel"
              id="encoder-left"
              auto-save-id="encoder-left"
              direction="vertical"
              class="h-full min-h-0"
            >
              <ResizablePanel
                id="encoder-outputs"
                :default-size="45"
                :min-size="20"
                class="min-h-0 min-w-0"
              >
                <OutputsQueue class="h-full min-h-0" />
              </ResizablePanel>
              <ResizableHandle
                with-handle
                class="my-1"
              />
              <ResizablePanel
                id="encoder-inputs"
                :default-size="55"
                :min-size="20"
                class="min-h-0 min-w-0"
              >
                <InputsPanel class="h-full min-h-0" />
              </ResizablePanel>
            </ResizablePanelGroup>
            <OutputsQueue
              v-else
              class="h-full min-h-0"
            />
          </ResizablePanel>

          <ResizableHandle
            with-handle
            class="mx-1"
          />

          <ResizablePanel
            id="encoder-preview"
            :default-size="48"
            :min-size="28"
            class="min-h-0 min-w-0"
          >
            <PreviewPane class="h-full min-h-0" />
          </ResizablePanel>

          <ResizableHandle
            with-handle
            class="mx-1"
          />

          <ResizablePanel
            id="encoder-options"
            :default-size="30"
            :min-size="18"
            :max-size="48"
            class="min-h-0 min-w-0"
          >
            <section class="flex h-full min-h-0 flex-col gap-2 overflow-hidden">
              <ScrollArea class="min-h-0 flex-1">
                <div class="pr-3">
                  <OptionsPanel />
                </div>
              </ScrollArea>
              <EncodeBar />
            </section>
          </ResizablePanel>
        </ResizablePanelGroup>
      </main>
    </div>
  </TooltipProvider>
</template>
