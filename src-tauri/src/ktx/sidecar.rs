//! Run the embedded `ktx` sidecar.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

use crate::error::AppError;

/// Shared cancel flag for a running `ktx` process.
#[derive(Debug, Clone, Default)]
pub struct CancellationHandle {
    cancelled: Arc<AtomicBool>,
}

impl CancellationHandle {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
pub struct KtxOutput {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn exit_meaning(code: i32) -> &'static str {
    AppError::exit_meaning(code)
}

/// Stop the cancel-watcher task.
///
/// When cancelled, await so the child kill can finish. Otherwise abort — the
/// watcher loops until the cancel flag is set, so awaiting it on the success
/// path hangs forever (inspector stuck on "Loading…").
async fn finish_cancel_watcher(cancel_task: tauri::async_runtime::JoinHandle<()>, cancelled: bool) {
    if cancelled {
        let _ = cancel_task.await;
    } else {
        cancel_task.abort();
    }
}

/// Collect sidecar output. Returns `Ok` for any exit code (including non-zero).
/// Used by inspect when invalid files still emit JSON on stdout.
pub async fn run_ktx_collect(
    app: &AppHandle,
    args: Vec<String>,
    on_stderr: impl Fn(String) + Send + 'static,
    cancel: CancellationHandle,
) -> Result<KtxOutput, AppError> {
    let cmd = app
        .shell()
        .sidecar("ktx")
        .map_err(|e| AppError::io(format!("sidecar ktx: {e}")))?
        .args(args);

    let (mut rx, child) = cmd
        .spawn()
        .map_err(|e| AppError::io(format!("spawn ktx: {e}")))?;

    let child = Arc::new(Mutex::new(Some(child)));
    let cancel_flag = cancel.cancelled.clone();
    let killer = Arc::clone(&child);
    let cancel_task = tauri::async_runtime::spawn(async move {
        while !cancel_flag.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        if let Ok(mut guard) = killer.lock() {
            if let Some(child) = guard.take() {
                let _ = child.kill();
            }
        }
    });

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut code: i32 = -1;

    while let Some(event) = rx.recv().await {
        if cancel.is_cancelled() {
            finish_cancel_watcher(cancel_task, true).await;
            return Err(AppError::cancelled());
        }
        match event {
            CommandEvent::Stdout(line) => {
                let text = String::from_utf8_lossy(&line);
                stdout.push_str(&text);
            }
            CommandEvent::Stderr(line) => {
                let text = String::from_utf8_lossy(&line).into_owned();
                on_stderr(text.clone());
                stderr.push_str(&text);
            }
            CommandEvent::Terminated(payload) => {
                code = payload.code.unwrap_or(-1);
            }
            CommandEvent::Error(err) => {
                finish_cancel_watcher(cancel_task, cancel.is_cancelled()).await;
                return Err(AppError::io(format!("ktx process error: {err}")));
            }
            _ => {}
        }
    }

    finish_cancel_watcher(cancel_task, cancel.is_cancelled()).await;

    if cancel.is_cancelled() {
        return Err(AppError::cancelled());
    }

    Ok(KtxOutput {
        code,
        stdout,
        stderr,
    })
}

/// Run `ktx` via the Tauri shell sidecar. Non-zero exit → [`AppError::Ktx`].
/// Cancellation kills the child and returns [`AppError::Cancelled`].
pub async fn run_ktx(
    app: &AppHandle,
    args: Vec<String>,
    on_stderr: impl Fn(String) + Send + 'static,
    cancel: CancellationHandle,
) -> Result<KtxOutput, AppError> {
    let out = run_ktx_collect(app, args, on_stderr, cancel).await?;
    if out.code != 0 {
        return Err(AppError::ktx(out.code, out.stderr));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;

    #[test]
    fn exit_meaning_covers_known_codes() {
        assert_eq!(exit_meaning(0), "Success");
        assert_eq!(exit_meaning(1), "Command line error");
        assert_eq!(exit_meaning(2), "IO failure");
        assert_eq!(exit_meaning(3), "Invalid input file");
        assert_eq!(exit_meaning(4), "Runtime or library error");
        assert_eq!(exit_meaning(5), "Unsupported state or operation");
        assert_eq!(exit_meaning(6), "Feature not implemented yet");
        assert_eq!(exit_meaning(99), "Unknown exit code");
        assert_eq!(exit_meaning(-1), "Unknown exit code");
    }

    fn staged_ktx_binary() -> Option<PathBuf> {
        let binaries = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
        let entries = std::fs::read_dir(&binaries).ok()?;
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("ktx-") {
                continue;
            }
            #[cfg(windows)]
            if name.ends_with(".exe") {
                return Some(entry.path());
            }
            #[cfg(not(windows))]
            if !name.contains('.') {
                return Some(entry.path());
            }
        }
        None
    }

    #[test]
    fn version_via_staged_binary() {
        let Some(bin) = staged_ktx_binary() else {
            // binaries/ is gitignored; skip when fetch-ktx has not been run.
            return;
        };
        let output = Command::new(&bin)
            .arg("--version")
            .output()
            .expect("run ktx --version");
        assert!(
            output.status.success(),
            "stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.to_ascii_lowercase().contains("ktx") || !stdout.trim().is_empty(),
            "unexpected version stdout: {stdout}"
        );
    }

    /// Awaiting the cancel watcher without setting the flag hangs (the old bug).
    #[tokio::test]
    async fn awaiting_cancel_watcher_without_flag_hangs() {
        let cancel = CancellationHandle::new();
        let cancel_flag = cancel.cancelled.clone();
        let cancel_task = tokio::spawn(async move {
            while !cancel_flag.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        });

        let result = tokio::time::timeout(Duration::from_millis(200), cancel_task).await;
        assert!(
            result.is_err(),
            "await without cancel should hang; if this fails the watcher model changed"
        );
    }

    /// Success-path cleanup must finish quickly (inspector "Loading…" regression).
    #[tokio::test]
    async fn finish_cancel_watcher_on_success_completes_quickly() {
        let cancel = CancellationHandle::new();
        let cancel_flag = cancel.cancelled.clone();
        let cancel_task = tauri::async_runtime::spawn(async move {
            while !cancel_flag.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        });

        let result = tokio::time::timeout(
            Duration::from_millis(200),
            finish_cancel_watcher(cancel_task, false),
        )
        .await;
        assert!(
            result.is_ok(),
            "success-path cancel-watcher cleanup hung (inspector would stay on Loading…)"
        );
    }

    #[tokio::test]
    async fn finish_cancel_watcher_on_cancel_awaits_exit() {
        let cancel = CancellationHandle::new();
        let cancel_flag = cancel.cancelled.clone();
        let cancel_task = tauri::async_runtime::spawn(async move {
            while !cancel_flag.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        });
        cancel.cancel();

        let result = tokio::time::timeout(
            Duration::from_millis(200),
            finish_cancel_watcher(cancel_task, true),
        )
        .await;
        assert!(result.is_ok(), "cancelled watcher cleanup hung");
    }
}
