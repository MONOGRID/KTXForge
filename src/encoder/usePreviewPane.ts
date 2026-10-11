import { computed, onUnmounted, ref, watch } from "vue";
import { call } from "../api/commands";
import { toAppError } from "../api/guards";
import type { EncodeResult } from "../bindings/EncodeResult";
import type { ImageSelection } from "../bindings/ImageSelection";
import type { InputRole } from "../bindings/InputRole";
import { toastAppError } from "../shared/toastAppError";
import { composeCubemapCross } from "./cubemapCross";
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

  let sourceGen = 0;
  let encodedGen = 0;

  const role = computed((): InputRole => store.inputRole);

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

  const showGeneratedMipSlider = computed((): boolean => {
    if (role.value === "mipChain") {
      return false;
    }
    const levels = result.value?.levels ?? 0;
    return selectedJobId.value !== null && levels > 1;
  });

  const generatedMipMax = computed((): number =>
    Math.max(0, (result.value?.levels ?? 1) - 1),
  );

  const showTonemapHint = computed((): string | null =>
    result.value?.preview.kind === "tonemappedFromExr"
      ? "Preview, not stored data"
      : null,
  );

  const comparing = computed(
    (): boolean => sourceUrl.value !== null && encodedUrl.value !== null,
  );

  const viewportSrc = computed((): string | null => {
    if (comparing.value) {
      return sourceUrl.value;
    }
    return sourceUrl.value ?? encodedUrl.value;
  });

  const compareSrc = computed((): string | null =>
    comparing.value ? encodedUrl.value : null,
  );

  const loading = computed(
    (): boolean =>
      viewportSrc.value === null &&
      (sourceLoading.value || encodedLoading.value),
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

  function selectionForEncode(face = 0): ImageSelection {
    const level = currentEncodedLevel();
    const layer = role.value === "arrayLayers" ? previewIndex.value : 0;
    const depthSlice = role.value === "depthSlices" ? previewIndex.value : 0;
    return { level, face, layer, depthSlice };
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

  async function fetchSourcePreview(
    slot: OutputSlot,
    currentRole: InputRole,
    gen: number,
  ): Promise<void> {
    if (gen !== sourceGen) {
      return;
    }
    // Keep the previous frame while scrubbing; only flash loading on first paint.
    sourceLoading.value = sourceUrl.value === null;
    try {
      if (currentRole === "cubemapFaces") {
        await loadSourceCubemap(slot, gen);
        return;
      }
      await loadSourceFlat(slot, gen);
    } catch {
      // Keep whatever was already on screen.
    } finally {
      if (gen === sourceGen) {
        sourceLoading.value = false;
      }
    }
  }

  function refreshSourcePreview(mode: "replace" | "scrub"): void {
    const slot = selectedOutput.value;
    const currentRole = role.value;

    if (slot === null || slot.inputs.length === 0) {
      wipeSourcePreview();
      return;
    }

    // Output/role/input changes must not keep showing the previous slot.
    if (mode === "replace") {
      revoke(sourceUrl.value);
      sourceUrl.value = null;
      revokeFaceList(faceSourceUrls.value);
      faceSourceUrls.value = emptyFaces();
    }

    const gen = (sourceGen += 1);
    void fetchSourcePreview(slot, currentRole, gen);
  }

  function wipeEncodedPreview(): void {
    encodedGen += 1;
    revoke(encodedUrl.value);
    encodedUrl.value = null;
    revokeFaceList(faceEncodedUrls.value);
    faceEncodedUrls.value = emptyFaces();
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

  async function fetchEncodedPreview(
    jobId: string,
    currentRole: InputRole,
    gen: number,
  ): Promise<void> {
    if (gen !== encodedGen) {
      return;
    }
    encodedLoading.value = encodedUrl.value === null;
    try {
      if (currentRole === "cubemapFaces") {
        await loadEncodedCubemap(jobId, gen);
        return;
      }
      await loadEncodedFlat(jobId, gen);
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

  function refreshEncodedPreview(mode: "replace" | "scrub"): void {
    const jobId = selectedJobId.value;
    const currentRole = role.value;

    if (jobId === null) {
      wipeEncodedPreview();
      return;
    }

    if (mode === "replace") {
      revoke(encodedUrl.value);
      encodedUrl.value = null;
      revokeFaceList(faceEncodedUrls.value);
      faceEncodedUrls.value = emptyFaces();
    }

    const gen = (encodedGen += 1);
    void fetchEncodedPreview(jobId, currentRole, gen);
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
    sourceUrl.value = null;
    encodedUrl.value = null;
  });

  return {
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
  };
}
