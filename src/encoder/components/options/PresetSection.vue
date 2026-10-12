<script setup lang="ts">
import { ChevronRight } from "@lucide/vue";
import { computed } from "vue";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  DropdownMenuPortal,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
} from "reka-ui";
import HelpTooltip from "../../../shared/components/HelpTooltip.vue";
import {
  applyEncodePreset,
  presetLabel,
  store,
} from "../../store";
import { presetsByCategory } from "../../presets";

const groups = computed(() => presetsByCategory(store.presets));

const label = computed(() => presetLabel());

const helpText =
  "Built-in encode recipes. Applying a preset fills format, encode, color, codec, and supercompression. Inputs and container shape stay unchanged. Edit an owned field to switch to Custom.";
</script>

<template>
  <div class="flex min-w-0 items-center gap-2">
    <HelpTooltip
      :text="helpText"
      class="shrink-0"
    >
      <span class="text-sm text-ink-muted">Preset</span>
    </HelpTooltip>
    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <Button
          variant="outline"
          class="h-8 min-w-0 max-w-md justify-between gap-2 px-2 font-normal"
          :disabled="!store.ready"
        >
          <span class="min-w-0 truncate text-left">{{ label }}</span>
          <ChevronRight class="size-4 shrink-0 rotate-90 text-ink-muted" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="start"
        class="min-w-56"
      >
        <DropdownMenuSub
          v-for="group in groups"
          :key="group.category"
        >
          <DropdownMenuSubTrigger
            class="flex w-full cursor-default select-none items-center justify-between gap-2 rounded-sm px-2 py-1.5 text-sm outline-none focus:bg-accent data-[state=open]:bg-accent"
          >
            <span class="min-w-0 truncate">{{ group.category }}</span>
            <ChevronRight class="size-4 shrink-0 text-ink-muted" />
          </DropdownMenuSubTrigger>
          <DropdownMenuPortal>
            <DropdownMenuSubContent
              class="z-80 min-w-44 overflow-hidden rounded-md border border-transparent bg-popover p-1 text-popover-foreground shadow-lg"
            >
              <DropdownMenuItem
                v-for="preset in group.presets"
                :key="preset.id"
                class="p-0"
                @click="void applyEncodePreset(preset.id)"
              >
                <HelpTooltip
                  :text="preset.help"
                  side="right"
                  class="w-full"
                >
                  <span class="block w-full px-2 py-1.5">
                    {{ preset.path[1] ?? preset.id }}
                  </span>
                </HelpTooltip>
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuPortal>
        </DropdownMenuSub>
      </DropdownMenuContent>
    </DropdownMenu>
  </div>
</template>
