pub mod app;
pub mod encode;
pub mod encoder;
pub mod export;
pub mod inspect;
pub mod inspector_window;
pub mod update;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ktx::info_types::{ImageSelection, PreviewKind};
use crate::ktx::input_role::InputRole;
use crate::ktx::options::{AnimationOptions, CreateOptions};

#[allow(unused_imports)]
pub use crate::ktx::options::{FieldState, OptionKey, OptionReport, OptionWarning, WarningLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum SourceKind {
    Ldr,
    Hdr,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub path: String,
    pub name: String,
    pub kind: SourceKind,
    pub width: u32,
    pub height: u32,
    pub needs_conversion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EncodeRequest {
    pub job_id: String,
    pub sources: Vec<String>,
    pub options: CreateOptions,
    pub animation: Option<AnimationOptions>,
    pub input_role: InputRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EncodeResult {
    pub job_id: String,
    #[ts(type = "number")]
    pub output_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub levels: u32,
    pub layers: u32,
    pub faces: u32,
    pub codec: String,
    pub ssim: Option<f64>,
    pub psnr: Option<f64>,
    pub warnings: Vec<String>,
    pub preview: PreviewKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ExportSelection {
    One { selection: ImageSelection },
    AllFrames { level: u32 },
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ExportFormat {
    Png,
    Jpeg { quality: u8 },
    Exr,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub path: String,
    pub selection: ExportSelection,
    pub format: ExportFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct KtxVersion {
    pub pinned: String,
    pub reported: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
    pub date: String,
}
