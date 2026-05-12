mod app;
mod core;
mod store;

use crate::app::{
    bootstrap::setup_app,
    commands::{
        choose_data_dir, create_note, delete_note, delete_tag, find_note_by_title,
        get_app_settings, list_note_kind_counts, list_notes_page, list_tags, migrate_data_dir,
        release_hotkeys_disabled, rename_tag, request_hotkeys_disabled, reveal_data_dir,
        search_notes, update_app_chrome_title, update_app_settings, update_note,
    },
    platform::{handle_window_event, show_main_window},
    state::{AppQuitState, HotkeyDisableCount, HotkeyShutdown, HotkeyState, SettingsState},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_main_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppQuitState::default())
        .manage(HotkeyDisableCount::default())
        .manage(HotkeyShutdown::default())
        .manage(HotkeyState::default())
        .manage(SettingsState::default())
        .on_window_event(handle_window_event)
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            list_notes_page,
            search_notes,
            find_note_by_title,
            list_tags,
            list_note_kind_counts,
            rename_tag,
            delete_tag,
            create_note,
            update_note,
            delete_note,
            get_app_settings,
            choose_data_dir,
            migrate_data_dir,
            reveal_data_dir,
            request_hotkeys_disabled,
            release_hotkeys_disabled,
            update_app_chrome_title,
            update_app_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
