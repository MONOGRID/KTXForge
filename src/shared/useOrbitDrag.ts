import { computed, ref, type ComputedRef, type Ref } from "vue";

export type OrbitDrag = {
  rotX: Ref<number>;
  rotZ: Ref<number>;
  dragging: Ref<boolean>;
  volumeTransform: ComputedRef<string>;
  onPointerDown: (event: PointerEvent) => void;
  onPointerMove: (event: PointerEvent) => void;
  onPointerUp: (event: PointerEvent) => void;
  resetOrbit: () => void;
};

/**
 * Click-drag orbit for a CSS 3D node (rotateX / rotateZ).
 * Double-click via `resetOrbit` restores the initial angles.
 */
export function useOrbitDrag(options: {
  initialRotX: number;
  initialRotZ: number;
  minRotX?: number;
  maxRotX?: number;
  /** Degrees per CSS pixel. */
  sensitivity?: number;
}): OrbitDrag {
  const minRotX = options.minRotX ?? 8;
  const maxRotX = options.maxRotX ?? 82;
  const sensitivity = options.sensitivity ?? 0.45;

  const rotX = ref(options.initialRotX);
  const rotZ = ref(options.initialRotZ);
  const dragging = ref(false);

  let lastX = 0;
  let lastY = 0;
  let captureEl: HTMLElement | null = null;

  function clampX(value: number): number {
    return Math.min(maxRotX, Math.max(minRotX, value));
  }

  function resetOrbit(): void {
    rotX.value = options.initialRotX;
    rotZ.value = options.initialRotZ;
  }

  function onPointerDown(event: PointerEvent): void {
    if (event.button !== 0) {
      return;
    }
    const target = event.currentTarget;
    if (!(target instanceof HTMLElement)) {
      return;
    }
    dragging.value = true;
    lastX = event.clientX;
    lastY = event.clientY;
    captureEl = target;
    target.setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  function onPointerMove(event: PointerEvent): void {
    if (!dragging.value) {
      return;
    }
    const dx = event.clientX - lastX;
    const dy = event.clientY - lastY;
    lastX = event.clientX;
    lastY = event.clientY;
    rotZ.value += dx * sensitivity;
    rotX.value = clampX(rotX.value - dy * sensitivity);
  }

  function onPointerUp(event: PointerEvent): void {
    if (!dragging.value) {
      return;
    }
    dragging.value = false;
    if (captureEl?.hasPointerCapture(event.pointerId) === true) {
      captureEl.releasePointerCapture(event.pointerId);
    }
    captureEl = null;
  }

  const volumeTransform = computed(
    (): string =>
      `rotateX(${String(rotX.value)}deg) rotateZ(${String(rotZ.value)}deg)`,
  );

  return {
    rotX,
    rotZ,
    dragging,
    volumeTransform,
    onPointerDown,
    onPointerMove,
    onPointerUp,
    resetOrbit,
  };
}
