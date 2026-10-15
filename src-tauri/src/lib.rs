mod blocking;
mod commands;
mod dialog;
mod drop;
#[cfg(feature = "e2e")]
pub mod e2e;
mod error;
mod events;
mod image;
mod ktx;
mod menu;
mod os_open;
mod state;
mod updater;
mod window_bounds;
mod workspace;

use commands::encoder_window::PendingEncoderSources;
use state::{AppState, PendingUpdate};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    os_open::note_process_start();

    let mut builder = tauri::Builder::default();

    // single-instance must be registered before other plugins (plugin order).
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let rest = argv.into_iter().skip(1).collect::<Vec<_>>();
            let opening_ktx = !os_open::ktx_paths_from_open_args(&rest).is_empty();
            // Focus encoder only when this activation is not an OS "open KTX".
            if !opening_ktx {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_focus();
                }
            }
            // Warm: leave a visible encoder alone; still drop a hidden zombie main.
            os_open::open_from_args_warm(app, rest);
        }));
    }

    builder = builder
        // Wayland + proprietary NVIDIA: create the GL context before the first
        // frame so GTK never attaches a shared-memory buffer (Error 71).
        // No-op on unaffected systems. Keep early in the plugin chain.
        .plugin(tauri_plugin_wayland_nvidia_quirk::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::default())
        .manage(PendingEncoderSources::default())
        .manage(PendingUpdate::default())
        .on_window_event(|window, event| {
            drop::on_window_event(window, event);
            window_bounds::on_window_event(window, event);
            #[cfg(desktop)]
            menu::on_window_event(window, event);
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

        // Cold KTX argv: do not restore/show or track `main` this session so the
        // encoder never appears and exit does not persist `visible: false`.
        let cold_ktx_argv = os_open::cold_ktx_from_argv();
        let mut window_state = WindowStateBuilder::default().with_state_flags(flags);
        if cold_ktx_argv {
            window_state = window_state.with_filter(|label| label != "main");
        }

        builder = builder
            .plugin(window_state.build())
            .menu(menu::app_menu)
            .on_menu_event(|app, event| {
                menu::on_menu_event(app, &event);
            });
    }

    builder
        .setup(|app| {
            window_bounds::init(app.handle());
            window_bounds::restore_open_windows(app.handle());
            updater::setup_updater(app.handle());
            // Windows / Linux cold start: OS passes the path on argv.
            // Always destroy `main` so Viewer association is inspector-only.
            #[cfg(desktop)]
            os_open::open_from_args_cold(app.handle(), std::env::args().skip(1));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::extract_concurrency,
            commands::app::ktx_version,
            commands::encoder::default_create_options,
            commands::encoder::list_encode_presets,
            commands::encoder::apply_encode_preset,
            commands::encoder::encode_preset_matches,
            commands::encoder::describe_options,
            commands::encoder::input_role_arities,
            commands::encoder::validate_pack_inputs,
            commands::encoder::check_output_compatible,
            commands::encoder::pick_sources,
            commands::encoder::source_thumbnail,
            commands::encoder::source_preview,
            commands::encode::encode,
            commands::encode::cancel_encode,
            commands::encode::discard_outputs,
            commands::encode::encoded_preview,
            commands::encode::save_encoded,
            commands::encode::save_encoded_batch,
            commands::encoder_window::take_pending_encoder_sources,
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
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match event {
            // macOS delivers file opens via Opened URLs (not always argv).
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Opened { urls } => os_open::open_from_urls(app, &urls),
            // The process exits right after this without dropping managed state or
            // pending tasks: stop running `ktx` children and delete the temp workspace.
            tauri::RunEvent::Exit => {
                ktx::sidecar::kill_all();
                app.state::<AppState>().shutdown();
            }
            _ => {}
        });
}
