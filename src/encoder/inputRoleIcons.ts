import type { Component } from "vue";
import { Cuboid, Cylinder, Layers, Pyramid } from "@lucide/vue";
import type { PackInputRole } from "./inputRole";

/** Lucide icons for Inputs / drop-intent pack roles. */
export const INPUT_ROLE_ICONS = {
  arrayLayers: Layers,
  cubemapFaces: Cuboid,
  mipChain: Pyramid,
  depthSlices: Cylinder,
} as const satisfies Record<PackInputRole, Component>;
