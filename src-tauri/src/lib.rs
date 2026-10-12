// Engine modules keep unused helpers from failing clippy -D warnings until the frontend consumes them.
#![allow(dead_code)]

mod commands;
mod drop;
mod error;
mod events;
mod image;
mod ktx;
mod menu;
mod state;
mod updater;
mod window_bounds;
mod workspace;

use state::{AppState, PendingUpdate};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::default())
        .manage(PendingUpdate::default())
        .on_window_event(|window, event| {
            drop::on_window_event(window, event);
            window_bounds::on_window_event(window, event);
        });

    #[cfg(desktop)]
    {
        use tauri_plugin_window_state::{Builder as WindowStateBuilder, StateFlags};

        // Wayland: VISIBLE only — POSITION unsupported; SIZE/MAXIMIZED owned by
        // window_bounds (GTK content size). X11 / macOS / Windows: full plugin flags.
        let flags = if window_bounds::is_wayland() {
            StateFlags::VISIBLE
        } else {
            StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED | StateFlags::VISIBLE
        };

        builder = builder
            .plugin(
                WindowStateBuilder::default()
                    .with_state_flags(flags)
                    .build(),
            )
            .menu(menu::app_menu)
            .on_menu_event(|app, event| {
                menu::on_menu_event(app, &event);
            });
    }

    builder
        .setup(|app| {
            window_bounds::init(app.handle());
            window_bounds::restore_open_windows(app.handle());
            drop::register_drop_handler(app.handle());
            updater::setup_updater(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::extract_concurrency,
            commands::app::ktx_version,
            commands::encoder::default_create_options,
            commands::encoder::describe_options,
            commands::encoder::check_output_compatible,
            commands::encoder::pick_sources,
            commands::encoder::source_thumbnail,
            commands::encoder::source_preview,
            commands::encode::encode,
            commands::encode::cancel_encode,
            commands::encode::encoded_preview,
            commands::encode::save_encoded,
            commands::encode::save_encoded_batch,
            commands::inspector_window::open_inspector,
            commands::inspector_window::inspector_current_file,
            commands::inspect::inspect,
            commands::inspect::inspector_image,
            commands::export::export_images,
            commands::update::check_update,
            commands::update::install_update,
            menu::set_menu_item_enabled,
            menu::dispatch_menu_action,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
