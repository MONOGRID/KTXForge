<script setup lang="ts">
import { computed } from "vue";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { ValidationMessage } from "../../bindings/ValidationMessage";
import ModeToggle from "../../shared/components/ModeToggle.vue";
import {
  displayValue,
  ktx1Header,
  ktx1Text,
  ktx2Info,
  loading,
  path,
  report,
} from "../store";
import BasisGlobalDataPanel from "./report/BasisGlobalDataPanel.vue";
import DfdPanel from "./report/DfdPanel.vue";
import HeaderPanel from "./report/HeaderPanel.vue";
import IndexPanel from "./report/IndexPanel.vue";
import KeyValueTable from "./report/KeyValueTable.vue";
import KnownMetadataPanel from "./report/KnownMetadataPanel.vue";
import Ktx1TextPanel from "./report/Ktx1TextPanel.vue";
import LevelIndexTable from "./report/LevelIndexTable.vue";
import StatusPanel from "./report/StatusPanel.vue";

defineProps<{
  statusValid: boolean | null;
  statusMessages: ValidationMessage[] | null;
  previewLabel: string;
}>();

const pathLabel = computed((): string => displayValue(path.value));
const fileSizeLabel = computed((): string => displayValue(report.value?.fileSize));
const isLoading = computed((): boolean => loading.value);

const ktx2Header = computed(() => ktx2Info.value?.header ?? null);
const levelIndex = computed(() => ktx2Info.value?.index.levels ?? null);
const dataFormatDescriptorOffset = computed(
  () => ktx2Info.value?.index.dataFormatDescriptor ?? null,
);
const keyValueDataOffset = computed(() => ktx2Info.value?.index.keyValueData ?? null);
const supercompressionOffset = computed(
  () => ktx2Info.value?.index.supercompressionGlobalData ?? null,
);
const dataFormatDescriptor = computed(
  () => ktx2Info.value?.dataFormatDescriptor ?? null,
);
const keyValueData = computed(() => ktx2Info.value?.keyValueData ?? null);
const otherKeyValues = computed(() => ktx2Info.value?.keyValueData.other ?? null);
const basisGlobalData = computed(
  () => ktx2Info.value?.supercompressionGlobalData ?? null,
);
const ktx1TextValue = computed(() => ktx1Text.value);
const ktx1HeaderValue = computed(() => ktx1Header.value);
</script>

<template>
  <section class="flex h-full min-h-0 flex-col gap-2 overflow-hidden">
    <header class="panel shrink-0 px-3 py-2">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0">
          <p class="m-0 break-all font-mono text-xs text-ink-muted">
            {{ pathLabel }}
          </p>
          <p class="m-0 mt-1 text-xs text-ink-faint">
            fileSize: {{ fileSizeLabel }} · preview: {{ previewLabel }}
          </p>
          <p
            v-if="isLoading"
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
          :ktx2="ktx2Header"
          :ktx1="ktx1HeaderValue"
        />
        <LevelIndexTable :levels="levelIndex" />
        <IndexPanel
          :data-format-descriptor="dataFormatDescriptorOffset"
          :key-value-data="keyValueDataOffset"
          :supercompression-global-data="supercompressionOffset"
        />
        <DfdPanel :descriptor="dataFormatDescriptor" />
        <KnownMetadataPanel :key-value-data="keyValueData" />
        <KeyValueTable :entries="otherKeyValues" />
        <BasisGlobalDataPanel :data="basisGlobalData" />
        <Ktx1TextPanel :text="ktx1TextValue" />
      </div>
    </ScrollArea>
  </section>
</template>
