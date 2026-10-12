use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::blocking::run_blocking;
use crate::commands::inspector_window;
use crate::commands::{OptionReport, SourceInfo};
use crate::dialog;
use crate::error::AppError;
use crate::image::classify::{classify, FileKind};
use crate::image::convert::{read_source_info, source_preview_png, thumbnail_png};
use crate::ktx::input_role::{self, InputRole, RoleArity};
use crate::ktx::options::CreateOptions;
use crate::ktx::rules;
use crate::state::AppState;

const SOURCE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "bmp", "gif", "tif", "tiff", "exr", "hdr", "rgbe", "xyze", "ktx",
    "ktx2",
];

const THUMB_MAX: u32 = 128;

#[tauri::command]
pub fn default_create_options() -> Result<CreateOptions, AppError> {
    Ok(CreateOptions::default())
}

#[tauri::command]
pub fn describe_options(
    options: CreateOptions,
    sources: Vec<SourceInfo>,
    input_role: InputRole,
) -> Result<OptionReport, AppError> {
    Ok(rules::describe(&options, &sources, input_role))
}

/// Input-count bounds for every role (drives the Inputs panel and Encode enablement).
#[tauri::command]
pub fn input_role_arities() -> Vec<RoleArity> {
    input_role::all_arities()
}

/// Whether `candidate` may be added as a new KTX output given the queue's first primary.
#[tauri::command]
pub fn check_output_compatible(anchor: SourceInfo, candidate: SourceInfo) -> Result<(), AppError> {
    input_role::compatible_with_anchor(&anchor, &candidate).map_err(AppError::invalid_input)
}

#[tauri::command]
pub async fn pick_sources(app: AppHandle) -> Result<Vec<SourceInfo>, AppError> {
    let picker = app
        .dialog()
        .file()
        .add_filter("Images and KTX", SOURCE_EXTENSIONS);
    let files = dialog::pick_files(picker).await?;

    let mut images = Vec::new();
    let mut first_ktx: Option<PathBuf> = None;

    for path in files {
        match classify(&path) {
            FileKind::Image { .. } => images.push(path),
            FileKind::Ktx1 | FileKind::Ktx2 => {
                if first_ktx.is_none() {
                    first_ktx = Some(path);
                }
            }
            FileKind::Unsupported => {
                return Err(AppError::unsupported_file(format!(
                    "unsupported file: {}",
                    path.display()
                )));
            }
        }
    }

    let sources = run_blocking(move || {
        images
            .iter()
            .map(|path| read_source_info(path))
            .collect::<Result<Vec<_>, _>>()
    })
    .await?;

    if let Some(ktx) = first_ktx {
        let state = app.state::<AppState>();
        inspector_window::open_inspector_path(&app, state.inner(), ktx).await?;
    }

    Ok(sources)
}

#[tauri::command]
pub async fn source_thumbnail(path: String) -> Result<tauri::ipc::Response, AppError> {
    let bytes = run_blocking(move || thumbnail_png(Path::new(&path), THUMB_MAX)).await?;
    Ok(tauri::ipc::Response::new(bytes))
}

#[tauri::command]
pub async fn source_preview(path: String) -> Result<tauri::ipc::Response, AppError> {
    let bytes = run_blocking(move || source_preview_png(Path::new(&path))).await?;
    Ok(tauri::ipc::Response::new(bytes))
}
