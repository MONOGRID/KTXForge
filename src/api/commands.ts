import { invoke } from "@tauri-apps/api/core";
import type { CreateOptions } from "../bindings/CreateOptions";
import type { EncodePresetInfo } from "../bindings/EncodePresetInfo";
import type { EncodeRequest } from "../bindings/EncodeRequest";
import type { EncodeResult } from "../bindings/EncodeResult";
import type { ExportRequest } from "../bindings/ExportRequest";
import type { ExportResult } from "../bindings/ExportResult";
import type { ImageSelection } from "../bindings/ImageSelection";
import type { InputRole } from "../bindings/InputRole";
import type { InputValidationReport } from "../bindings/InputValidationReport";
import type { InspectReport } from "../bindings/InspectReport";
import type { KtxVersion } from "../bindings/KtxVersion";
import type { OptionReport } from "../bindings/OptionReport";
import type { RoleArity } from "../bindings/RoleArity";
import type { SourceInfo } from "../bindings/SourceInfo";
import type { UpdateInfo } from "../bindings/UpdateInfo";
import { toAppError } from "./guards";

type EmptyArgs = Record<string, never>;

export type Commands = {
  extract_concurrency: { args: EmptyArgs; result: number };
  ktx_version: { args: EmptyArgs; result: KtxVersion };
  default_create_options: { args: EmptyArgs; result: CreateOptions };
  list_encode_presets: { args: EmptyArgs; result: EncodePresetInfo[] };
  apply_encode_preset: {
    args: { options: CreateOptions; id: string };
    result: CreateOptions;
  };
  encode_preset_matches: {
    args: { options: CreateOptions; id: string };
    result: boolean;
  };
  describe_options: {
    args: {
      options: CreateOptions;
      sources: SourceInfo[];
      inputRole: InputRole;
    };
    result: OptionReport;
  };
  input_role_arities: { args: EmptyArgs; result: RoleArity[] };
  validate_pack_inputs: {
    args: { sources: SourceInfo[]; inputRole: InputRole };
    result: InputValidationReport;
  };
  check_output_compatible: {
    args: { anchor: SourceInfo; candidate: SourceInfo };
    result: null;
  };
  pick_sources: { args: EmptyArgs; result: SourceInfo[] };
  source_thumbnail: { args: { path: string }; result: ArrayBuffer };
  source_preview: { args: { path: string }; result: ArrayBuffer };
  encode: { args: { request: EncodeRequest }; result: EncodeResult };
  cancel_encode: { args: { jobId: string }; result: null };
  discard_outputs: { args: { outputIds: string[] }; result: null };
  encoded_preview: {
    args: { jobId: string; selection: ImageSelection };
    result: ArrayBuffer;
  };
  save_encoded: { args: { jobId: string }; result: string | null };
  save_encoded_batch: {
    args: { jobIds: string[] };
    result: number | null;
  };
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
  dispatch_menu_action: {
    args: { id: string };
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
