use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

const SETTINGS_FILE_NAME: &str = "settings.json";
const DEFAULT_LOCALE: &str = "zh-CN";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSettings {
    #[serde(default = "default_locale")]
    locale: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            locale: default_locale(),
        }
    }
}

pub(crate) fn get_app_settings<R: Runtime>(app: &AppHandle<R>) -> Result<AppSettings, String> {
    let settings_path = settings_file_path(app)?;

    if !settings_path.exists() {
        return Ok(AppSettings::default());
    }

    let contents = fs::read_to_string(settings_path).map_err(|error| error.to_string())?;
    let settings = serde_json::from_str::<AppSettings>(&contents).unwrap_or_default();

    Ok(normalize_settings(settings))
}

pub(crate) fn update_app_settings<R: Runtime>(
    app: &AppHandle<R>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let settings_path = settings_file_path(app)?;
    let settings = normalize_settings(settings);

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let contents = serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?;
    fs::write(settings_path, contents).map_err(|error| error.to_string())?;

    Ok(settings)
}

fn settings_file_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let app_config_dir = app.path().app_config_dir().map_err(|error| error.to_string())?;

    Ok(app_config_dir.join(SETTINGS_FILE_NAME))
}

fn normalize_settings(settings: AppSettings) -> AppSettings {
    AppSettings {
        locale: normalize_locale(settings.locale),
    }
}

fn normalize_locale(locale: String) -> String {
    match locale.as_str() {
        "zh-CN" | "en-US" => locale,
        _ => DEFAULT_LOCALE.to_string(),
    }
}

fn default_locale() -> String {
    DEFAULT_LOCALE.to_string()
}
