use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

/// Errors returned from every Tauri command.
///
/// Serialized as `{ kind, message, exitCode?, exitMeaning?, stderr? }` for the webview.
#[derive(Debug, Error, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AppError {
    #[error("{message}")]
    NotImplemented { message: String },
    #[error("{message}")]
    Io { message: String },
    #[error("{message}")]
    InvalidInput { message: String },
    #[error("{message}")]
    UnsupportedFile { message: String },
    #[error("{message}")]
    #[serde(rename_all = "camelCase")]
    Ktx {
        message: String,
        exit_code: i32,
        exit_meaning: String,
        stderr: String,
    },
    #[error("{message}")]
    Cancelled { message: String },
    #[error("{message}")]
    Image { message: String },
    #[error("{message}")]
    Update { message: String },
}

impl AppError {
    pub fn not_implemented(name: &'static str) -> Self {
        Self::NotImplemented {
            message: format!("not implemented: {name}"),
        }
    }

    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io {
            message: msg.into(),
        }
    }

    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: msg.into(),
        }
    }

    pub fn unsupported_file(msg: impl Into<String>) -> Self {
        Self::UnsupportedFile {
            message: msg.into(),
        }
    }

    pub fn ktx(exit_code: i32, stderr: impl Into<String>) -> Self {
        let stderr = stderr.into();
        Self::Ktx {
            message: format!("ktx exited with code {exit_code}"),
            exit_code,
            exit_meaning: Self::exit_meaning(exit_code).to_owned(),
            stderr,
        }
    }

    pub fn cancelled() -> Self {
        Self::Cancelled {
            message: "cancelled".to_owned(),
        }
    }

    pub fn image(msg: impl Into<String>) -> Self {
        Self::Image {
            message: msg.into(),
        }
    }

    pub fn update(msg: impl Into<String>) -> Self {
        Self::Update {
            message: msg.into(),
        }
    }

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
