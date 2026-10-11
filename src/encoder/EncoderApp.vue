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
import { TooltipProvider } from "@/components/ui/tooltip";
import { toAppError } from "../api/guards";
import ModeToggle from "../shared/components/ModeToggle.vue";
import UpdateBanner from "../shared/components/UpdateBanner.vue";
import { toastAppError } from "../shared/toastAppError";
import DropOverlay from "./components/DropOverlay.vue";
import EncodeBar from "./components/EncodeBar.vue";
import PreviewPane from "./components/PreviewPane.vue";
import SourceList from "./components/SourceList.vue";
import OptionsPanel from "./components/options/OptionsPanel.vue";
import { bindEncoderMenu } from "./menuBridge";
import { initStore, store } from "./store";

useTitle("KTX-GUI");

const versionLabel = computed((): string => {
  const version = store.ktxVersion;
  if (version === null) {
    return "…";
  }
  return version.pinned;
});

let unbindMenu: (() => void) | undefined;

onMounted(() => {
  unbindMenu = bindEncoderMenu();
  void initStore().catch((err: unknown) => {
    toastAppError(toAppError(err));
  });
});

onUnmounted(() => {
  unbindMenu?.();
});
</script>

<template>
  <TooltipProvider :delay-duration="400">
    <div class="box-border flex h-screen flex-col overflow-hidden font-sans text-ink">
      <Toaster
        rich-colors
        close-button
      />
      <DropOverlay />

      <header class="flex shrink-0 items-center justify-between gap-3 border-b border-line bg-surface-raised px-4 py-2">
        <h1 class="m-0 text-lg font-semibold tracking-tight">
          KTX-GUI
        </h1>
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
            <SourceList class="h-full min-h-0" />
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
