//! Run the embedded sidecars (`ktx`, and legacy `ktx2ktx2` for KTX1 input).

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

use crate::error::AppError;

/// Binaries listed in `bundle.externalBin` and staged by `npm run fetch-ktx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sidecar {
    Ktx,
    /// KTX1 → KTX2 converter; `ktx extract` only reads KTX2.
    Ktx2Ktx2,
}

impl Sidecar {
    pub fn name(self) -> &'static str {
        match self {
            Self::Ktx => "ktx",
            Self::Ktx2Ktx2 => "ktx2ktx2",
        }
    }
}

const CANCEL_POLL: Duration = Duration::from_millis(25);

/// Shared cancel flag for a running sidecar process.
#[derive(Debug, Clone, Default)]
pub struct CancellationHandle {
    cancelled: Arc<AtomicBool>,
}

impl CancellationHandle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Resolves once [`cancel`](Self::cancel) has been called.
    async fn cancelled(&self) {
        while !self.is_cancelled() {
            tokio::time::sleep(CANCEL_POLL).await;
        }
    }
}

#[derive(Debug, Clone)]
pub struct KtxOutput {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Children still running, by PID. Tauri ends the process on exit without dropping
/// pending tasks, so [`kill_all`] is the only thing that stops them then.
fn running() -> MutexGuard<'static, HashMap<u32, CommandChild>> {
    static RUNNING: OnceLock<Mutex<HashMap<u32, CommandChild>>> = OnceLock::new();
    RUNNING
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// Kill every sidecar process still running (app exit).
pub fn kill_all() {
    let children: Vec<CommandChild> = running().drain().map(|(_, child)| child).collect();
    for child in children {
        let _ = child.kill();
    }
}

/// One registered child. Dropping it kills the process unless it already exited,
/// so cancel, a process error, and an abandoned future all stop the child.
struct RunningChild {
    pid: u32,
}

impl RunningChild {
    fn register(child: CommandChild) -> Self {
        let pid = child.pid();
        running().insert(pid, child);
        Self { pid }
    }

    /// The process ended on its own: stop tracking it without a kill.
    fn exited(&self) {
        running().remove(&self.pid);
    }
}

impl Drop for RunningChild {
    fn drop(&mut self) {
        let child = running().remove(&self.pid);
        if let Some(child) = child {
            let _ = child.kill();
        }
    }
}

/// Collect sidecar output. Returns `Ok` for any exit code (including non-zero).
/// Used by inspect when invalid files still emit JSON on stdout.
pub async fn run_collect(
    app: &AppHandle,
    tool: Sidecar,
    args: Vec<String>,
    on_stderr: impl Fn(String) + Send + 'static,
    cancel: CancellationHandle,
) -> Result<KtxOutput, AppError> {
    let name = tool.name();
    let (mut rx, child) = app
        .shell()
        .sidecar(name)
        .map_err(|e| AppError::io(format!("sidecar {name}: {e}")))?
        .args(args)
        .spawn()
        .map_err(|e| AppError::io(format!("spawn {name}: {e}")))?;
    let child = RunningChild::register(child);

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut code: i32 = -1;

    loop {
        let event = tokio::select! {
            event = rx.recv() => event,
            () = cancel.cancelled() => return Err(AppError::cancelled()),
        };
        let Some(event) = event else {
            break;
        };
        match event {
            CommandEvent::Stdout(line) => {
                stdout.push_str(&String::from_utf8_lossy(&line));
            }
            CommandEvent::Stderr(line) => {
                let text = String::from_utf8_lossy(&line).into_owned();
                on_stderr(text.clone());
                stderr.push_str(&text);
            }
            CommandEvent::Terminated(payload) => {
                child.exited();
                code = payload.code.unwrap_or(-1);
            }
            CommandEvent::Error(err) => {
                return Err(AppError::io(format!("{name} process error: {err}")));
            }
            _ => {}
        }
    }

    if cancel.is_cancelled() {
        return Err(AppError::cancelled());
    }

    Ok(KtxOutput {
        code,
        stdout,
        stderr,
    })
}

/// Run a sidecar. Non-zero exit → [`AppError::Ktx`].
/// Cancellation kills the child and returns [`AppError::Cancelled`].
pub async fn run(
    app: &AppHandle,
    tool: Sidecar,
    args: Vec<String>,
    on_stderr: impl Fn(String) + Send + 'static,
    cancel: CancellationHandle,
) -> Result<KtxOutput, AppError> {
    let out = run_collect(app, tool, args, on_stderr, cancel).await?;
    if out.code != 0 {
        return Err(AppError::ktx(out.code, out.stderr));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::staged_sidecar;
    use std::process::Command;

    #[test]
    fn staged_sidecars_report_pinned_version() {
        for tool in [Sidecar::Ktx, Sidecar::Ktx2Ktx2] {
            let output = Command::new(staged_sidecar(tool))
                .arg("--version")
                .output()
                .expect("run --version");
            assert!(
                output.status.success(),
                "{}: stderr={}",
                tool.name(),
                String::from_utf8_lossy(&output.stderr)
            );
            // `ktx2ktx2` prints its version on stderr.
            let reported = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(reported.contains("v4.4.2"), "{}: {reported}", tool.name());
        }
    }

    #[tokio::test]
    async fn cancelled_waits_for_cancel() {
        let cancel = CancellationHandle::new();
        let early = tokio::time::timeout(CANCEL_POLL * 4, cancel.cancelled()).await;
        assert!(early.is_err(), "resolved without cancel");

        let trigger = cancel.clone();
        tokio::spawn(async move {
            tokio::time::sleep(CANCEL_POLL).await;
            trigger.cancel();
        });
        let resolved = tokio::time::timeout(CANCEL_POLL * 8, cancel.cancelled()).await;
        assert!(resolved.is_ok(), "did not resolve after cancel");
    }
}
