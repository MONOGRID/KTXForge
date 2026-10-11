import { computed, onUnmounted, ref, watch, type Ref } from "vue";
import type { ImageSelection } from "../bindings/ImageSelection";
import type { InspectReport } from "../bindings/InspectReport";
import { composeCubemapCross } from "../shared/cubemapCross";
import { ensureInspectorImageUrl, peekInspectorImageUrl } from "./imageCache";

function faceCountFromReport(current: InspectReport | null): number {
  if (current === null) {
    return 1;
  }
  if (current.kind.kind === "ktx2") {
    return Math.max(1, current.kind.info.header.faceCount);
  }
  return Math.max(1, current.kind.header.numberOfFaces);
}

function revoke(url: string | null): void {
  if (url !== null) {
    URL.revokeObjectURL(url);
  }
}

async function loadFaceUrl(
  filePath: string,
  baseSel: Omit<ImageSelection, "face">,
  face: number,
): Promise<string | null> {
  const sel = { ...baseSel, face };
  const cached = peekInspectorImageUrl(filePath, sel);
  if (cached !== null) {
    return cached;
  }
  try {
    return await ensureInspectorImageUrl(filePath, sel);
  } catch {
    return null;
  }
}

function seedFaceUrls(
  filePath: string,
  baseSel: Omit<ImageSelection, "face">,
  count: number,
): Array<string | null> {
  return Array.from({ length: count }, (_, face) =>
    peekInspectorImageUrl(filePath, { ...baseSel, face }),
  );
}

async function resolveFaceUrls(
  filePath: string,
  baseSel: Omit<ImageSelection, "face">,
  seeded: ReadonlyArray<string | null>,
): Promise<Array<string | null>> {
  return Promise.all(
    seeded.map(async (hit, face) => {
      if (hit !== null) {
        return hit;
      }
      return loadFaceUrl(filePath, baseSel, face);
    }),
  );
}

/**
 * Composites every face of a cubemap into the shared classic cross layout
 * (same canvas path as encoder Preview).
 */
export function useCubemapCross(args: {
  path: Ref<string | null>;
  report: Ref<InspectReport | null>;
  selection: Ref<ImageSelection>;
  previewUnavailable: Ref<string | null>;
  /** Depth volume wins when both could apply. */
  showDepthVolume: Ref<boolean>;
  preparePath: (filePath: string) => void;
}): {
  showCubemapCross: Ref<boolean>;
  crossUrl: Ref<string | null>;
  crossWidth: Ref<number | null>;
  crossHeight: Ref<number | null>;
  crossLoading: Ref<boolean>;
  invalidateCross: () => void;
} {
  const crossUrl = ref<string | null>(null);
  const crossWidth = ref<number | null>(null);
  const crossHeight = ref<number | null>(null);
  const crossLoading = ref(false);
  let crossGen = 0;

  const faceCount = computed((): number =>
    faceCountFromReport(args.report.value),
  );

  const showCubemapCross = computed(
    (): boolean =>
      args.previewUnavailable.value === null &&
      !args.showDepthVolume.value &&
      faceCount.value > 1,
  );

  function clearCross(): void {
    revoke(crossUrl.value);
    crossUrl.value = null;
    crossWidth.value = null;
    crossHeight.value = null;
    crossLoading.value = false;
  }

  function invalidateCross(): void {
    crossGen += 1;
    clearCross();
  }

  function crossLoadTarget(): string | null {
    const filePath = args.path.value;
    if (filePath === null || filePath === "" || !showCubemapCross.value) {
      return null;
    }
    return filePath;
  }

  function applyComposed(composed: {
    url: string;
    width: number;
    height: number;
  }): void {
    revoke(crossUrl.value);
    crossUrl.value = composed.url;
    crossWidth.value = composed.width;
    crossHeight.value = composed.height;
    crossLoading.value = false;
  }

  function isStaleGeneration(
    gen: number,
    composed: { url: string; width: number; height: number } | null,
  ): boolean {
    if (gen === crossGen) {
      return false;
    }
    if (composed !== null) {
      revoke(composed.url);
    }
    return true;
  }

  function finishCompose(
    gen: number,
    composed: { url: string; width: number; height: number } | null,
  ): void {
    if (isStaleGeneration(gen, composed)) {
      return;
    }
    if (composed === null) {
      clearCross();
      return;
    }
    applyComposed(composed);
  }

  async function composeAndApply(
    faceUrls: ReadonlyArray<string | null>,
    gen: number,
  ): Promise<void> {
    try {
      finishCompose(gen, await composeCubemapCross(faceUrls));
    } catch {
      finishCompose(gen, null);
    }
  }

  async function loadCross(): Promise<void> {
    const gen = (crossGen += 1);
    const filePath = crossLoadTarget();
    if (filePath === null) {
      clearCross();
      return;
    }

    args.preparePath(filePath);

    const baseSel = {
      level: args.selection.value.level,
      layer: args.selection.value.layer,
      depthSlice: args.selection.value.depthSlice,
    };
    const seeded = seedFaceUrls(filePath, baseSel, faceCount.value);
    crossLoading.value = seeded.every((url) => url === null);

    const faceUrls = await resolveFaceUrls(filePath, baseSel, seeded);
    if (gen !== crossGen) {
      return;
    }

    await composeAndApply(faceUrls, gen);
  }

  watch(
    () =>
      [
        args.path.value,
        args.report.value,
        args.selection.value.level,
        args.selection.value.layer,
        args.selection.value.depthSlice,
        faceCount.value,
        args.showDepthVolume.value,
        args.previewUnavailable.value,
      ] as const,
    () => {
      void loadCross();
    },
    { immediate: true },
  );

  onUnmounted(() => {
    invalidateCross();
  });

  return {
    showCubemapCross,
    crossUrl,
    crossWidth,
    crossHeight,
    crossLoading,
    invalidateCross,
  };
}
