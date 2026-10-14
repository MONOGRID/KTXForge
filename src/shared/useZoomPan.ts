import {
  useDevicePixelRatio,
  useElementBounding,
  useElementSize,
  useEventListener,
} from "@vueuse/core";
import {
  computed,
  ref,
  watch,
  type ComputedRef,
  type Ref,
} from "vue";

const ZOOM_STEP = 1.25;
const MIN_SCALE = 0.05;
const MAX_SCALE = 64;

/** Sticky View sizing rule until the user picks another or zooms freely. */
export type ZoomMode = "fit" | "actual" | "manual";

export type ZoomPanTransformStyle = {
  transform: string;
  transformOrigin: "0 0";
};

/** Viewport position of the transform origin (the frame's padding box) and the device pixel ratio. */
export type PixelGrid = { x: number; y: number; dpr: number };

/** Nearest offset from `origin` that lands on a whole device pixel. */
function snapToDevicePixel(offset: number, origin: number, dpr: number): number {
  return Math.round((origin + offset) * dpr) / dpr - origin;
}

/**
 * Frame-space place: scale from the content top-left, then translate so pan=0
 * centers the scaled bitmap. Do **not** use left/top 50% + translate(-50%) —
 * an absolutely positioned shrink-to-fit box with `left: 50%` gets a huge used
 * width (available space to the CB’s right edge), so %-translate + scale turns
 * a 1×1 mip into a wide rectangle stuck to a corner.
 *
 * The translate is snapped so the image's top-left sits on a device pixel:
 * centring (and a frame at a fractional layout position) otherwise leaves a
 * half-pixel offset, and Actual size would resample every texel.
 */
export function zoomPanTransformStyle(
  panX: number,
  panY: number,
  scale: number,
  naturalWidth: number,
  naturalHeight: number,
  frameW: number,
  frameH: number,
  grid: PixelGrid = { x: 0, y: 0, dpr: 1 },
): ZoomPanTransformStyle {
  const tx = snapToDevicePixel((frameW - naturalWidth * scale) / 2 + panX, grid.x, grid.dpr);
  const ty = snapToDevicePixel((frameH - naturalHeight * scale) / 2 + panY, grid.y, grid.dpr);
  return {
    transform: `translate(${String(tx)}px, ${String(ty)}px) scale(${String(scale)})`,
    transformOrigin: "0 0",
  };
}

/**
 * Window-scoped sticky zoom. Survives ImageViewport remounts (e.g. depth
 * volume vs flat) and stays put across outputs / layers / mips.
 */
const sharedZoomMode = ref<ZoomMode>("fit");
const sharedManualScale = ref(1);
const sharedPanX = ref(0);
const sharedPanY = ref(0);

export type ZoomPanApi = {
  frameRef: Ref<HTMLElement | null>;
  /** Frame content width in CSS pixels (for frame-space compare wipe). */
  frameW: Ref<number>;
  scale: ComputedRef<number>;
  zoomMode: Ref<ZoomMode>;
  panX: Ref<number>;
  panY: Ref<number>;
  dragging: Ref<boolean>;
  sizeLabel: ComputedRef<string | null>;
  transformStyle: ComputedRef<ZoomPanTransformStyle>;
  zoomIn: () => void;
  zoomOut: () => void;
  zoomActual: () => void;
  zoomFit: () => void;
  onPointerDown: (event: PointerEvent) => void;
  onPointerMove: (event: PointerEvent) => void;
  onPointerUp: (event: PointerEvent) => void;
};

/**
 * Fit / 1:1 / stepped zoom with cursor-anchored Ctrl/Cmd+wheel and drag pan.
 * `canInteract` gates pan (and should be false while a compare handle is dragged).
 *
 * Fit and Actual are sticky modes: content size changes (mip/layer/output) keep
 * the mode. Wheel / Zoom In/Out switch to manual and keep that scale.
 */
