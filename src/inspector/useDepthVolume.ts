import { computed, ref, watch, type Ref } from "vue";
import type { InspectReport } from "../bindings/InspectReport";
import type { ImageSelection } from "../bindings/ImageSelection";
import {
  ensureInspectorImageUrl,
  peekInspectorImageUrl,
} from "./imageCache";

function baseDepthFromReport(current: InspectReport | null): number {
  if (current === null) {
    return 1;
  }
  if (current.kind.kind === "ktx2") {
    return Math.max(1, current.kind.info.header.pixelDepth);
  }
  return Math.max(1, current.kind.header.pixelDepth);
}

/**
 * Loads every depth plane for the current level/face/layer into object URLs
 * for the CSS3D volume view.
 */
export function useDepthVolume(args: {
  path: Ref<string | null>;
  report: Ref<InspectReport | null>;
  selection: Ref<ImageSelection>;
  previewUnavailable: Ref<string | null>;
  preparePath: (filePath: string) => void;
}): {
  depthSliceUrls: Ref<Array<string | null>>;
  volumeLoading: Ref<boolean>;
  showDepthVolume: Ref<boolean>;
  activeDepthIndex: Ref<number>;
  depthCountAtLevel: Ref<number>;
  resetVolume: () => void;
  invalidateVolume: () => void;
} {
  const depthSliceUrls = ref<Array<string | null>>([]);
  const volumeLoading = ref(false);
  let volumeGen = 0;

  const baseDepthCount = computed((): number =>
    baseDepthFromReport(args.report.value),
  );

  const depthCountAtLevel = computed((): number =>
    Math.max(1, baseDepthCount.value >> args.selection.value.level),
  );

  const showDepthVolume = computed(
    (): boolean =>
      args.previewUnavailable.value === null && depthCountAtLevel.value > 1,
  );

  const activeDepthIndex = computed((): number =>
    Math.min(
      args.selection.value.depthSlice,
      Math.max(0, depthCountAtLevel.value - 1),
    ),
  );

  function resetVolume(): void {
    depthSliceUrls.value = [];
    volumeLoading.value = false;
  }

  function invalidateVolume(): void {
    volumeGen += 1;
    resetVolume();
  }

  function setDepthUrl(index: number, url: string): void {
    const next = depthSliceUrls.value.slice();
    if (index < 0 || index >= next.length) {
      return;
    }
    next[index] = url;
    depthSliceUrls.value = next;
  }

  function volumeLoadTarget(): string | null {
    const filePath = args.path.value;
    if (filePath === null || filePath === "" || !showDepthVolume.value) {
      return null;
    }
    return filePath;
  }

  async function loadOneSlice(
    filePath: string,
    baseSel: Omit<ImageSelection, "depthSlice">,
    depthSlice: number,
    gen: number,
  ): Promise<void> {
    try {
      const url = await ensureInspectorImageUrl(filePath, {
        ...baseSel,
        depthSlice,
      });
      if (gen === volumeGen) {
        setDepthUrl(depthSlice, url);
      }
    } catch {
      // Leave the plane empty; active-slice ImageViewport surfaces real errors.
    }
  }

  function seedSliceUrls(
    filePath: string,
    baseSel: Omit<ImageSelection, "depthSlice">,
    count: number,
  ): Array<string | null> {
    return Array.from({ length: count }, (_, depthSlice) =>
      peekInspectorImageUrl(filePath, { ...baseSel, depthSlice }),
    );
  }

  function missingDepthIndexes(
    seeded: ReadonlyArray<string | null>,
  ): number[] {
    return seeded.flatMap((url, depthSlice) =>
      url === null ? [depthSlice] : [],
    );
  }

  async function loadDepthVolume(): Promise<void> {
    const gen = (volumeGen += 1);
    const filePath = volumeLoadTarget();
    if (filePath === null) {
      resetVolume();
      return;
    }

    args.preparePath(filePath);

    const baseSel = {
      level: args.selection.value.level,
      face: args.selection.value.face,
      layer: args.selection.value.layer,
    };
    const seeded = seedSliceUrls(filePath, baseSel, depthCountAtLevel.value);
    depthSliceUrls.value = seeded;
    volumeLoading.value = seeded.every((url) => url === null);

    await Promise.all(
      missingDepthIndexes(seeded).map((depthSlice) =>
        loadOneSlice(filePath, baseSel, depthSlice, gen),
      ),
    );

    if (gen === volumeGen) {
      volumeLoading.value = false;
    }
  }

  watch(
    () =>
      [
        args.path.value,
        args.report.value,
        args.selection.value.level,
        args.selection.value.face,
        args.selection.value.layer,
        depthCountAtLevel.value,
        args.previewUnavailable.value,
      ] as const,
    () => {
      void loadDepthVolume();
    },
    { immediate: true },
  );

  return {
    depthSliceUrls,
    volumeLoading,
    showDepthVolume,
    activeDepthIndex,
    depthCountAtLevel,
    resetVolume,
    invalidateVolume,
  };
}
