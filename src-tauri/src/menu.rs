//! Native application menu (File / View / Help, plus macOS app/Edit menus).

use tauri::{AppHandle, Emitter, WebviewWindow};

use crate::commands::{encoder, encoder_window};
use crate::error::AppError;
use crate::events::{DropRejectedPayload, MenuAction, ENCODER_DROP_REJECTED, MENU_ACTION};
use crate::updater;

/// Menu item ids (also used by `set_menu_item_enabled`).
pub mod id {
    pub const OPEN: &str = "open";
    pub const ENCODE: &str = "encode";
    pub const CANCEL_ENCODE: &str = "cancel-encode";
    pub const SAVE: &str = "save";
    pub const CLEAR_SOURCES: &str = "clear-sources";
    pub const EXPORT: &str = "export";
    pub const ZOOM_IN: &str = "zoom-in";
    pub const ZOOM_OUT: &str = "zoom-out";
    pub const ZOOM_ACTUAL: &str = "zoom-actual";
    pub const ZOOM_FIT: &str = "zoom-fit";
    pub const CHECK_UPDATE: &str = "check-update";
    pub const EXIT: &str = "exit";
}

#[cfg(desktop)]
pub fn app_menu(app: &AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

    // Predefined quit is unsupported on Linux, so Exit is a normal item on every desktop OS.
    let open = MenuItem::with_id(app, id::OPEN, "&Open…", true, Some("CmdOrCtrl+O"))?;
    let encode = MenuItem::with_id(app, id::ENCODE, "&Encode", false, Some("CmdOrCtrl+Enter"))?;
    let cancel = MenuItem::with_id(
        app,
        id::CANCEL_ENCODE,
        "&Cancel Encode",
        false,
        Some("CmdOrCtrl+."),
    )?;
    let save = MenuItem::with_id(app, id::SAVE, "&Save…", false, Some("CmdOrCtrl+S"))?;
    let clear = MenuItem::with_id(
        app,
        id::CLEAR_SOURCES,
        "Clear &Sources",
        false,
        None::<&str>,
    )?;
    let export = MenuItem::with_id(
        app,
        id::EXPORT,
        "E&xport…",
        false,
        Some("CmdOrCtrl+Shift+E"),
    )?;
    let exit = MenuItem::with_id(app, id::EXIT, "E&xit", true, Some("CmdOrCtrl+Q"))?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    let file = Submenu::with_items(
        app,
        "&File",
        true,
        &[
            &open, &sep1, &encode, &cancel, &save, &sep2, &clear, &export, &sep3, &exit,
        ],
    )?;

    let zoom_in = MenuItem::with_id(app, id::ZOOM_IN, "Zoom &In", true, Some("CmdOrCtrl+="))?;
    let zoom_out = MenuItem::with_id(app, id::ZOOM_OUT, "Zoom &Out", true, Some("CmdOrCtrl+-"))?;
    let zoom_actual = MenuItem::with_id(
        app,
        id::ZOOM_ACTUAL,
        "&Actual Size",
        true,
        Some("CmdOrCtrl+1"),
    )?;
    let zoom_fit = MenuItem::with_id(app, id::ZOOM_FIT, "&Fit", true, Some("CmdOrCtrl+0"))?;
    let view_sep = PredefinedMenuItem::separator(app)?;
    let view = Submenu::with_items(
        app,
        "&View",
        true,
        &[&zoom_in, &zoom_out, &view_sep, &zoom_actual, &zoom_fit],
    )?;

    let check_update = MenuItem::with_id(
        app,
        id::CHECK_UPDATE,
        "&Check for Updates…",
        true,
        None::<&str>,
    )?;
    let help = Submenu::with_items(app, "&Help", true, &[&check_update])?;

    #[cfg(target_os = "macos")]
    {
        let app_menu = macos_app_menu(app)?;
        let edit = macos_edit_menu(app)?;
        Menu::with_items(app, &[&app_menu, &file, &edit, &view, &help])
    }

    #[cfg(not(target_os = "macos"))]
    Menu::with_items(app, &[&file, &view, &help])
}

/// After a cold OS-open destroys `main`, ensure `window` has a working app menu.
///
/// macOS keeps an app-wide menu bar, so this is a no-op there. On Windows/Linux
/// the menu is per-window. When `main` is gone we always rebuild via
/// [`AppHandle::set_menu`]: muda init errors are swallowed at window creation, so
/// `window.menu()` can look set even when the native bar never attached (or was
/// disposed with `main`).
#[cfg(desktop)]
pub fn ensure_app_menu_on_window(app: &AppHandle, window: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        let _ = (app, window);
    }
    #[cfg(not(target_os = "macos"))]
    {
        use tauri::Manager;

        // Encoder still up: only fill in a missing bar (warm inspector open).
        if app.get_webview_window("main").is_some() && window.menu().is_some() {
            return;
        }
        match app_menu(app) {
            Ok(menu) => {
                if let Err(e) = app.set_menu(menu) {
                    eprintln!("set app menu on window: {e}");
                }
            }
            Err(e) => eprintln!("build app menu: {e}"),
        }
    }
}

