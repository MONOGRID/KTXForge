//! Types for `ktx info --format json` and inspector reports. Filled in by T2.2 / T2.3.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct InspectReport {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ImageSelection {
    pub level: u32,
    pub face: u32,
    pub layer: u32,
    pub depth_slice: u32,
}
