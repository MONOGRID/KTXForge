//! Native application menu (File / Help, plus macOS app/Edit menus).

use tauri::{AppHandle, Emitter};

use crate::commands::encoder;
use crate::error::AppError;
use crate::events::{
    DropRejectedPayload, MenuAction, ENCODER_DROP_REJECTED, ENCODER_SOURCES_ADDED, MENU_ACTION,
};
use crate::updater;

/// Menu item ids (also used by `set_menu_item_enabled`).
pub mod id {
    pub const OPEN: &str = "open";
    pub const ENCODE: &str = "encode";
    pub const CANCEL_ENCODE: &str = "cancel-encode";
    pub const SAVE: &str = "save";
    pub const CLEAR_SOURCES: &str = "clear-sources";
    pub const EXPORT: &str = "export";
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
        return Menu::with_items(app, &[&app_menu, &file, &edit, &help]);
    }

    #[cfg(not(target_os = "macos"))]
    Menu::with_items(app, &[&file, &help])
}

#[cfg(desktop)]
pub fn on_menu_event(app: &AppHandle, event: &tauri::menu::MenuEvent) {
    let item = event.id();
    if item == id::EXIT {
        app.exit(0);
        return;
    }
    if item == id::OPEN {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            match encoder::pick_sources(app.clone()).await {
                Ok(sources) if !sources.is_empty() => {
                    let _ = app.emit(ENCODER_SOURCES_ADDED, sources);
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

/// Enable or disable a File / Help menu item by id.
#[tauri::command]
pub fn set_menu_item_enabled(app: AppHandle, id: String, enabled: bool) -> Result<(), AppError> {
    let Some(menu) = app.menu() else {
        return Ok(());
    };
    for top in menu.items().unwrap_or_default() {
        let Some(sub) = top.as_submenu() else {
            continue;
        };
        let Some(item) = sub.get(id.as_str()) else {
            continue;
        };
        if let Some(mi) = item.as_menuitem() {
            mi.set_enabled(enabled)
                .map_err(|e| AppError::io(format!("set menu enabled: {e}")))?;
            return Ok(());
        }
    }
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
