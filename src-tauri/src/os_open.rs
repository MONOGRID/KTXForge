//! OS-level "Open with" / double-click handling for `.ktx` / `.ktx2`.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager, Url};

use crate::commands::inspector_window;
use crate::image::classify::{classify, FileKind};
use crate::state::AppState;

/// Open the first KTX path from argv-like strings (executable already skipped by caller).
pub fn open_from_args<I, S>(app: &AppHandle, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    open_ktx_paths(app, ktx_paths_from_open_args(args));
}

/// Open the first KTX path from `RunEvent::Opened` URLs (macOS).
#[cfg(target_os = "macos")]
pub fn open_from_urls(app: &AppHandle, urls: &[Url]) {
    let paths = urls
        .iter()
        .filter_map(|url| url.to_file_path().ok())
        .filter(|path| is_ktx_path(path))
        .collect::<Vec<_>>();
    open_ktx_paths(app, paths);
}

fn open_ktx_paths(app: &AppHandle, paths: Vec<PathBuf>) {
    let Some(path) = paths.into_iter().next() else {
        return;
    };
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        if let Err(e) = inspector_window::open_inspector_path(&app, state.inner(), path).await {
            eprintln!("os open inspector: {e}");
        }
    });
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
