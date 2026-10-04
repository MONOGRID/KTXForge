// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Command id for File → Exit. The handler lives in `on_menu_event`.
#[cfg(desktop)]
const EXIT_MENU_ID: &str = "exit";

#[cfg(desktop)]
fn app_menu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{Menu, MenuItem, Submenu};

    // Predefined quit is unsupported on Linux, so Exit is a normal item on every desktop OS.
    let exit = MenuItem::with_id(app, EXIT_MENU_ID, "E&xit", true, Some("CmdOrCtrl+Q"))?;
    let file = Submenu::with_items(app, "&File", true, &[&exit])?;

    #[cfg(target_os = "macos")]
    {
        let app_menu = macos_app_menu(app)?;
        let edit = macos_edit_menu(app)?;
        return Menu::with_items(app, &[&app_menu, &file, &edit]);
    }

    #[cfg(not(target_os = "macos"))]
    Menu::with_items(app, &[&file])
}

#[cfg(target_os = "macos")]
fn macos_app_menu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Submenu<tauri::Wry>> {
    use tauri::menu::{AboutMetadata, PredefinedMenuItem, Submenu};

    let package = app.package_info();
    let config = app.config();
    let about = AboutMetadata {
        name: Some(package.name.clone()),
        version: Some(package.version.to_string()),
        copyright: config.bundle.copyright.clone(),
        authors: config.bundle.publisher.clone().map(|publisher| vec![publisher]),
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
fn macos_edit_menu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Submenu<tauri::Wry>> {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    // Register before config windows are created. `setup` runs after that, so a
    // plugin registered there misses the first restore.
    #[cfg(desktop)]
    {
        use tauri_plugin_window_state::{Builder as WindowStateBuilder, StateFlags};

        builder = builder
            .plugin(
                WindowStateBuilder::default()
                    .with_state_flags(
                        // VISIBLE is what shows a window created hidden after restore.
                        StateFlags::SIZE
                            | StateFlags::POSITION
                            | StateFlags::MAXIMIZED
                            | StateFlags::VISIBLE,
                    )
                    .build(),
            )
            .menu(app_menu)
            .on_menu_event(|app, event| {
                if event.id() == EXIT_MENU_ID {
                    app.exit(0);
                }
            });
    }

    builder
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
