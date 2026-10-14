//! Headless IPC server for the end-to-end suite (`e2e/`).
//!
//! Serves the app's Tauri commands, by the same names and argument keys the
//! webview uses, on a `MockRuntime` app with the real shell plugin, so `ktx`
//! sidecars resolve next to this binary exactly as in the app. One JSON request
//! per stdin line, `{"id", "cmd", "args"}`; one JSON response per stdout line,
//! `{"id", "ok"}` or `{"id", "err": AppError}`. Requests run concurrently, like
//! IPC calls from the webview. Byte responses (`tauri::ipc::Response`) are
//! `{"bytes": "<base64>"}`.
//!
//! Commands that open a native dialog have an `e2e_*` twin taking the path the
//! user would have picked.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use base64::Engine;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::{AppHandle, Manager};

use crate::blocking::run_blocking;
use crate::commands::app as app_commands;
use crate::commands::encode::{encoded_preview_png, run_encode};
use crate::commands::encoder;
use crate::commands::export::export_to;
use crate::commands::inspect::inspector_image_png;
use crate::commands::{EncodeRequest, ExportFormat, ExportSelection};
use crate::error::AppError;
use crate::image::convert::{read_source_info, source_preview_png, thumbnail_png};
use crate::ktx::extract_ops::texture_meta_from_report;
use crate::ktx::info;
use crate::ktx::info_types::InspectKind;
use crate::state::AppState;

#[derive(Deserialize)]
struct Request {
    id: u64,
    cmd: String,
    #[serde(default)]
    args: Value,
}

pub fn main() {
    let app = match mock_builder()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .build(mock_context(noop_assets()))
    {
        Ok(app) => app,
        Err(e) => {
            eprintln!("ktxforge-e2e: build app: {e}");
            std::process::exit(2);
        }
    };
    let handle = app.handle().clone();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            let Ok(line) = line else { break };
            if tx.send(line).is_err() {
                break;
            }
        }
    });

    tauri::async_runtime::block_on(async {
        let mut running = Vec::new();
        while let Some(line) = rx.recv().await {
            if line.trim().is_empty() {
                continue;
            }
            let app = handle.clone();
            running.push(tauri::async_runtime::spawn(async move {
                let response = match serde_json::from_str::<Request>(&line) {
                    Ok(request) => {
                        let outcome = dispatch(&app, &request.cmd, &request.args).await;
                        match outcome {
                            Ok(value) => json!({ "id": request.id, "ok": value }),
                            Err(err) => json!({ "id": request.id, "err": err }),
                        }
                    }
                    Err(e) => json!({ "id": null, "err": AppError::invalid_input(e.to_string()) }),
                };
                let mut stdout = std::io::stdout().lock();
                let _ = writeln!(stdout, "{response}");
                let _ = stdout.flush();
            }));
        }
        for task in running {
            let _ = task.await;
        }
    });
    handle.state::<AppState>().shutdown();
}

fn arg<T: DeserializeOwned>(args: &Value, key: &str) -> Result<T, AppError> {
    serde_json::from_value(args.get(key).cloned().unwrap_or(Value::Null))
        .map_err(|e| AppError::invalid_input(format!("argument `{key}`: {e}")))
}

fn value(value: impl serde::Serialize) -> Result<Value, AppError> {
    serde_json::to_value(value).map_err(|e| AppError::io(e.to_string()))
}

fn bytes(bytes: &[u8]) -> Value {
    json!({ "bytes": base64::engine::general_purpose::STANDARD.encode(bytes) })
}

async fn dispatch(
    app: &AppHandle<MockRuntime>,
    cmd: &str,
    args: &Value,
) -> Result<Value, AppError> {
    let state = app.state::<AppState>();
    match cmd {
        "ktx_version" => value(app_commands::ktx_version(app.clone()).await?),
        "extract_concurrency" => value(app_commands::extract_concurrency()),
        "default_create_options" => value(encoder::default_create_options()?),
        "list_encode_presets" => value(encoder::list_encode_presets()),
        "apply_encode_preset" => value(encoder::apply_encode_preset(
            arg(args, "options")?,
            arg(args, "id")?,
        )?),
        "encode_preset_matches" => value(encoder::encode_preset_matches(
            arg(args, "options")?,
            arg(args, "id")?,
        )),
        "describe_options" => value(encoder::describe_options(
            arg(args, "options")?,
            arg(args, "sources")?,
            arg(args, "inputRole")?,
        )?),
        "input_role_arities" => value(encoder::input_role_arities()),
        "validate_pack_inputs" => value(encoder::validate_pack_inputs(
            arg(args, "sources")?,
            arg(args, "inputRole")?,
        )),
        "check_output_compatible" => value(encoder::check_output_compatible(
            arg(args, "anchor")?,
            arg(args, "candidate")?,
        )?),
        "source_thumbnail" => {
            let path: String = arg(args, "path")?;
            Ok(bytes(
                &run_blocking(move || thumbnail_png(Path::new(&path), 128)).await?,
            ))
        }
        "source_preview" => {
            let path: String = arg(args, "path")?;
            Ok(bytes(
                &run_blocking(move || source_preview_png(Path::new(&path))).await?,
            ))
        }
        "encode" => {
            let request: EncodeRequest = arg(args, "request")?;
            value(run_encode(app, &state, request).await?)
        }
        "encoded_preview" => {
            let job_id: String = arg(args, "jobId")?;
            let png = encoded_preview_png(app, &state, &job_id, &arg(args, "selection")?).await?;
            Ok(bytes(&png))
        }
        "cancel_encode" => {
            let job_id: String = arg(args, "jobId")?;
            value(state.with_job(&job_id, |job| job.cancel.cancel())?)
        }
        "discard_outputs" => {
            let ids: Vec<String> = arg(args, "outputIds")?;
            value(state.discard_output_jobs(&ids)?)
        }
        "inspect" => {
            let path = PathBuf::from(arg::<String>(args, "path")?);
            let report = info::inspect(app, &path).await?;
            if matches!(report.kind, InspectKind::Ktx2 { .. }) {
                state.set_inspector_meta(path, texture_meta_from_report(&report));
            }
            value(report)
        }
        "inspector_image" => {
            let path = PathBuf::from(arg::<String>(args, "path")?);
            let png = inspector_image_png(app, &state, path, &arg(args, "selection")?).await?;
            Ok(bytes(&png))
        }
        // What a drop or the Open dialog reports for a file.
        "e2e_source_info" => {
            let path: String = arg(args, "path")?;
            value(run_blocking(move || read_source_info(Path::new(&path))).await?)
        }
        // Save… with `dest` picked in the dialog.
        "e2e_save_encoded" => {
            let job_id: String = arg(args, "jobId")?;
            let dest: String = arg(args, "dest")?;
            let output = state.with_job(&job_id, |job| job.output_path.clone())?;
            tokio::fs::copy(&output, &dest)
                .await
                .map_err(|e| AppError::io(format!("{dest}: {e}")))?;
            value(dest)
        }
        // Export… with `dest` picked in the dialog (a file for one image, a folder for many).
        "e2e_export_images" => {
            let path: String = arg(args, "path")?;
            let selection: ExportSelection = arg(args, "selection")?;
            let format: ExportFormat = arg(args, "format")?;
            let dest: String = arg(args, "dest")?;
            value(
                export_to(
                    app,
                    &state,
                    Path::new(&path),
                    &selection,
                    &format,
                    Path::new(&dest),
                )
                .await?,
            )
        }
        other => Err(AppError::invalid_input(format!(
            "ktxforge-e2e does not serve `{other}`"
        ))),
    }
}
