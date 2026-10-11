import { toast } from "vue-sonner";
import type { AppError } from "../bindings/AppError";
import { formatAppError } from "./formatAppError";

/** Show an AppError as a destructive toast (soft / non-encode failures). */
export function toastAppError(error: AppError): void {
  if (error.kind === "cancelled") {
    return;
  }
  toast.error(formatAppError(error));
}
