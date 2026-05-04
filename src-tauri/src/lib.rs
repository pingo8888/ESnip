mod app;
mod core;
mod store;

use crate::app::{
    bootstrap::setup_app,
    commands::{
        create_note, delete_note, find_note_by_title, get_app_settings, list_notes_page, list_tags, search_notes,
        update_app_chrome_title, update_app_settings, update_note,
    },
    platform::handle_window_event,
    state::{AppQuitState, HotkeyShutdown},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppQuitState::default())
        .manage(HotkeyShutdown::default())
        .on_window_event(handle_window_event)
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            list_notes_page,
            search_notes,
            find_note_by_title,
            list_tags,
            create_note,
            update_note,
            delete_note,
            get_app_settings,
            update_app_chrome_title,
            update_app_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
