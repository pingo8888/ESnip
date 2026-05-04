use tauri::Manager;

use crate::{
    app::{
        platform::{setup_tray_icon, start_hotkey_listener},
        state::{DbState, HotkeyState},
    },
    store::{notes::init_connection, settings::get_app_settings},
};

pub(crate) fn setup_app<R: tauri::Runtime>(
    app: &mut tauri::App<R>,
) -> Result<(), Box<dyn std::error::Error>>
where
    tauri::AppHandle<R>: Send + 'static,
{
    let conn = init_connection(app.handle()).map_err(|error| {
        Box::<dyn std::error::Error>::from(std::io::Error::new(std::io::ErrorKind::Other, error))
    })?;

    app.manage(DbState::new(conn));

    let settings = get_app_settings(app.handle()).map_err(|error| {
        Box::<dyn std::error::Error>::from(std::io::Error::new(std::io::ErrorKind::Other, error))
    })?;
    let hotkey_state = app.state::<HotkeyState>();
    let mut hotkey_guard = hotkey_state.0.lock().map_err(|_| {
        Box::<dyn std::error::Error>::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "快捷键状态锁已中毒（poisoned）",
        ))
    })?;
    *hotkey_guard = settings.hotkeys().clone();
    drop(hotkey_guard);

    #[cfg(desktop)]
    start_hotkey_listener(app.handle().clone());
    #[cfg(desktop)]
    setup_tray_icon(app)?;

    Ok(())
}
