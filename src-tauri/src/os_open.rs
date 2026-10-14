//! OS-level "Open with" / double-click handling for `.ktx` / `.ktx2`.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager, Url};

use crate::commands::inspector_window;
use crate::image::classify::{classify, FileKind};
use crate::state::AppState;

#[cfg(target_os = "macos")]
use std::sync::OnceLock;
#[cfg(target_os = "macos")]
use std::time::{Duration, Instant};

#[cfg(target_os = "macos")]
static LAUNCHED_AT: OnceLock<Instant> = OnceLock::new();

/// Call once at process start so macOS `Opened` can detect a cold viewer launch.
pub fn note_process_start() {
    #[cfg(target_os = "macos")]
    {
        let _ = LAUNCHED_AT.set(Instant::now());
    }
}

/// `true` when argv (this process) already named a KTX file — used to keep the
/// encoder window from being restored/shown by window-state.
pub fn cold_ktx_from_argv() -> bool {
    !ktx_paths_from_open_args(std::env::args().skip(1)).is_empty()
}

/// How aggressively to drop the encoder window when opening a KTX from the OS.
#[derive(Clone, Copy)]
enum SuppressMain {
    /// Only if still hidden (Linux/Windows argv cold start after skip-restore).
    IfHidden,
    /// Also if the app just launched (macOS `Opened` after window-state showed main).
    #[cfg(target_os = "macos")]
    IfHiddenOrLaunchGrace,
}

/// Open the first KTX path from argv-like strings (executable already skipped by caller).
pub fn open_from_args<I, S>(app: &AppHandle, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    open_ktx_paths(app, ktx_paths_from_open_args(args), SuppressMain::IfHidden);
}

/// Open the first KTX path from `RunEvent::Opened` URLs (macOS).
#[cfg(target_os = "macos")]
pub fn open_from_urls(app: &AppHandle, urls: &[Url]) {
    let paths = urls
        .iter()
        .filter_map(|url| url.to_file_path().ok())
        .filter(|path| is_ktx_path(path))
        .collect::<Vec<_>>();
    open_ktx_paths(app, paths, SuppressMain::IfHiddenOrLaunchGrace);
}

fn open_ktx_paths(app: &AppHandle, paths: Vec<PathBuf>, suppress: SuppressMain) {
    let Some(path) = paths.into_iter().next() else {
        return;
    };
    suppress_encoder_window(app, suppress);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        if let Err(e) = inspector_window::open_inspector_path(&app, state.inner(), path).await {
            eprintln!("os open inspector: {e}");
        }
    });
}

/// Viewer association should not leave an empty encoder window beside the inspector.
fn suppress_encoder_window(app: &AppHandle, suppress: SuppressMain) {
    let Some(main) = app.get_webview_window("main") else {
        return;
    };
    let visible = main.is_visible().unwrap_or(false);
    let drop = match suppress {
        SuppressMain::IfHidden => !visible,
        #[cfg(target_os = "macos")]
        SuppressMain::IfHiddenOrLaunchGrace => {
            let within_grace = LAUNCHED_AT
                .get()
                .is_some_and(|t| t.elapsed() < Duration::from_secs(2));
            !visible || within_grace
        }
    };
    if drop {
        // Detach the app menu before destroy. On Windows, DestroyWindow frees the
        // window's HMENU; on GTK, tearing down the window destroys the menubar
        // widgets. Either way the shared app-wide Menu becomes unusable and the
        // inspector that opens next would have no menu bar.
        #[cfg(desktop)]
        {
            let _ = main.remove_menu();
        }
        let _ = main.destroy();
    }
}

/// Parse open-with arguments into existing (or extension-classified) KTX paths.
///
/// Skips empty strings and CLI flags (`-…`). Accepts plain paths and `file://` URLs.
pub fn ktx_paths_from_open_args<I, S>(args: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .filter_map(|arg| {
            let s = arg.as_ref().trim();
            if s.is_empty() || s.starts_with('-') {
                return None;
            }
            let path = path_from_open_arg(s)?;
            if is_ktx_path(&path) {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

fn path_from_open_arg(s: &str) -> Option<PathBuf> {
    if s.starts_with("file:") {
        return Url::parse(s).ok()?.to_file_path().ok();
    }
    Some(PathBuf::from(s))
}

fn is_ktx_path(path: &Path) -> bool {
    matches!(classify(path), FileKind::Ktx1 | FileKind::Ktx2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_ktx_args_skips_flags_and_images() {
        let paths = ktx_paths_from_open_args([
            "--foo",
            "/tmp/a.png",
            "/tmp/tex.ktx2",
            "/tmp/legacy.ktx",
            "-v",
        ]);
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/tmp/tex.ktx2"),
                PathBuf::from("/tmp/legacy.ktx")
            ]
        );
    }

    #[test]
    fn accepts_file_url() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/basis_lz.ktx2");
        let url = Url::from_file_path(&path).expect("file url");
        let paths = ktx_paths_from_open_args([url.as_str()]);
        assert_eq!(paths, vec![path]);
    }
}
