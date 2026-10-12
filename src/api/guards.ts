import type { AppError } from "../bindings/AppError";

const APP_ERROR_KINDS = new Set([
  "io",
  "invalidInput",
  "unsupportedFile",
  "ktx",
  "cancelled",
  "image",
  "update",
]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function hasKtxFields(value: Record<string, unknown>): boolean {
  return (
    typeof value.exitCode === "number" &&
    typeof value.exitMeaning === "string" &&
    typeof value.stderr === "string"
  );
}

function isAppError(value: unknown): value is AppError {
  if (!isRecord(value) || typeof value.kind !== "string") {
    return false;
  }
  if (!APP_ERROR_KINDS.has(value.kind) || typeof value.message !== "string") {
    return false;
  }
  return value.kind !== "ktx" || hasKtxFields(value);
}

export function toAppError(err: unknown): AppError {
  if (isAppError(err)) {
    return err;
  }
  if (isRecord(err) && typeof err.message === "string") {
    return { kind: "io", message: err.message };
  }
  if (typeof err === "string") {
    return { kind: "io", message: err };
  }
  return { kind: "io", message: "Unknown error" };
}
