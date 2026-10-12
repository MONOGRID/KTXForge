import { computed, onUnmounted, ref, watch } from "vue";
import { call } from "../api/commands";
import type { EncodeResult } from "../bindings/EncodeResult";
import type { ImageSelection } from "../bindings/ImageSelection";
import type { InputRole } from "../bindings/InputRole";
import { composeCubemapCross } from "../shared/cubemapCross";
import { selectedOutput, selectedRole, type OutputSlot } from "./store";
import {
  bytesToUrl,
  createPreviewHalf,
  pickListUrl,
  replaceUrl,
  replaceUrlList,
  skipDepthScrub,
  type PreviewHalf,
} from "./previewHalf";

type PackAxis = "layer" | "mip" | "depth" | null;

function clampIndex(index: number, max: number): number {
  if (max < 0) {
    return 0;
  }
  return Math.min(Math.max(0, index), max);
}

type FaceFetcher = (face: number) => Promise<string | null>;
type SliceFetcher = (index: number) => Promise<string | null>;

async function loadCubemapHalf(
  half: PreviewHalf,
  gen: number,
  fetchFace: FaceFetcher,
  onComposed: (width: number, height: number) => void,
): Promise<void> {
  const urls: Array<string | null> = await Promise.all(
    Array.from({ length: 6 }, async (_, face) => {
      try {
        return await fetchFace(face);
      } catch {
        return null;
      }
    }),
  );
  if (half.revokeIfStale(gen, urls)) {
    return;
  }
  replaceUrlList(half.faceUrls, urls);
  const composed = await composeCubemapCross(urls);
  if (half.revokeUrlIfStale(gen, composed.url)) {
    return;
  }
  replaceUrl(half.url, composed.url);
  onComposed(composed.width, composed.height);
}

async function loadDepthHalf(
  half: PreviewHalf,
  gen: number,
  count: number,
  fetchSlice: SliceFetcher,
): Promise<void> {
  const urls: Array<string | null> = await Promise.all(
    Array.from({ length: count }, async (_, index) => {
      try {
        return await fetchSlice(index);
      } catch {
        return null;
      }
    }),
  );
  if (half.revokeIfStale(gen, urls)) {
    return;
  }
  replaceUrlList(half.depthUrls, urls);
}

async function loadFlatHalf(
  half: PreviewHalf,
  gen: number,
  fetchUrl: () => Promise<string>,
): Promise<void> {
  const url = await fetchUrl();
  if (half.revokeUrlIfStale(gen, url)) {
    return;
  }
  replaceUrl(half.url, url);
}

async function fetchHalfPreview(
  half: PreviewHalf,
  currentRole: InputRole,
  gen: number,
  load: () => Promise<void>,
): Promise<void> {
  if (!half.isCurrent(gen)) {
    return;
  }
  half.loading.value = half.needsLoading(currentRole);
  try {
    await load();
  } catch {
    // Keep whatever was already on screen.
  } finally {
    if (half.isCurrent(gen)) {
      half.loading.value = false;
    }
  }
}

