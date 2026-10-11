<script setup lang="ts">
import ImageViewport from "../../shared/components/ImageViewport.vue";
import { setDropZone } from "../dropTarget";
import { usePreviewPane } from "../usePreviewPane";
import PreviewIndexSliders from "./PreviewIndexSliders.vue";
import PreviewResultMeta from "./PreviewResultMeta.vue";

const {
  previewIndex,
  encodedMip,
  contentWidth,
  contentHeight,
  packAxis,
  isCubemap,
  result,
  packMax,
  showPackSlider,
  showGeneratedMipSlider,
  generatedMipMax,
  showTonemapHint,
  viewportSrc,
  compareSrc,
  loading,
  packSliderLabel,
  generatedMipLabel,
  onPackInput,
  onGeneratedMipInput,
} = usePreviewPane();
</script>

<template>
  <section
    class="panel flex h-full min-h-0 flex-col overflow-hidden"
    data-drop-zone="center"
    @pointerenter="setDropZone('center')"
  >
    <header class="shrink-0 border-b border-line px-3 py-2">
      <h2 class="m-0 text-sm font-semibold">
        Preview
      </h2>
    </header>

    <div class="flex min-h-0 flex-1 flex-col gap-2 p-3">
      <PreviewIndexSliders
        :show-pack-slider="showPackSlider"
        :show-generated-mip-slider="showGeneratedMipSlider"
        :pack-slider-label="packSliderLabel"
        :generated-mip-label="generatedMipLabel"
        :pack-max="packMax"
        :generated-mip-max="generatedMipMax"
        :preview-index="previewIndex"
        :encoded-mip="encodedMip"
        :pack-axis-is-mip="packAxis === 'mip'"
        @pack-input="onPackInput"
        @generated-mip-input="onGeneratedMipInput"
      />

      <ImageViewport
        class="min-h-0 flex-1"
        :src="viewportSrc"
        :compare-src="compareSrc"
        :content-width="contentWidth"
        :content-height="contentHeight"
        :alt="isCubemap ? 'Cubemap source' : 'Source'"
        :compare-alt="isCubemap ? 'Cubemap encoded' : 'Encoded preview'"
        :loading="loading"
        empty-label="Drop images here to add KTX outputs, or select one to preview"
        :hint="showTonemapHint"
        original-label="Original"
        result-label="Encoded"
      />

      <PreviewResultMeta :result="result" />
    </div>
  </section>
</template>
