// fallow-ignore-file unused-file
// consumed by T5.5
// Minimal stub so T5.6 viewer components typecheck and build.
// T5.5 owns the full store (path loading, inspect, events) and replaces this.
import { reactive } from "vue";
import type { ImageSelection } from "../bindings/ImageSelection";
import type { InspectReport } from "../bindings/InspectReport";

const defaultSelection = {
  level: 0,
  face: 0,
  layer: 0,
  depthSlice: 0,
} satisfies ImageSelection;

export const store = reactive({
  path: null as string | null,
  report: null as InspectReport | null,
  selection: { ...defaultSelection },
});
