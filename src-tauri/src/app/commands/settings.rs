use tauri::AppHandle;

use crate::{app::platform::set_app_chrome_labels, store::settings::AppSettings};

#[tauri::command]
pub(crate) fn get_app_settings(app: AppHandle) -> Result<AppSettings, String> {
    crate::store::settings::get_app_settings(&app)
}

#[tauri::command]
pub(crate) fn update_app_settings(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    crate::store::settings::update_app_settings(&app, settings)
}

#[tauri::command]
pub(crate) fn update_app_chrome_title(
    app: AppHandle,
    title: String,
    show_label: String,
    quit_label: String,
) -> Result<(), String> {
    set_app_chrome_labels(&app, title.trim(), show_label.trim(), quit_label.trim())
}
