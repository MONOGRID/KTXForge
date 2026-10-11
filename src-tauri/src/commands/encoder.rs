use std::path::PathBuf;

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::commands::inspector_window;
use crate::commands::{OptionReport, SourceInfo};
use crate::error::AppError;
use crate::image::classify::{classify, FileKind};
use crate::image::convert::{read_source_info, source_preview_png, thumbnail_png};
use crate::ktx::input_role::{self, InputRole};
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

/// Whether `candidate` may be added as a new KTX output given the queue's first primary.
#[tauri::command]
pub fn check_output_compatible(anchor: SourceInfo, candidate: SourceInfo) -> Result<(), AppError> {
    input_role::compatible_with_anchor(&anchor, &candidate).map_err(AppError::invalid_input)
}

#[tauri::command]
pub async fn pick_sources(app: AppHandle) -> Result<Vec<SourceInfo>, AppError> {
    let picked = app
        .dialog()
        .file()
        .add_filter("Images and KTX", SOURCE_EXTENSIONS)
        .blocking_pick_files();

    let Some(files) = picked else {
        return Ok(Vec::new());
    };

    let mut sources = Vec::new();
    let mut first_ktx: Option<PathBuf> = None;

    for file in files {
        let path = file
            .into_path()
            .map_err(|e| AppError::io(format!("dialog path: {e}")))?;
        match classify(&path) {
            FileKind::Image { .. } => {
                sources.push(read_source_info(&path)?);
            }
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

    if let Some(ktx) = first_ktx {
        let state = app.state::<AppState>();
        inspector_window::open_inspector(app.clone(), state, ktx.to_string_lossy().into_owned())
            .await?;
    }

    Ok(sources)
}

#[tauri::command]
pub async fn source_thumbnail(path: String) -> Result<tauri::ipc::Response, AppError> {
    let bytes = thumbnail_png(PathBuf::from(path).as_path(), THUMB_MAX)?;
    Ok(tauri::ipc::Response::new(bytes))
}

#[tauri::command]
pub async fn source_preview(path: String) -> Result<tauri::ipc::Response, AppError> {
    let bytes = source_preview_png(PathBuf::from(path).as_path())?;
    Ok(tauri::ipc::Response::new(bytes))
}
