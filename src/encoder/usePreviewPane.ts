import { computed, onUnmounted, ref, watch } from "vue";
import { call } from "../api/commands";
import { toAppError } from "../api/guards";
import type { EncodeResult } from "../bindings/EncodeResult";
import type { ImageSelection } from "../bindings/ImageSelection";
import type { InputRole } from "../bindings/InputRole";
import { toastAppError } from "../shared/toastAppError";
import { composeCubemapCross } from "../shared/cubemapCross";
import { selectedOutput, store, type OutputSlot } from "./store";

type PackAxis = "layer" | "mip" | "depth" | null;

function revoke(url: string | null): void {
  if (url !== null) {
    URL.revokeObjectURL(url);
  }
}

function revokeFaceList(urls: Array<string | null>): void {
  for (const url of urls) {
    revoke(url);
  }
}

function revokeUrlList(urls: ReadonlyArray<string | null>): void {
  for (const url of urls) {
    revoke(url);
  }
}

function replaceUrl(
  target: { value: string | null },
  next: string,
): void {
  const prev = target.value;
  target.value = next;
  if (prev !== null && prev !== next) {
    revoke(prev);
  }
}

function replaceFaceList(
  target: { value: Array<string | null> },
  next: Array<string | null>,
): void {
  const prev = target.value;
  target.value = next;
  revokeFaceList(prev);
}

function replaceUrlList(
  target: { value: Array<string | null> },
  next: Array<string | null>,
): void {
  const prev = target.value;
  target.value = next;
  revokeUrlList(prev);
}

function clampIndex(index: number, max: number): number {
  if (max < 0) {
    return 0;
  }
  return Math.min(Math.max(0, index), max);
}

function bytesToUrl(bytes: ArrayBuffer): string {
  return URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
}

function emptyFaces(): Array<string | null> {
  return [null, null, null, null, null, null];
}

function pickListUrl(
  urls: ReadonlyArray<string | null>,
  index: number,
): string | null {
  if (index < 0 || index >= urls.length) {
    return null;
  }
  return urls[index] ?? null;
}

