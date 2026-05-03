use tauri::Manager;

use crate::app::state::{AppQuitState, HotkeyShutdown};

const TRAY_SHOW_ID: &str = "tray-show";
const TRAY_QUIT_ID: &str = "tray-quit";

pub(crate) fn show_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Option<()> {
    let window = app.get_webview_window("main")?;

    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();

    Some(())
}

pub(crate) fn handle_window_event<R: tauri::Runtime>(window: &tauri::Window<R>, event: &tauri::WindowEvent) {
    if window.label() != "main" {
        return;
    }

    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        if window.state::<AppQuitState>().is_quitting() {
            return;
        }

        api.prevent_close();
        let _ = window.hide();
    }
}

pub(crate) fn setup_tray_icon<R: tauri::Runtime>(app: &mut tauri::App<R>) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let show_item = MenuItem::with_id(app, TRAY_SHOW_ID, "显示", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, TRAY_QUIT_ID, "退出", true, None::<&str>)?;
    let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    let mut tray_builder = TrayIconBuilder::with_id("main")
        .menu(&tray_menu)
        .tooltip("简摘")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            TRAY_SHOW_ID => {
                let _ = show_main_window(app);
            }
            TRAY_QUIT_ID => {
                app.state::<HotkeyShutdown>().request_shutdown();
                app.state::<AppQuitState>().request_quit();
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(icon);
    }

    let _tray = tray_builder.build(app)?;
    Ok(())
}