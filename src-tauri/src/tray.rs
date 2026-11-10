
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
};

pub fn build_tray() -> SystemTray {
    let toggle = CustomMenuItem::new("toggle".to_string(), "Подключить");
    let show = CustomMenuItem::new("show".to_string(), "Открыть");
    let quit = CustomMenuItem::new("quit".to_string(), "Выход");

    let menu = SystemTrayMenu::new()
        .add_item(toggle)
        .add_item(show)
        .add_item(quit);

    SystemTray::new().with_menu(menu)
}

pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            if let Some(window) = app.get_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "show" => {
                if let Some(window) = app.get_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "toggle" => {
                let _ = app.emit_all("tray-toggle-requested", ());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        },
        _ => {}
    }
}

pub fn set_tray_status(_app: &AppHandle, _status: &str) {
    // Icon switching temporarily disabled.
}