export function useZoomPan(
  naturalWidth: Ref<number>,
  naturalHeight: Ref<number>,
  canInteract: () => boolean,
): ZoomPanApi {
  const frameRef = ref<HTMLElement | null>(null);
  // content-box matches the absolute inset-0 child’s containing block (no padding).
  const { width: frameW, height: frameH } = useElementSize(frameRef);
  const { left: frameLeft, top: frameTop } = useElementBounding(frameRef);
  const { pixelRatio } = useDevicePixelRatio();

  const zoomMode = sharedZoomMode;
  const manualScale = sharedManualScale;
  const panX = sharedPanX;
  const panY = sharedPanY;
  const dragging = ref(false);
  const dragOrigin = ref<{
    x: number;
    y: number;
    panX: number;
    panY: number;
  } | null>(null);

  const fitScale = computed((): number => {
    if (
      naturalWidth.value <= 0 ||
      naturalHeight.value <= 0 ||
      frameW.value <= 0 ||
      frameH.value <= 0
    ) {
      return 1;
    }
    const pad = 8;
    const availW = Math.max(1, frameW.value - pad);
    const availH = Math.max(1, frameH.value - pad);
    return Math.min(availW / naturalWidth.value, availH / naturalHeight.value);
  });

  const scale = computed((): number => {
    if (zoomMode.value === "fit") {
      return fitScale.value;
    }
    if (zoomMode.value === "actual") {
      return 1;
    }
    return clampScale(manualScale.value);
  });

  const sizeLabel = computed((): string | null => {
    if (naturalWidth.value <= 0) {
      return null;
    }
    const pct = Math.round(scale.value * 100);
    return `${String(naturalWidth.value)}×${String(naturalHeight.value)} · ${String(pct)}%`;
  });

  const transformStyle = computed((): ZoomPanTransformStyle => {
    // Layers are positioned in the padding box, inside the frame's border.
    const frame = frameRef.value;
    return zoomPanTransformStyle(
      panX.value,
      panY.value,
      scale.value,
      naturalWidth.value,
      naturalHeight.value,
      frameW.value,
      frameH.value,
      {
        x: frameLeft.value + (frame?.clientLeft ?? 0),
        y: frameTop.value + (frame?.clientTop ?? 0),
        dpr: pixelRatio.value,
      },
    );
  });

  function clampScale(value: number): number {
    return Math.min(MAX_SCALE, Math.max(MIN_SCALE, value));
  }

  function resetPan(): void {
    panX.value = 0;
    panY.value = 0;
  }

  function zoomFit(): void {
    zoomMode.value = "fit";
    resetPan();
  }

  function zoomActual(): void {
    zoomMode.value = "actual";
    resetPan();
  }

  // Fit/Actual mean centered for *this* content. Clear stale pan on size jumps
  // (e.g. 5×4 → 2×2) so letterboxing changes don’t look like a ratio bug.
  watch(
    [naturalWidth, naturalHeight],
    () => {
      if (zoomMode.value === "fit" || zoomMode.value === "actual") {
        resetPan();
      }
    },
    { flush: "sync" },
  );

  function zoomAt(nextScale: number, originX: number, originY: number): void {
    const frame = frameRef.value;
    if (frame === null) {
      zoomMode.value = "manual";
      manualScale.value = clampScale(nextScale);
      return;
    }

    const rect = frame.getBoundingClientRect();
    const cx = originX - rect.left - rect.width / 2;
    const cy = originY - rect.top - rect.height / 2;
    const prev = scale.value;
    const clamped = clampScale(nextScale);
    if (prev <= 0) {
      zoomMode.value = "manual";
      manualScale.value = clamped;
      return;
    }
    const ratio = clamped / prev;
    panX.value = cx - (cx - panX.value) * ratio;
    panY.value = cy - (cy - panY.value) * ratio;
    zoomMode.value = "manual";
    manualScale.value = clamped;
  }

  function zoomBy(factor: number): void {
    const frame = frameRef.value;
    if (frame === null) {
      zoomMode.value = "manual";
      manualScale.value = clampScale(scale.value * factor);
      return;
    }
    const rect = frame.getBoundingClientRect();
    zoomAt(
      scale.value * factor,
      rect.left + rect.width / 2,
      rect.top + rect.height / 2,
    );
  }

  function zoomIn(): void {
    zoomBy(ZOOM_STEP);
  }

  function zoomOut(): void {
    zoomBy(1 / ZOOM_STEP);
  }

  function onWheel(event: WheelEvent): void {
    if (!event.ctrlKey && !event.metaKey) {
      return;
    }
    if (!canInteract()) {
      return;
    }
    event.preventDefault();
    const factor = event.deltaY < 0 ? ZOOM_STEP : 1 / ZOOM_STEP;
    zoomAt(scale.value * factor, event.clientX, event.clientY);
  }

  function onPointerDown(event: PointerEvent): void {
    if (event.button !== 0 || !canInteract()) {
      return;
    }
    dragging.value = true;
    dragOrigin.value = {
      x: event.clientX,
      y: event.clientY,
      panX: panX.value,
      panY: panY.value,
    };
    frameRef.value?.setPointerCapture(event.pointerId);
  }

  function onPointerMove(event: PointerEvent): void {
    const origin = dragOrigin.value;
    if (!dragging.value || origin === null) {
      return;
    }
    panX.value = origin.panX + (event.clientX - origin.x);
    panY.value = origin.panY + (event.clientY - origin.y);
  }

  function onPointerUp(event: PointerEvent): void {
    if (!dragging.value) {
      return;
    }
    dragging.value = false;
    dragOrigin.value = null;
    if (frameRef.value?.hasPointerCapture(event.pointerId) === true) {
      frameRef.value.releasePointerCapture(event.pointerId);
    }
  }

  useEventListener(frameRef, "wheel", onWheel, { passive: false });

  return {
    frameRef,
    frameW,
    scale,
    zoomMode,
    panX,
    panY,
    dragging,
    sizeLabel,
    transformStyle,
    zoomIn,
    zoomOut,
    zoomActual,
    zoomFit,
    onPointerDown,
    onPointerMove,
    onPointerUp,
  };
}
