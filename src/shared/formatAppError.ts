import type { AppError } from "../bindings/AppError";

/** One user-facing string for an AppError (no duplicated exit/stderr lines). */
export function formatAppError(error: AppError): string {
  if (error.kind === "cancelled") {
    return "";
  }
  if (error.kind === "ktx") {
    const stderr = error.stderr.trim();
    const exitLine = `Exit ${String(error.exitCode)} — ${error.exitMeaning}`;
    if (stderr.length === 0) {
      return exitLine;
    }
    return `${stderr}\n\n(${exitLine})`;
  }
  return error.message;
}
