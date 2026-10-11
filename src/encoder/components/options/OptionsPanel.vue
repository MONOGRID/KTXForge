<script setup lang="ts">
import AnimationSection from "../AnimationSection.vue";
import { store } from "../../store";
import AstcSection from "./AstcSection.vue";
import BasisLzSection from "./BasisLzSection.vue";
import ColorSection from "./ColorSection.vue";
import ContainerSection from "./ContainerSection.vue";
import EncoderFlagsSection from "./EncoderFlagsSection.vue";
import FilterSection from "./FilterSection.vue";
import FormatSection from "./FormatSection.vue";
import SupercompressionSection from "./SupercompressionSection.vue";
import UastcLdrSection from "./UastcLdrSection.vue";
</script>

<template>
  <div
    v-if="store.ready"
    class="flex flex-col gap-2"
  >
    <FormatSection />
    <ContainerSection />
    <FilterSection />
    <ColorSection />
    <BasisLzSection />
    <UastcLdrSection />
    <AstcSection />
    <EncoderFlagsSection />
    <SupercompressionSection />
    <AnimationSection />
  </div>
</template>
