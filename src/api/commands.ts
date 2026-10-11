import { invoke } from "@tauri-apps/api/core";
import type { CreateOptions } from "../bindings/CreateOptions";
import type { EncodeRequest } from "../bindings/EncodeRequest";
import type { EncodeResult } from "../bindings/EncodeResult";
import type { ExportRequest } from "../bindings/ExportRequest";
import type { ExportResult } from "../bindings/ExportResult";
import type { ImageSelection } from "../bindings/ImageSelection";
import type { InspectReport } from "../bindings/InspectReport";
import type { KtxVersion } from "../bindings/KtxVersion";
import type { OptionReport } from "../bindings/OptionReport";
import type { SourceInfo } from "../bindings/SourceInfo";
import type { UpdateInfo } from "../bindings/UpdateInfo";
import { toAppError } from "./guards";

type EmptyArgs = Record<string, never>;

export type Commands = {
  ktx_version: { args: EmptyArgs; result: KtxVersion };
  default_create_options: { args: EmptyArgs; result: CreateOptions };
  describe_options: {
    args: { options: CreateOptions; sources: SourceInfo[] };
    result: OptionReport;
  };
  pick_sources: { args: EmptyArgs; result: SourceInfo[] };
  source_thumbnail: { args: { path: string }; result: ArrayBuffer };
  encode: { args: { request: EncodeRequest }; result: EncodeResult };
  cancel_encode: { args: { jobId: string }; result: null };
  encoded_preview: { args: { jobId: string }; result: ArrayBuffer };
  save_encoded: { args: { jobId: string }; result: string | null };
  open_inspector: { args: { path: string }; result: null };
  inspector_current_file: { args: EmptyArgs; result: string | null };
  inspect: { args: { path: string }; result: InspectReport };
  inspector_image: {
    args: { path: string; selection: ImageSelection };
    result: ArrayBuffer;
  };
  export_images: { args: { request: ExportRequest }; result: ExportResult };
  check_update: { args: EmptyArgs; result: UpdateInfo | null };
  install_update: { args: EmptyArgs; result: null };
  set_menu_item_enabled: {
    args: { id: string; enabled: boolean };
    result: null;
  };
};

export async function call<K extends keyof Commands>(
  name: K,
  args: Commands[K]["args"],
): Promise<Commands[K]["result"]> {
  try {
    return await invoke<Commands[K]["result"]>(name, args);
  } catch (err: unknown) {
    const appError = toAppError(err);
    throw Object.assign(new Error(appError.message), appError);
  }
}
