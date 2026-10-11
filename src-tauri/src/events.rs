//! Event name constants and payloads.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::commands::SourceInfo;

pub const ENCODER_SOURCES_ADDED: &str = "encoder://sources-added";
pub const ENCODER_DROP_REJECTED: &str = "encoder://drop-rejected";
pub const ENCODE_PROGRESS: &str = "encode://progress";
pub const INSPECTOR_FILE: &str = "inspector://file";
pub const UPDATE_STATUS: &str = "update://status";
pub const MENU_ACTION: &str = "menu://action";

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DropRejectedPayload {
    pub paths: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum EncodeStage {
    Converting,
    Encoding,
    WritingAnim,
    Previewing,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EncodeProgressPayload {
    pub job_id: String,
    pub stage: EncodeStage,
    pub stderr_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct InspectorFilePayload {
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum UpdateState {
    Available,
    Downloading,
    Installed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusPayload {
    pub state: UpdateState,
    pub version: Option<String>,
    pub notes: Option<String>,
    #[ts(type = "number")]
    pub downloaded: Option<u64>,
    #[ts(type = "number")]
    pub total: Option<u64>,
    pub message: Option<String>,
}

/// Typed payload for `encoder://sources-added` (exported for the frontend).
pub type SourcesAddedPayload = Vec<SourceInfo>;

/// Native menu command that needs frontend state (encode job, sources, export UI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum MenuAction {
    Encode,
    CancelEncode,
    Save,
    ClearSources,
    Export,
}
