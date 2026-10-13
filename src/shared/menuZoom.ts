import type { MenuAction } from "../bindings/MenuAction";
import { isMenuZoomWindowFocused } from "./menuZoomFocus";
import { getViewerControls, type ViewerControls } from "./viewerControls";

const ZOOM_BY_ACTION = {
  "zoom-in": (viewer: ViewerControls) => {
    viewer.zoomIn();
  },
  "zoom-out": (viewer: ViewerControls) => {
    viewer.zoomOut();
  },
  "zoom-actual": (viewer: ViewerControls) => {
    viewer.zoomActual();
  },
  "zoom-fit": (viewer: ViewerControls) => {
    viewer.zoomFit();
  },
} as const satisfies Record<string, (viewer: ViewerControls) => void>;

type ZoomMenuAction = keyof typeof ZOOM_BY_ACTION;

function isZoomMenuAction(action: MenuAction): action is ZoomMenuAction {
  return action in ZOOM_BY_ACTION;
}

/**
 * Handle View-menu zoom actions when this window is the menu-zoom target
 * (last focused among app windows; see `menuZoomFocus`).
 * Returns true when `action` was a zoom command (handled or ignored).
 */
export function applyZoomMenuAction(action: MenuAction): boolean {
  if (!isZoomMenuAction(action)) {
    return false;
  }
  if (!isMenuZoomWindowFocused()) {
    return true;
  }
  const viewer = getViewerControls();
  if (viewer !== null) {
    ZOOM_BY_ACTION[action](viewer);
  }
  return true;
}
