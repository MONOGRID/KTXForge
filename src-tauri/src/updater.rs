//! Launch-time update check. Implemented in T4.6.

use tauri::{AppHandle, Runtime};

pub fn setup_updater<R: Runtime>(_app: &AppHandle<R>) {
    // Plugin registration waits for pubkey/endpoints in T6.3.
    // Keep the crate linked so `cargo machete` stays green in the scaffold.
    let _ = tauri_plugin_updater::Builder::new;
}
