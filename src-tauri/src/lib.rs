#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    let configured_builder = builder
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let show_item = tauri::menu::MenuItemBuilder::with_id("show", "显示便签").build(app)?;
            let new_item = tauri::menu::MenuItemBuilder::with_id("new", "新建笔记").build(app)?;
            let pin_item = tauri::menu::MenuItemBuilder::with_id("pin", "切换置顶").build(app)?;
            let quit_item = tauri::menu::MenuItemBuilder::with_id("quit", "退出应用").build(app)?;
            let menu = tauri::menu::MenuBuilder::new(app).items(&[&show_item, &new_item, &pin_item, &quit_item]).build()?;
            let tray_bytes = include_bytes!("../icons/tray-icon.png");
            let tray_icon = tauri::image::Image::from_bytes(tray_bytes)?;
            let tray_builder = tauri::tray::TrayIconBuilder::with_id("quicknotion-tray").icon(tray_icon).menu(&menu).tooltip("QuickNotion");
            tray_builder
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        let app_handle = tray.app_handle();
                        toggle_main_window(&app_handle);
                    }
                })
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => show_main_window(app),
                    "new" => show_main_window(app),
                    "pin" => toggle_pin(app),
                    "quit" => {
                        let window = app.get_webview_window("main");
                        if let Some(main_window) = window {
                            let _ = main_window.destroy();
                        }
                        app.exit(0);
                    }
                    _ => {}
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

fn toggle_pin(app: &tauri::AppHandle) {
    let window = app.get_webview_window("main");
    if let Some(main_window) = window {
        let current = main_window.is_always_on_top().unwrap_or(false);
        let _ = main_window.set_always_on_top(!current);
    }
}

