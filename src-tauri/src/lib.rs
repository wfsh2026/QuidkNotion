use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    let configured_builder = builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![set_window_always_on_top, hide_main_window, start_window_drag, resize_main_window, toggle_main_window])
        .setup(|app| {
            let settings_item = tauri::menu::MenuItemBuilder::with_id("settings", "设置").build(app)?;
            let quit_item = tauri::menu::MenuItemBuilder::with_id("quit", "退出应用").build(app)?;
            let menu = tauri::menu::MenuBuilder::new(app).items(&[&settings_item, &quit_item]).build()?;
            let icon_bytes = include_bytes!("../icons/tray-icon.png");
            let decoded_icon = image::load_from_memory(icon_bytes).map_err(|error| error.to_string())?;
            let rgba_icon = decoded_icon.to_rgba8();
            let icon_width = rgba_icon.width();
            let icon_height = rgba_icon.height();
            let tray_icon = tauri::image::Image::new_owned(rgba_icon.into_raw(), icon_width, icon_height);
            let tray_builder = tauri::tray::TrayIconBuilder::with_id("quicknotion-tray").icon(tray_icon).menu(&menu).show_menu_on_left_click(false).tooltip("QuickNotion");
            tray_builder
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { button: tauri::tray::MouseButton::Left, .. } = event {
                        let app_handle = tray.app_handle();
                        show_main_window(&app_handle);
                    }
                })
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "settings" => {
                        show_main_window(app);
                        let _ = app.emit("open-settings", ());
                    }
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

#[tauri::command]
fn set_window_always_on_top(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or_else(|| "主窗口不存在".to_string())?;
    window.set_always_on_top(enabled).map_err(|error| error.to_string())
}

#[tauri::command]
fn hide_main_window(app: tauri::AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or_else(|| "主窗口不存在".to_string())?;
    window.hide().map_err(|error| error.to_string())
}

#[tauri::command]
fn start_window_drag(window: tauri::WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|error| error.to_string())
}

#[tauri::command]
fn resize_main_window(window: tauri::WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    let current_size = window.outer_size().map_err(|error| error.to_string())?;
    let requested_width = width.max(320.0).round() as u32;
    let requested_height = height.max(400.0).round() as u32;
    let fallback_width = current_size.width;
    let actual_width = if requested_width == 0 { fallback_width } else { requested_width };
    let size = tauri::PhysicalSize { width: actual_width, height: requested_height };
    window.set_size(tauri::Size::Physical(size)).map_err(|error| error.to_string())
}

#[tauri::command]
fn toggle_main_window(app: tauri::AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or_else(|| "主窗口不存在".to_string())?;
    let visible = window.is_visible().unwrap_or(false);
    if visible {
        window.hide().map_err(|error| error.to_string())
    } else {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())
    }
}

fn show_main_window(app: &tauri::AppHandle) {
    let window = app.get_webview_window("main");
    if let Some(main_window) = window {
        let _ = main_window.show();
        let _ = main_window.set_focus();
    }
}
