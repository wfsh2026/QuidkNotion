use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    let configured_builder = builder
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let quit_item = tauri::menu::MenuItemBuilder::with_id("quit", "退出应用").build(app)?;
            let menu = tauri::menu::MenuBuilder::new(app).items(&[&quit_item]).build()?;
            let default_icon = app.default_window_icon().cloned();
            let mut tray_builder = tauri::tray::TrayIconBuilder::with_id("quicknotion-tray").menu(&menu).tooltip("QuickNotion");
            if let Some(icon) = default_icon {
                tray_builder = tray_builder.icon(icon);
            }
            tray_builder
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        let app_handle = tray.app_handle();
                        toggle_main_window(&app_handle);
                    }
                })
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        let window = app.get_webview_window("main");
                        if let Some(main_window) = window {
                            let _ = main_window.destroy();
                        }
                        app.exit(0);
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        });
    configured_builder.run(tauri::generate_context!()).expect("error while running QuickNotion");
}

fn show_main_window(app: &tauri::AppHandle) {
    let window = app.get_webview_window("main");
    if let Some(main_window) = window {
        let _ = main_window.show();
        let _ = main_window.set_focus();
    }
}

fn toggle_main_window(app: &tauri::AppHandle) {
    let window = app.get_webview_window("main");
    if let Some(main_window) = window {
        let visible = main_window.is_visible().unwrap_or(false);
        if visible {
            let _ = main_window.hide();
        } else {
            show_main_window(app);
        }
    }
}