export function usePreviewPane() {
  const previewIndex = ref(0);
  const encodedMip = ref(0);

  const sourceUrl = ref<string | null>(null);
  const encodedUrl = ref<string | null>(null);
  const contentWidth = ref<number | null>(null);
  const contentHeight = ref<number | null>(null);
  const sourceLoading = ref(false);
  const encodedLoading = ref(false);
  const faceSourceUrls = ref<Array<string | null>>(emptyFaces());
  const faceEncodedUrls = ref<Array<string | null>>(emptyFaces());
  /** Owned blob URLs for DepthSlices volume (source side). */
  const depthSourceUrls = ref<Array<string | null>>([]);
  /** Owned blob URLs for DepthSlices volume / wipe (encoded side). */
  const depthEncodedUrls = ref<Array<string | null>>([]);

  let sourceGen = 0;
  let encodedGen = 0;

  const role = computed((): InputRole => store.inputRole);

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
      return pickListUrl(depthSourceUrls.value, previewIndex.value);
    }
    return sourceUrl.value;
  });

  const activeEncodedUrl = computed((): string | null => {
    if (isDepthVolume.value) {
      return pickListUrl(depthEncodedUrls.value, previewIndex.value);
    }
    return encodedUrl.value;
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
    return sourceLoading.value || encodedLoading.value;
  });

  const volumeLoading = computed(
    (): boolean =>
      isDepthVolume.value &&
      depthSourceUrls.value.every((url) => url === null) &&
      sourceLoading.value,
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
    },
  );

  function wipeSourcePreview(): void {
    sourceGen += 1;
    revoke(sourceUrl.value);
    sourceUrl.value = null;
    revokeFaceList(faceSourceUrls.value);
    faceSourceUrls.value = emptyFaces();
    revokeUrlList(depthSourceUrls.value);
    depthSourceUrls.value = [];
    contentWidth.value = null;
    contentHeight.value = null;
    sourceLoading.value = false;
  }

  async function loadSourceCubemap(
    slot: OutputSlot,
    gen: number,
  ): Promise<void> {
    const urls: Array<string | null> = await Promise.all(
      Array.from({ length: 6 }, async (_, face) => {
        if (face >= slot.inputs.length) {
          return null;
        }
        try {
          return await loadSourcePreview(slot.inputs[face].path);
        } catch {
          return null;
        }
      }),
    );
    if (gen !== sourceGen) {
      revokeFaceList(urls);
      return;
    }
    replaceFaceList(faceSourceUrls, urls);
    const composed = await composeCubemapCross(urls);
    if (gen !== sourceGen) {
      revoke(composed.url);
      return;
    }
    replaceUrl(sourceUrl, composed.url);
    contentWidth.value = composed.width;
    contentHeight.value = composed.height;
  }

  async function loadSourceDepth(
    slot: OutputSlot,
    gen: number,
  ): Promise<void> {
    const urls: Array<string | null> = await Promise.all(
      slot.inputs.map(async (input) => {
        try {
          return await loadSourcePreview(input.path);
        } catch {
          return null;
        }
      }),
    );
    if (gen !== sourceGen) {
      revokeUrlList(urls);
      return;
    }
    replaceUrlList(depthSourceUrls, urls);
  }

  async function loadSourceFlat(slot: OutputSlot, gen: number): Promise<void> {
    const index =
      packAxis.value === null
        ? 0
        : clampIndex(previewIndex.value, packMax.value);
    if (index >= slot.inputs.length) {
      return;
    }
    const url = await loadSourcePreview(slot.inputs[index].path);
    if (gen !== sourceGen) {
      revoke(url);
      return;
    }
    replaceUrl(sourceUrl, url);
  }

  function depthStackReady(
    urls: ReadonlyArray<string | null>,
    expected: number,
  ): boolean {
    return urls.length === expected && urls.some((url) => url !== null);
  }

  function sourceNeedsLoading(currentRole: InputRole): boolean {
    if (currentRole === "depthSlices") {
      return !depthSourceUrls.value.some((url) => url !== null);
    }
    return sourceUrl.value === null;
  }

  async function loadSourceByRole(
    slot: OutputSlot,
    currentRole: InputRole,
    gen: number,
  ): Promise<void> {
    if (currentRole === "cubemapFaces") {
      await loadSourceCubemap(slot, gen);
      return;
    }
    if (currentRole === "depthSlices") {
      await loadSourceDepth(slot, gen);
      return;
    }
    await loadSourceFlat(slot, gen);
  }

  async function fetchSourcePreview(
    slot: OutputSlot,
    currentRole: InputRole,
    gen: number,
  ): Promise<void> {
    if (gen !== sourceGen) {
      return;
    }
    // Keep the previous frame while scrubbing; only flash loading on first paint.
    sourceLoading.value = sourceNeedsLoading(currentRole);
    try {
      await loadSourceByRole(slot, currentRole, gen);
    } catch {
      // Keep whatever was already on screen.
    } finally {
      if (gen === sourceGen) {
        sourceLoading.value = false;
      }
    }
  }

  function clearSourceSurfaces(): void {
    revoke(sourceUrl.value);
    sourceUrl.value = null;
    revokeFaceList(faceSourceUrls.value);
    faceSourceUrls.value = emptyFaces();
    revokeUrlList(depthSourceUrls.value);
    depthSourceUrls.value = [];
  }

  function skipDepthScrub(
    mode: "replace" | "scrub",
    currentRole: InputRole,
    urls: ReadonlyArray<string | null>,
    expected: number,
  ): boolean {
    return (
      mode === "scrub" &&
      currentRole === "depthSlices" &&
      depthStackReady(urls, expected)
    );
  }

  function selectedSlotOrWipe(): OutputSlot | null {
    const slot = selectedOutput.value;
    if (slot === null || slot.inputs.length === 0) {
      wipeSourcePreview();
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
    // Depth stack is fully cached; scrubbing only changes the highlight / wipe slice.
    if (
      skipDepthScrub(
        mode,
        currentRole,
        depthSourceUrls.value,
        slot.inputs.length,
      )
    ) {
      return;
    }

    // Output/role/input changes must not keep showing the previous slot.
    if (mode === "replace") {
      clearSourceSurfaces();
    }

    void fetchSourcePreview(slot, currentRole, (sourceGen += 1));
  }

  function wipeEncodedPreview(): void {
    encodedGen += 1;
    revoke(encodedUrl.value);
    encodedUrl.value = null;
    revokeFaceList(faceEncodedUrls.value);
    faceEncodedUrls.value = emptyFaces();
    revokeUrlList(depthEncodedUrls.value);
    depthEncodedUrls.value = [];
    encodedLoading.value = false;
  }

  async function loadEncodedCubemap(jobId: string, gen: number): Promise<void> {
    const urls: Array<string | null> = await Promise.all(
      Array.from({ length: 6 }, async (_, face) => {
        try {
          const bytes = await call("encoded_preview", {
            jobId,
            selection: selectionForEncode(face),
          });
          return bytesToUrl(bytes);
        } catch {
          return null;
        }
      }),
    );
    if (gen !== encodedGen) {
      revokeFaceList(urls);
      return;
    }
    replaceFaceList(faceEncodedUrls, urls);
    const composed = await composeCubemapCross(urls);
    if (gen !== encodedGen) {
      revoke(composed.url);
      return;
    }
    replaceUrl(encodedUrl, composed.url);
    contentWidth.value = composed.width;
    contentHeight.value = composed.height;
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

  async function loadEncodedDepth(jobId: string, gen: number): Promise<void> {
    const count = inputCount.value;
    const urls: Array<string | null> = await Promise.all(
      Array.from({ length: count }, async (_, depthSlice) => {
        try {
          const bytes = await call("encoded_preview", {
            jobId,
            selection: selectionForEncode(0, depthSlice),
          });
          return bytesToUrl(bytes);
        } catch {
          return null;
        }
      }),
    );
    if (gen !== encodedGen) {
      revokeUrlList(urls);
      return;
    }
    replaceUrlList(depthEncodedUrls, urls);
    applyEncodedDimensions();
  }

  async function loadEncodedFlat(jobId: string, gen: number): Promise<void> {
    const bytes = await call("encoded_preview", {
      jobId,
      selection: selectionForEncode(0),
    });
    const url = bytesToUrl(bytes);
    if (gen !== encodedGen) {
      revoke(url);
      return;
    }
    replaceUrl(encodedUrl, url);
    applyEncodedDimensions();
  }

  function encodedNeedsLoading(currentRole: InputRole): boolean {
    if (currentRole === "depthSlices") {
      return !depthEncodedUrls.value.some((url) => url !== null);
    }
    return encodedUrl.value === null;
  }

  async function loadEncodedByRole(
    jobId: string,
    currentRole: InputRole,
    gen: number,
  ): Promise<void> {
    if (currentRole === "cubemapFaces") {
      await loadEncodedCubemap(jobId, gen);
      return;
    }
    if (currentRole === "depthSlices") {
      await loadEncodedDepth(jobId, gen);
      return;
    }
    await loadEncodedFlat(jobId, gen);
  }

  async function fetchEncodedPreview(
    jobId: string,
    currentRole: InputRole,
    gen: number,
  ): Promise<void> {
    if (gen !== encodedGen) {
      return;
    }
    encodedLoading.value = encodedNeedsLoading(currentRole);
    try {
      await loadEncodedByRole(jobId, currentRole, gen);
    } catch (err: unknown) {
      if (gen !== encodedGen) {
        return;
      }
      toastAppError(toAppError(err));
    } finally {
      if (gen === encodedGen) {
        encodedLoading.value = false;
      }
    }
  }

  function clearEncodedSurfaces(): void {
    revoke(encodedUrl.value);
    encodedUrl.value = null;
    revokeFaceList(faceEncodedUrls.value);
    faceEncodedUrls.value = emptyFaces();
    revokeUrlList(depthEncodedUrls.value);
    depthEncodedUrls.value = [];
  }

  function refreshEncodedPreview(mode: "replace" | "scrub"): void {
    const jobId = selectedJobId.value;
    if (jobId === null) {
      wipeEncodedPreview();
      return;
    }

    const currentRole = role.value;
    // Depth extracts are cached per stack; scrubbing depth only rebinds the wipe.
    if (
      skipDepthScrub(
        mode,
        currentRole,
        depthEncodedUrls.value,
        inputCount.value,
      )
    ) {
      return;
    }

    if (mode === "replace") {
      clearEncodedSurfaces();
    }

    void fetchEncodedPreview(jobId, currentRole, (encodedGen += 1));
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
    sourceGen += 1;
    encodedGen += 1;
    revoke(sourceUrl.value);
    revoke(encodedUrl.value);
    revokeFaceList(faceSourceUrls.value);
    revokeFaceList(faceEncodedUrls.value);
    revokeUrlList(depthSourceUrls.value);
    revokeUrlList(depthEncodedUrls.value);
    sourceUrl.value = null;
    encodedUrl.value = null;
    depthSourceUrls.value = [];
    depthEncodedUrls.value = [];
  });

  return {
    previewIndex,
    encodedMip,
    contentWidth,
    contentHeight,
    packAxis,
    isCubemap,
    isDepthVolume,
    depthSourceUrls,
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
