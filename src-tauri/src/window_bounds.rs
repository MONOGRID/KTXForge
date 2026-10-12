//! Linux Wayland window size / maximized persistence.
//!
//! Wayland has no global coordinates, so position is not restored (see
//! `is_wayland` + plugin flags in `lib.rs`). Size is GTK's *content* size from
//! `gtk_window_get_size`: logical pixels, excluding client-side titlebar and
//! shadow, i.e. exactly what `gtk_window_resize` (tao `set_size`) takes, so it
//! round-trips. `inner_size()` / `Resized` report the configure size, which on
//! GNOME includes the CSD margins; restoring that grew the window every launch.
//!
//! State is written whenever it changes (not on exit): `tauri dev` restarts
//! kill the process without a clean `RunEvent::Exit`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, LogicalSize, Manager, Runtime, WebviewWindowBuilder, Window, WindowEvent};

const BOUNDS_FILENAME: &str = ".window-size.json";
/// Earlier format stored configure (CSD-inflated) sizes; never read it back.
const LEGACY_FILENAME: &str = ".window-bounds.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct WindowBounds {
    /// Logical GTK content width (unmaximized).
    width: i32,
    /// Logical GTK content height (unmaximized).
    height: i32,
    maximized: bool,
}

struct BoundsState(Mutex<HashMap<String, WindowBounds>>);

/// `true` when this process is on a Wayland session (Linux only).
pub fn is_wayland() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("WAYLAND_DISPLAY").is_some()
            || std::env::var_os("WAYLAND_SOCKET").is_some()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

fn config_file<R: Runtime>(app: &AppHandle<R>, name: &str) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|dir| dir.join(name))
}

fn load_all<R: Runtime>(app: &AppHandle<R>) -> HashMap<String, WindowBounds> {
    config_file(app, BOUNDS_FILENAME)
        .and_then(|path| fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

#[cfg(target_os = "linux")]
fn save_all<R: Runtime>(app: &AppHandle<R>, map: &HashMap<String, WindowBounds>) {
    let Some(path) = config_file(app, BOUNDS_FILENAME) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(bytes) = serde_json::to_vec_pretty(map) {
        let _ = fs::write(path, bytes);
    }
}

pub fn init<R: Runtime>(app: &AppHandle<R>) {
    if !is_wayland() {
        return;
    }
    if let Some(legacy) = config_file(app, LEGACY_FILENAME) {
        let _ = fs::remove_file(legacy);
    }
    app.manage(BoundsState(Mutex::new(load_all(app))));
}

fn saved_bounds<R: Runtime>(app: &AppHandle<R>, label: &str) -> Option<WindowBounds> {
    let state = app.try_state::<BoundsState>()?;
    let map = state.0.lock().ok()?;
    map.get(label).copied()
}

/// Windows declared in `tauri.conf.json`: called from `setup`, before the event
/// loop delivers any `Resized` that could overwrite the saved size.
pub fn restore_open_windows<R: Runtime>(app: &AppHandle<R>) {
    for window in app.webview_windows().values() {
        let Some(bounds) = saved_bounds(app, window.label()) else {
            continue;
        };
        let _ = window.set_size(LogicalSize::new(bounds.width, bounds.height));
        if bounds.maximized {
            let _ = window.maximize();
        }
    }
}

/// Windows built at runtime: bake the saved size into the builder so the first
/// configure is already right (no default-size `Resized` racing the restore).
pub fn with_saved_bounds<'a, R: Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
    app: &AppHandle<R>,
    label: &str,
) -> WebviewWindowBuilder<'a, R, M> {
    match saved_bounds(app, label) {
        Some(bounds) => builder
            .inner_size(f64::from(bounds.width), f64::from(bounds.height))
            .maximized(bounds.maximized),
        None => builder,
    }
}

pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    #[cfg(target_os = "linux")]
    if matches!(event, WindowEvent::Resized(_)) {
        record(window);
    }
    #[cfg(not(target_os = "linux"))]
    let _ = (window, event);
}

/// Runs on the main thread (window events), so GTK calls are allowed.
#[cfg(target_os = "linux")]
fn record<R: Runtime>(window: &Window<R>) {
    use gtk::gdk::WindowState;
    use gtk::prelude::{GtkWindowExt, WidgetExt};

    let app = window.app_handle();
    let Some(state) = app.try_state::<BoundsState>() else {
        return;
    };
    let Ok(gtk_window) = window.gtk_window() else {
        return;
    };
    let Some(flags) = gtk_window.window().map(|gdk| gdk.state()) else {
        return;
    };
    if flags.contains(WindowState::ICONIFIED) {
        return;
    }
    let maximized = flags.contains(WindowState::MAXIMIZED);
    // Maximized / fullscreen / tiled sizes are compositor-imposed: keep the last
    // free-floating size so unmaximize and the next launch land on it.
    let constrained =
        flags.intersects(WindowState::MAXIMIZED | WindowState::FULLSCREEN | WindowState::TILED);

    let Ok(mut map) = state.0.lock() else {
        return;
    };
    let label = window.label();
    let previous = map.get(label).copied();
    let (width, height) = match previous {
        Some(prev) if constrained => (prev.width, prev.height),
        // First run straight into maximized: fall back to the configured default.
        None if constrained => gtk_window.default_size(),
        _ => gtk_window.size(),
    };
    if width <= 0 || height <= 0 {
        return;
    }
    let next = WindowBounds {
        width,
        height,
        maximized,
    };
    if previous != Some(next) {
        map.insert(label.to_owned(), next);
        save_all(app, &map);
    }
}
