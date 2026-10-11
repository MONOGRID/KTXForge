import { useElementSize, useEventListener } from "@vueuse/core";
import {
  computed,
  ref,
  type ComputedRef,
  type Ref,
} from "vue";

const ZOOM_STEP = 1.25;
const MIN_SCALE = 0.05;
const MAX_SCALE = 64;

export type ZoomPanApi = {
  frameRef: Ref<HTMLElement | null>;
  /** Frame content width in CSS pixels (for frame-space compare wipe). */
  frameW: Ref<number>;
  scale: ComputedRef<number>;
  panX: Ref<number>;
  panY: Ref<number>;
  dragging: Ref<boolean>;
  sizeLabel: ComputedRef<string | null>;
  transformStyle: ComputedRef<{ transform: string }>;
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
 */
export function useZoomPan(
  naturalWidth: Ref<number>,
  naturalHeight: Ref<number>,
  canInteract: () => boolean,
): ZoomPanApi {
  const frameRef = ref<HTMLElement | null>(null);
  const { width: frameW, height: frameH } = useElementSize(frameRef);

  /** User-controlled scale; `null` means fit-to-frame. */
  const userScale = ref<number | null>(null);
  const panX = ref(0);
  const panY = ref(0);
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

  const scale = computed((): number => userScale.value ?? fitScale.value);

  const sizeLabel = computed((): string | null => {
    if (naturalWidth.value <= 0) {
      return null;
    }
    const pct = Math.round(scale.value * 100);
    return `${String(naturalWidth.value)}×${String(naturalHeight.value)} · ${String(pct)}%`;
  });

  const transformStyle = computed((): { transform: string } => ({
    transform: `translate(${String(panX.value)}px, ${String(panY.value)}px) scale(${String(scale.value)})`,
  }));

  function clampScale(value: number): number {
    return Math.min(MAX_SCALE, Math.max(MIN_SCALE, value));
  }

  function resetPan(): void {
    panX.value = 0;
    panY.value = 0;
  }

  function zoomFit(): void {
    userScale.value = null;
    resetPan();
  }

  function zoomActual(): void {
    userScale.value = 1;
    resetPan();
  }

  function zoomAt(nextScale: number, originX: number, originY: number): void {
    const frame = frameRef.value;
    if (frame === null) {
      userScale.value = clampScale(nextScale);
      return;
    }

    const rect = frame.getBoundingClientRect();
    const cx = originX - rect.left - rect.width / 2;
    const cy = originY - rect.top - rect.height / 2;
    const prev = scale.value;
    const clamped = clampScale(nextScale);
    if (prev <= 0) {
      userScale.value = clamped;
      return;
    }
    const ratio = clamped / prev;
    panX.value = cx - (cx - panX.value) * ratio;
    panY.value = cy - (cy - panY.value) * ratio;
    userScale.value = clamped;
  }

  function zoomBy(factor: number): void {
    const frame = frameRef.value;
    if (frame === null) {
      userScale.value = clampScale(scale.value * factor);
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
