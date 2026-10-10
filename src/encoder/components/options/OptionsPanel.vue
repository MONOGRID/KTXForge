<script setup lang="ts">
import AnimationSection from "../AnimationSection.vue";
import AstcSection from "./AstcSection.vue";
import BasisLzSection from "./BasisLzSection.vue";
import ColorSection from "./ColorSection.vue";
import ContainerSection from "./ContainerSection.vue";
import EncoderFlagsSection from "./EncoderFlagsSection.vue";
import FilterSection from "./FilterSection.vue";
import FormatSection from "./FormatSection.vue";
import SupercompressionSection from "./SupercompressionSection.vue";
import UastcHdrSection from "./UastcHdrSection.vue";
import UastcLdrSection from "./UastcLdrSection.vue";
</script>

<template>
  <div class="flex flex-col gap-2">
    <ContainerSection />
    <FilterSection />
    <ColorSection />
    <FormatSection />
    <BasisLzSection />
    <UastcLdrSection />
    <UastcHdrSection />
    <AstcSection />
    <EncoderFlagsSection />
    <SupercompressionSection />
    <AnimationSection />
  </div>
</template>
