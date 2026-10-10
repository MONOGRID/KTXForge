//! Event name constants and payload stubs. Payloads are filled in T2.3.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const ENCODER_SOURCES_ADDED: &str = "encoder://sources-added";
pub const ENCODER_DROP_REJECTED: &str = "encoder://drop-rejected";
pub const ENCODE_PROGRESS: &str = "encode://progress";
pub const INSPECTOR_FILE: &str = "inspector://file";
pub const UPDATE_STATUS: &str = "update://status";

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DropRejectedPayload {
    pub paths: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EncodeProgressPayload {
    pub job_id: String,
    pub stage: String,
    pub stderr_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct InspectorFilePayload {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusPayload {
    pub state: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub downloaded: Option<u64>,
    pub total: Option<u64>,
    pub message: Option<String>,
}
