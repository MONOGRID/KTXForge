import { toast } from "vue-sonner";
import type { AppError } from "../bindings/AppError";

function ktxDescription(error: Extract<AppError, { kind: "ktx" }>): string {
  const stderr = error.stderr.trim();
  const exitLine = `Exit ${String(error.exitCode)} — ${error.exitMeaning}`;
  if (stderr.length === 0) {
    return exitLine;
  }
  return `${exitLine}\n${stderr}`;
}

/** Show an AppError as a destructive toast (encoder + inspector). */
export function toastAppError(error: AppError): void {
  if (error.kind === "cancelled") {
    return;
  }
  if (error.kind === "ktx") {
    toast.error(error.message, { description: ktxDescription(error) });
    return;
  }
  toast.error(error.message);
}
