import { message } from "@tauri-apps/plugin-dialog";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import type { AppError } from "../bindings/AppError";
import { formatAppError } from "./formatAppError";

async function ensureNotificationPermission(): Promise<boolean> {
  let granted = await isPermissionGranted();
  if (!granted) {
    const permission = await requestPermission();
    granted = permission === "granted";
  }
  return granted;
}

/** Best-effort OS notification (encode job complete). */
export async function notifyText(
  title: string,
  body: string,
): Promise<void> {
  const trimmed = body.trim();
  if (trimmed.length === 0) {
    return;
  }
  try {
    if (!(await ensureNotificationPermission())) {
      return;
    }
    sendNotification({ title, body: trimmed });
  } catch {
    // OS notifications are best-effort (e.g. unsupported in some dev hosts).
  }
}

async function showAlert(body: string, title: string): Promise<void> {
  const trimmed = body.trim();
  if (trimmed.length === 0) {
    return;
  }
  await message(trimmed, { title, kind: "error" });
}

/**
 * Encode job failure: OS notification + native alert with one shared body.
 */
export async function alertErrorText(
  body: string,
  title = "Error",
): Promise<void> {
  const trimmed = body.trim();
  if (trimmed.length === 0) {
    return;
  }
  await notifyText(title, trimmed);
  await showAlert(trimmed, title);
}

/**
 * Soft / non-job failure: native alert only (no OS notification).
 * Replaces former Sonner toasts outside encode jobs.
 */
export async function alertAppError(
  error: AppError,
  title = "Error",
): Promise<void> {
  if (error.kind === "cancelled") {
    return;
  }
  await showAlert(formatAppError(error), title);
}

/** Soft invalid-input notice: native alert only (no OS notification). */
export async function alertInvalidInput(messageText: string): Promise<void> {
  await showAlert(messageText, "Invalid input");
}
