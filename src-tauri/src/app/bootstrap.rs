use tauri::Manager;

use crate::{
    app::{
        platform::{setup_tray_icon, start_hotkey_listener},
        state::DbState,
    },
    store::notes::init_connection,
};

pub(crate) fn setup_app<R: tauri::Runtime>(
    app: &mut tauri::App<R>,
) -> Result<(), Box<dyn std::error::Error>>
where
    tauri::AppHandle<R>: Send + 'static,
{
    let conn = init_connection(app.handle()).map_err(|error| {
        Box::<dyn std::error::Error>::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            error,
        ))
    })?;

    app.manage(DbState::new(conn));

    #[cfg(desktop)]
    start_hotkey_listener(app.handle().clone());
    #[cfg(desktop)]
    setup_tray_icon(app)?;

    Ok(())
}