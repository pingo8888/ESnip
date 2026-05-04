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

pub(crate) fn app_display_name_for_locale(locale: &str) -> &'static str {
    match locale {
        "en-US" => "ESnip",
        _ => "简摘",
    }
}

pub(crate) fn tray_menu_labels_for_locale(locale: &str) -> (&'static str, &'static str) {
    match locale {
        "en-US" => ("Show", "Quit"),
        _ => ("显示", "退出"),
    }
}

pub(crate) fn set_app_chrome_labels<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    title: &str,
    show_label: &str,
    quit_label: &str,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_title(title).map_err(|error| error.to_string())?;
    }

    if let Some(tray) = app.tray_by_id("main") {
        tray.set_tooltip(Some(title)).map_err(|error| error.to_string())?;
        let menu = create_tray_menu(app, show_label, quit_label).map_err(|error| error.to_string())?;
        tray.set_menu(Some(menu)).map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn create_tray_menu<R, M>(manager: &M, show_label: &str, quit_label: &str) -> tauri::Result<tauri::menu::Menu<R>>
where
    R: tauri::Runtime,
    M: Manager<R>,
{
    use tauri::menu::{Menu, MenuItem};

    let show_item = MenuItem::with_id(manager, TRAY_SHOW_ID, show_label, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(manager, TRAY_QUIT_ID, quit_label, true, None::<&str>)?;

    Menu::with_items(manager, &[&show_item, &quit_item])
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
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let locale = crate::store::settings::get_app_settings(app.handle())
        .map(|settings| settings.locale().to_string())
        .unwrap_or_else(|_| "zh-CN".to_string());
    let tooltip = app_display_name_for_locale(&locale);
    let (show_label, quit_label) = tray_menu_labels_for_locale(&locale);
    let tray_menu = create_tray_menu(app, show_label, quit_label)?;

    let mut tray_builder = TrayIconBuilder::with_id("main")
        .menu(&tray_menu)
        .tooltip(tooltip)
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
    let _ = set_app_chrome_labels(app.handle(), tooltip, show_label, quit_label);
    Ok(())
}
