/** Imperative zoom API registered by ImageViewer for the View menu. */
export type ViewerControls = {
  zoomIn: () => void;
  zoomOut: () => void;
  zoomActual: () => void;
  zoomFit: () => void;
};

let controls: ViewerControls | null = null;

export function registerViewerControls(next: ViewerControls): () => void {
  controls = next;
  return () => {
    if (controls === next) {
      controls = null;
    }
  };
}

export function getViewerControls(): ViewerControls | null {
  return controls;
}
