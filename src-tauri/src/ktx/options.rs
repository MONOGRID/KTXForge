//! `CreateOptions` and related enums. Filled in by T2.1.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use ts_rs::TS;

/// Placeholder enum so `strum` is linked until T2.1 adds real CLI enums.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, EnumIter, Display)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum EncodeCodecStub {
    BasisLz,
    Uastc,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CreateOptions {
    /// Stub until T2.1 fills every flag group.
    pub stub: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AnimationOptions {
    pub duration: u32,
    pub timescale: u32,
    pub loop_count: u32,
}
