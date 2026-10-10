use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

/// Errors returned from every Tauri command.
///
/// Serialized as `{ kind, message, exitCode?, exitMeaning?, stderr? }` for the webview.
#[derive(Debug, Error, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum AppError {
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    InvalidInput(String),
    #[error("{0}")]
    UnsupportedFile(String),
    #[error("ktx exited with code {exit_code}")]
    Ktx { exit_code: i32, stderr: String },
    #[error("cancelled")]
    Cancelled,
    #[error("{0}")]
    Image(String),
    #[error("{0}")]
    Update(String),
}

impl AppError {
    pub fn exit_meaning(code: i32) -> &'static str {
        match code {
            0 => "Success",
            1 => "Command line error",
            2 => "IO failure",
            3 => "Invalid input file",
            4 => "Runtime or library error",
            5 => "Unsupported state or operation",
            6 => "Feature not implemented yet",
            _ => "Unknown exit code",
        }
    }

    /// JSON form used when surfacing errors to the webview (keeps `serde_json` linked).
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self)
            .unwrap_or_else(|_| serde_json::json!({ "kind": "io", "message": self.to_string() }))
    }
}
