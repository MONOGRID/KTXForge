import { message } from "@tauri-apps/plugin-dialog";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

async function ensureNotificationPermission(): Promise<boolean> {
  let granted = await isPermissionGranted();
  if (!granted) {
    const permission = await requestPermission();
    granted = permission === "granted";
  }
  return granted;
}

async function notifyOs(title: string, body: string): Promise<void> {
  try {
    if (!(await ensureNotificationPermission())) {
      return;
    }
    sendNotification({ title, body });
  } catch {
    // OS notifications are best-effort (e.g. unsupported in some dev hosts).
  }
}

/** Native alert dialog + OS notification with one shared message body. */
export async function alertErrorText(
  body: string,
  title = "Error",
): Promise<void> {
  const trimmed = body.trim();
  if (trimmed.length === 0) {
    return;
  }
  await notifyOs(title, trimmed);
  await message(trimmed, { title, kind: "error" });
}