export function usePreviewPane() {
  const previewIndex = ref(0);
  const encodedMip = ref(0);
  const contentWidth = ref<number | null>(null);
  const contentHeight = ref<number | null>(null);

  const source = createPreviewHalf();
  const encoded = createPreviewHalf();

  const role = computed((): InputRole => selectedRole.value);

  const isDepthVolume = computed((): boolean => role.value === "depthSlices");

  const packAxis = computed((): PackAxis => {
    if (role.value === "arrayLayers") {
      return "layer";
    }
    if (role.value === "mipChain") {
      return "mip";
    }
    if (role.value === "depthSlices") {
      return "depth";
    }
    return null;
  });

  const isCubemap = computed((): boolean => role.value === "cubemapFaces");

  const result = computed((): EncodeResult | null => {
    const slot = selectedOutput.value;
    if (slot === null || slot.job.status !== "done") {
      return null;
    }
    return slot.job.result;
  });

  const selectedJobId = computed((): string | null => {
    const slot = selectedOutput.value;
    if (
      slot === null ||
      slot.job.status !== "done" ||
      slot.job.jobId === null
    ) {
      return null;
    }
    return slot.job.jobId;
  });

  const inputCount = computed(
    (): number => selectedOutput.value?.inputs.length ?? 0,
  );

  const packMax = computed((): number => Math.max(0, inputCount.value - 1));

  const showPackSlider = computed(
    (): boolean => packAxis.value !== null && inputCount.value > 1,
  );

  const packingRoleBlocksMips = computed(
    (): boolean =>
      role.value === "mipChain" || role.value === "depthSlices",
  );

  const encodedLevelCount = computed(
    (): number => result.value?.levels ?? 0,
  );

  const showGeneratedMipSlider = computed(
    (): boolean =>
      selectedJobId.value !== null &&
      encodedLevelCount.value > 1 &&
      !packingRoleBlocksMips.value,
  );

  const generatedMipMax = computed((): number =>
    Math.max(0, (result.value?.levels ?? 1) - 1),
  );

  const showTonemapHint = computed((): string | null =>
    result.value?.preview.kind === "tonemappedFromExr"
      ? "Preview, not stored data"
      : null,
  );

  const activeSourceUrl = computed((): string | null => {
    if (isDepthVolume.value) {
      return pickListUrl(source.depthUrls.value, previewIndex.value);
    }
    return source.url.value;
  });

  const activeEncodedUrl = computed((): string | null => {
    if (isDepthVolume.value) {
      return pickListUrl(encoded.depthUrls.value, previewIndex.value);
    }
    return encoded.url.value;
  });

  const comparing = computed(
    (): boolean =>
      activeSourceUrl.value !== null && activeEncodedUrl.value !== null,
  );

  const viewportSrc = computed((): string | null => {
    if (comparing.value) {
      return activeSourceUrl.value;
    }
    return activeSourceUrl.value ?? activeEncodedUrl.value;
  });

  const compareSrc = computed((): string | null =>
    comparing.value ? activeEncodedUrl.value : null,
  );

  const loading = computed((): boolean => {
    if (viewportSrc.value !== null) {
      return false;
    }
    return source.loading.value || encoded.loading.value;
  });

  const volumeLoading = computed(
    (): boolean =>
      isDepthVolume.value &&
      source.depthUrls.value.every((url) => url === null) &&
      source.loading.value,
  );

  const packSliderLabel = computed((): string => {
    const i = previewIndex.value;
    if (packAxis.value === "layer") {
      return `Layer ${String(i)}`;
    }
    if (packAxis.value === "mip") {
      return `Mip L${String(i)}`;
    }
    if (packAxis.value === "depth") {
      return `Depth ${String(i)}`;
    }
    return "";
  });

  const generatedMipLabel = computed((): string => {
    const level = encodedMip.value;
    const baseW = result.value?.width ?? 0;
    const baseH = result.value?.height ?? 0;
    const w = Math.max(1, baseW >> level);
    const h = Math.max(1, baseH >> level);
    return `Mip L${String(level)} · ${String(w)}×${String(h)}`;
  });

  async function loadSourcePreview(path: string): Promise<string> {
    const bytes = await call("source_preview", { path });
    return bytesToUrl(bytes);
  }

  function currentEncodedLevel(): number {
    if (role.value === "mipChain") {
      return previewIndex.value;
    }
    if (showGeneratedMipSlider.value) {
      return encodedMip.value;
    }
    return 0;
  }

  function selectionForEncode(face = 0, depthSlice = 0): ImageSelection {
    const level = currentEncodedLevel();
    const layer = role.value === "arrayLayers" ? previewIndex.value : 0;
    const depth =
      role.value === "depthSlices" ? depthSlice : 0;
    return { level, face, layer, depthSlice: depth };
  }

  function onPackInput(event: Event): void {
    const el = event.target;
    if (!(el instanceof HTMLInputElement)) {
      return;
    }
    previewIndex.value = Number(el.value);
  }

  function onGeneratedMipInput(event: Event): void {
    const el = event.target;
    if (!(el instanceof HTMLInputElement)) {
      return;
    }
    encodedMip.value = Number(el.value);
  }

  function resetContentSize(): void {
    contentWidth.value = null;
    contentHeight.value = null;
  }

  function applyEncodedDimensions(): void {
    const res = result.value;
    if (res === null) {
      return;
    }
    const level = currentEncodedLevel();
    contentWidth.value = Math.max(1, res.width >> level);
    contentHeight.value = Math.max(1, res.height >> level);
  }

  watch(
    [role, inputCount, () => result.value?.levels ?? 1],
    () => {
      previewIndex.value = clampIndex(previewIndex.value, packMax.value);
      encodedMip.value = clampIndex(encodedMip.value, generatedMipMax.value);
    },
    { immediate: true },
  );

  watch(
    () => selectedOutput.value?.id ?? null,
    () => {
      previewIndex.value = 0;
      encodedMip.value = 0;
      resetContentSize();
    },
  );

  function sourceFacePath(slot: OutputSlot, face: number): string | null {
    if (face >= slot.inputs.length) {
      return null;
    }
    return slot.inputs[face].path;
  }

  function flatSourceIndex(slot: OutputSlot): number | null {
    const index =
      packAxis.value === null
        ? 0
        : clampIndex(previewIndex.value, packMax.value);
    if (index >= slot.inputs.length) {
      return null;
    }
    return index;
  }

  async function loadSourceByRole(
    slot: OutputSlot,
    currentRole: InputRole,
    gen: number,
  ): Promise<void> {
    if (currentRole === "cubemapFaces") {
      await loadCubemapHalf(
        source,
        gen,
        async (face) => {
          const path = sourceFacePath(slot, face);
          return path === null ? null : loadSourcePreview(path);
        },
        (width, height) => {
          contentWidth.value = width;
          contentHeight.value = height;
        },
      );
      return;
    }
    if (currentRole === "depthSlices") {
      await loadDepthHalf(source, gen, slot.inputs.length, async (index) =>
        loadSourcePreview(slot.inputs[index].path),
      );
      return;
    }
    const index = flatSourceIndex(slot);
    if (index === null) {
      return;
    }
    await loadFlatHalf(source, gen, () =>
      loadSourcePreview(slot.inputs[index].path),
    );
  }

  function selectedSlotOrWipe(): OutputSlot | null {
    const slot = selectedOutput.value;
    if (slot === null || slot.inputs.length === 0) {
      source.wipe();
      resetContentSize();
      return null;
    }
    return slot;
  }

  function refreshSourcePreview(mode: "replace" | "scrub"): void {
    const slot = selectedSlotOrWipe();
    if (slot === null) {
      return;
    }

    const currentRole = role.value;
    if (
      skipDepthScrub(
        mode,
        currentRole,
        source.depthUrls.value,
        slot.inputs.length,
      )
    ) {
      return;
    }

    const gen =
      mode === "replace" ? source.beginReplace() : source.generation();
    const slotId = slot.id;
    const gateKey = `src:${slotId}:${currentRole}:${String(previewIndex.value)}`;
    source.gate.schedule(gateKey, async () => {
      const live = selectedOutput.value;
      if (
        live === null ||
        live.id !== slotId ||
        !source.isCurrent(gen)
      ) {
        return;
      }
      await fetchHalfPreview(source, role.value, gen, () =>
        loadSourceByRole(live, role.value, gen),
      );
    });
  }

  async function loadEncodedByRole(
    jobId: string,
    currentRole: InputRole,
    gen: number,
  ): Promise<void> {
    if (currentRole === "cubemapFaces") {
      await loadCubemapHalf(
        encoded,
        gen,
        async (face) => {
          const bytes = await call("encoded_preview", {
            jobId,
            selection: selectionForEncode(face),
          });
          return bytesToUrl(bytes);
        },
        (width, height) => {
          contentWidth.value = width;
          contentHeight.value = height;
        },
      );
      return;
    }
    if (currentRole === "depthSlices") {
      await loadDepthHalf(encoded, gen, inputCount.value, async (depthSlice) => {
        const bytes = await call("encoded_preview", {
          jobId,
          selection: selectionForEncode(0, depthSlice),
        });
        return bytesToUrl(bytes);
      });
      applyEncodedDimensions();
      return;
    }
    await loadFlatHalf(encoded, gen, async () => {
      const bytes = await call("encoded_preview", {
        jobId,
        selection: selectionForEncode(0),
      });
      return bytesToUrl(bytes);
    });
    applyEncodedDimensions();
  }

  function refreshEncodedPreview(mode: "replace" | "scrub"): void {
    const jobId = selectedJobId.value;
    if (jobId === null) {
      encoded.wipe();
      return;
    }

    const currentRole = role.value;
    if (
      skipDepthScrub(
        mode,
        currentRole,
        encoded.depthUrls.value,
        inputCount.value,
      )
    ) {
      return;
    }

    const gen =
      mode === "replace" ? encoded.beginReplace() : encoded.generation();
    const gateKey = [
      "enc",
      jobId,
      currentRole,
      String(previewIndex.value),
      String(encodedMip.value),
    ].join(":");
    encoded.gate.schedule(gateKey, async () => {
      if (selectedJobId.value !== jobId || !encoded.isCurrent(gen)) {
        return;
      }
      await fetchHalfPreview(encoded, role.value, gen, () =>
        loadEncodedByRole(jobId, role.value, gen),
      );
    });
  }

  watch(
    [
      () => selectedOutput.value?.id ?? null,
      role,
      () => selectedOutput.value?.inputs.map((s) => s.path).join("\0") ?? "",
    ],
    () => {
      refreshSourcePreview("replace");
    },
    { immediate: true },
  );

  watch(previewIndex, () => {
    refreshSourcePreview("scrub");
  });

  watch(
    [selectedJobId, role, () => result.value?.levels ?? 0],
    () => {
      refreshEncodedPreview("replace");
    },
    { immediate: true },
  );

  watch([previewIndex, encodedMip], () => {
    refreshEncodedPreview("scrub");
  });

  onUnmounted(() => {
    source.wipe();
    encoded.wipe();
  });

  return {
    previewIndex,
    encodedMip,
    contentWidth,
    contentHeight,
    packAxis,
    isCubemap,
    isDepthVolume,
    depthSourceUrls: source.depthUrls,
    result,
    packMax,
    showPackSlider,
    showGeneratedMipSlider,
    generatedMipMax,
    showTonemapHint,
    viewportSrc,
    compareSrc,
    loading,
    volumeLoading,
    packSliderLabel,
    generatedMipLabel,
    onPackInput,
    onGeneratedMipInput,
  };
}