#[cfg(desktop)]
pub fn on_menu_event(app: &AppHandle, event: &tauri::menu::MenuEvent) {
    handle_menu_id(app, event.id().as_ref());
}

/// Run the same path as a native menu click / accelerator for `id`.
#[cfg(desktop)]
pub fn handle_menu_id(app: &AppHandle, item: &str) {
    if item == id::EXIT {
        app.exit(0);
        return;
    }
    if item == id::OPEN {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            match encoder::pick_sources(app.clone()).await {
                Ok(sources) if !sources.is_empty() => {
                    if let Err(e) = encoder_window::deliver_sources(&app, sources).await {
                        let _ = app.emit(
                            ENCODER_DROP_REJECTED,
                            DropRejectedPayload {
                                paths: Vec::new(),
                                reason: e.to_string(),
                            },
                        );
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    let _ = app.emit(
                        ENCODER_DROP_REJECTED,
                        DropRejectedPayload {
                            paths: Vec::new(),
                            reason: e.to_string(),
                        },
                    );
                }
            }
        });
        return;
    }
    if item == id::ENCODE {
        emit_action(app, MenuAction::Encode);
        return;
    }
    if item == id::CANCEL_ENCODE {
        emit_action(app, MenuAction::CancelEncode);
        return;
    }
    if item == id::SAVE {
        emit_action(app, MenuAction::Save);
        return;
    }
    if item == id::CLEAR_SOURCES {
        emit_action(app, MenuAction::ClearSources);
        return;
    }
    if item == id::EXPORT {
        emit_action(app, MenuAction::Export);
        return;
    }
    if item == id::ZOOM_IN {
        emit_action(app, MenuAction::ZoomIn);
        return;
    }
    if item == id::ZOOM_OUT {
        emit_action(app, MenuAction::ZoomOut);
        return;
    }
    if item == id::ZOOM_ACTUAL {
        emit_action(app, MenuAction::ZoomActual);
        return;
    }
    if item == id::ZOOM_FIT {
        emit_action(app, MenuAction::ZoomFit);
        return;
    }
    if item == id::CHECK_UPDATE {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let _ = updater::run_update_check(app).await;
        });
    }
}

#[cfg(desktop)]
fn emit_action(app: &AppHandle, action: MenuAction) {
    let _ = app.emit(MENU_ACTION, action);
}

fn find_menu_item(app: &AppHandle, id: &str) -> Option<tauri::menu::MenuItem<tauri::Wry>> {
    let menu = app.menu()?;
    for top in menu.items().unwrap_or_default() {
        let Some(sub) = top.as_submenu() else {
            continue;
        };
        let Some(item) = sub.get(id) else {
            continue;
        };
        if let Some(mi) = item.as_menuitem() {
            return Some(mi.clone());
        }
    }
    None
}

/// Enable or disable a File / Help menu item by id.
#[tauri::command]
pub fn set_menu_item_enabled(app: AppHandle, id: String, enabled: bool) -> Result<(), AppError> {
    let Some(mi) = find_menu_item(&app, id.as_str()) else {
        return Ok(());
    };
    mi.set_enabled(enabled)
        .map_err(|e| AppError::io(format!("set menu enabled: {e}")))?;
    Ok(())
}

/// Windows WebView2 does not deliver native menu accelerators to the app
/// (wry#451). The frontend keydown fallback calls this so enablement and
/// handlers stay identical to a real menu activation.
#[tauri::command]
pub fn dispatch_menu_action(app: AppHandle, id: String) -> Result<(), AppError> {
    let Some(mi) = find_menu_item(&app, id.as_str()) else {
        return Ok(());
    };
    let enabled = mi
        .is_enabled()
        .map_err(|e| AppError::io(format!("menu enabled: {e}")))?;
    if !enabled {
        return Ok(());
    }
    #[cfg(desktop)]
    handle_menu_id(&app, id.as_str());
    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_app_menu(app: &AppHandle) -> tauri::Result<tauri::menu::Submenu<tauri::Wry>> {
    use tauri::menu::{AboutMetadata, PredefinedMenuItem, Submenu};

    let package = app.package_info();
    let config = app.config();
    let about = AboutMetadata {
        name: Some(package.name.clone()),
        version: Some(package.version.to_string()),
        copyright: config.bundle.copyright.clone(),
        authors: config
            .bundle
            .publisher
            .clone()
            .map(|publisher| vec![publisher]),
        ..Default::default()
    };

    Submenu::with_items(
        app,
        package.name.clone(),
        true,
        &[
            &PredefinedMenuItem::about(app, None, Some(about))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::services(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::separator(app)?,
        ],
    )
}

#[cfg(target_os = "macos")]
fn macos_edit_menu(app: &AppHandle) -> tauri::Result<tauri::menu::Submenu<tauri::Wry>> {
    use tauri::menu::{PredefinedMenuItem, Submenu};

    Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )
}
