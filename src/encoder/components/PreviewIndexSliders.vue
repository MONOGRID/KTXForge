<script setup lang="ts">
import RangeTickLabels from "../../shared/components/RangeTickLabels.vue";

defineProps<{
  showPackSlider: boolean;
  showGeneratedMipSlider: boolean;
  packSliderLabel: string;
  generatedMipLabel: string;
  packMax: number;
  generatedMipMax: number;
  previewIndex: number;
  encodedMip: number;
  packAxisIsMip: boolean;
}>();

const emit = defineEmits<{
  packInput: [event: Event];
  generatedMipInput: [event: Event];
}>();

function formatMipTick(level: number): string {
  return `L${String(level)}`;
}
</script>

<template>
  <div
    v-if="showPackSlider || showGeneratedMipSlider"
    class="flex shrink-0 flex-col gap-2"
  >
    <div
      v-if="showPackSlider"
      class="flex flex-col gap-1"
    >
      <label
        class="text-sm text-ink-muted"
        for="preview-pack"
      >{{ packSliderLabel }}</label>
      <input
        id="preview-pack"
        type="range"
        class="w-full accent-brand"
        min="0"
        :max="packMax"
        step="1"
        :value="previewIndex"
        @input="emit('packInput', $event)"
      >
      <RangeTickLabels
        :min="0"
        :max="packMax"
        :format="packAxisIsMip ? formatMipTick : undefined"
      />
    </div>

    <div
      v-if="showGeneratedMipSlider"
      class="flex flex-col gap-1"
    >
      <label
        class="text-sm text-ink-muted"
        for="preview-gen-mip"
      >{{ generatedMipLabel }}</label>
      <input
        id="preview-gen-mip"
        type="range"
        class="w-full accent-brand"
        min="0"
        :max="generatedMipMax"
        step="1"
        :value="encodedMip"
        @input="emit('generatedMipInput', $event)"
      >
      <RangeTickLabels
        :min="0"
        :max="generatedMipMax"
        :format="formatMipTick"
      />
    </div>
  </div>
</template>
