use tauri::AppHandle;

use crate::store::settings::AppSettings;

#[tauri::command]
pub(crate) fn get_app_settings(app: AppHandle) -> Result<AppSettings, String> {
    crate::store::settings::get_app_settings(&app)
}

#[tauri::command]
pub(crate) fn update_app_settings(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    crate::store::settings::update_app_settings(&app, settings)
}
