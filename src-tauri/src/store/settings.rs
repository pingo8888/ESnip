use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

use crate::app::state::SettingsState;

const SETTINGS_FILE_NAME: &str = "settings.json";
const DEFAULT_LOCALE: &str = "zh-CN";
const DEFAULT_SEARCH_ENGINE: &str = "google";
pub(crate) const DEFAULT_TITLE_HOTKEY: &str = "Alt+W";
pub(crate) const DEFAULT_CONTENT_HOTKEY: &str = "Alt+S";
pub(crate) const DEFAULT_PARAGRAPH_HOTKEY: &str = "Alt+P";
const DEFAULT_SAVE_HOTKEY: &str = "Alt+Enter";
const MIN_RESTORED_WINDOW_WIDTH: u32 = 360;
const MIN_RESTORED_WINDOW_HEIGHT: u32 = 320;
const WINDOWS_MINIMIZED_POSITION_SENTINEL: i32 = -30_000;

#[derive(Clone, Debug)]
struct ParsedHotkey {
    ctrl: bool,
    alt: bool,
    key: HotkeyKey,
}

#[derive(Clone, Debug)]
enum HotkeyKey {
    Alphanumeric(char),
    Enter,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSettings {
    #[serde(default = "default_locale")]
    locale: String,
    #[serde(default)]
    clean_bracketed_content_on_capture: bool,
    #[serde(default)]
    data_dir: String,
    #[serde(default)]
    hotkeys: HotkeySettings,
    #[serde(default = "default_search_engine")]
    search_engine: String,
    #[serde(default)]
    window_x: Option<i32>,
    #[serde(default)]
    window_y: Option<i32>,
    #[serde(default)]
    window_width: Option<u32>,
    #[serde(default)]
    window_height: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HotkeySettings {
    #[serde(default = "default_title_hotkey")]
    pub(crate) title: String,
    #[serde(default = "default_content_hotkey")]
    pub(crate) content: String,
    #[serde(default = "default_paragraph_hotkey")]
    pub(crate) paragraph: String,
    #[serde(default = "default_save_hotkey")]
    pub(crate) save: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            locale: default_locale(),
            clean_bracketed_content_on_capture: false,
            data_dir: String::new(),
            hotkeys: HotkeySettings::default(),
            search_engine: default_search_engine(),
            window_x: None,
            window_y: None,
            window_width: None,
            window_height: None,
        }
    }
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            title: default_title_hotkey(),
            content: default_content_hotkey(),
            paragraph: default_paragraph_hotkey(),
            save: default_save_hotkey(),
        }
    }
}

impl AppSettings {
    pub(crate) fn locale(&self) -> &str {
        &self.locale
    }

    pub(crate) fn hotkeys(&self) -> &HotkeySettings {
        &self.hotkeys
    }

    fn apply_user_settings(&mut self, settings: &AppSettings) {
        // The settings page owns user-editable fields only; window state is
        // persisted separately and must survive frontend settings saves.
        self.locale = settings.locale.clone();
        self.clean_bracketed_content_on_capture = settings.clean_bracketed_content_on_capture;
        self.data_dir = settings.data_dir.clone();
        self.hotkeys = settings.hotkeys.clone();
        self.search_engine = settings.search_engine.clone();
    }
}

pub(crate) fn get_app_settings<R: Runtime>(app: &AppHandle<R>) -> Result<AppSettings, String> {
    let state = app.state::<SettingsState>();
    let mut guard = state.0.lock().map_err(|error| error.to_string())?;

    if let Some(settings) = guard.as_ref() {
        return Ok(settings.clone());
    }

    let settings = read_settings_from_disk(app)?;
    *guard = Some(settings.clone());

    Ok(settings)
}

pub(crate) fn update_app_settings<R: Runtime>(
    app: &AppHandle<R>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    mutate_settings_with_lock(app, true, |current| {
        current.apply_user_settings(&settings);
        Ok(())
    })
}

pub(crate) fn current_data_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let settings = get_app_settings(app)?;

    Ok(PathBuf::from(settings.data_dir))
}

pub(crate) fn update_data_dir<R: Runtime>(
    app: &AppHandle<R>,
    data_dir: &Path,
) -> Result<AppSettings, String> {
    mutate_settings_with_lock(app, true, |settings| {
        settings.data_dir = normalize_data_dir(data_dir)?;
        Ok(())
    })
}

pub(crate) fn save_window_state<R: Runtime>(
    app: &AppHandle<R>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    if !is_valid_window_position(x, y) || !is_valid_window_size(width, height) {
        return Ok(());
    }

    mutate_settings_with_lock(app, false, |settings| {
        if settings.window_x == Some(x)
            && settings.window_y == Some(y)
            && settings.window_width == Some(width)
            && settings.window_height == Some(height)
        {
            return Ok(());
        }

        settings.window_x = Some(x);
        settings.window_y = Some(y);
        settings.window_width = Some(width);
        settings.window_height = Some(height);
        Ok(())
    })?;

    Ok(())
}

fn read_settings_from_disk<R: Runtime>(app: &AppHandle<R>) -> Result<AppSettings, String> {
    let settings_path = settings_file_path(app)?;

    if !settings_path.exists() {
        return normalize_settings(app, settings_with_initial_locale(), false);
    }

    let contents = fs::read_to_string(settings_path).map_err(|error| error.to_string())?;
    let mut settings = serde_json::from_str::<AppSettings>(&contents).unwrap_or_default();
    if !settings_json_has_supported_locale(&contents) {
        settings.locale = detect_initial_locale();
    }

    normalize_settings(app, settings, false)
}

fn mutate_settings_with_lock<R, F>(
    app: &AppHandle<R>,
    reject_duplicate_hotkeys: bool,
    mut mutate: F,
) -> Result<AppSettings, String>
where
    R: Runtime,
    F: FnMut(&mut AppSettings) -> Result<(), String>,
{
    let state = app.state::<SettingsState>();
    let mut guard = state.0.lock().map_err(|error| error.to_string())?;
    let mut settings = match guard.as_ref() {
        Some(settings) => settings.clone(),
        None => read_settings_from_disk(app)?,
    };

    mutate(&mut settings)?;
    let settings = normalize_settings(app, settings, reject_duplicate_hotkeys)?;
    write_settings_to_disk(app, &settings)?;
    *guard = Some(settings.clone());

    Ok(settings)
}

fn write_settings_to_disk<R: Runtime>(
    app: &AppHandle<R>,
    settings: &AppSettings,
) -> Result<(), String> {
    let settings_path = settings_file_path(app)?;

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let contents = serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?;
    fs::write(settings_path, contents).map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn apply_saved_window_state<R: Runtime>(app: &AppHandle<R>) {
    let settings = match get_app_settings(app) {
        Ok(s) => s,
        Err(_) => return,
    };

    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    if let (Some(x), Some(y)) = (settings.window_x, settings.window_y) {
        let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
    }
    if let (Some(width), Some(height)) = (settings.window_width, settings.window_height) {
        let _ = window.set_size(tauri::PhysicalSize::new(width, height));
    }
}

pub(crate) fn hotkey_virtual_key(hotkey: &str, default_hotkey: &str) -> u32 {
    parse_hotkey(hotkey)
        .or_else(|| parse_hotkey(default_hotkey))
        .map(|parsed| parsed.key.virtual_key())
        .unwrap_or('W' as u32)
}

pub(crate) fn hotkey_modifier_state(hotkey: &str, default_hotkey: &str) -> (bool, bool) {
    let parsed = parse_hotkey(hotkey)
        .or_else(|| parse_hotkey(default_hotkey))
        .unwrap_or(ParsedHotkey {
            ctrl: false,
            alt: true,
            key: HotkeyKey::Alphanumeric('W'),
        });

    (parsed.ctrl, parsed.alt)
}

fn settings_file_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let app_config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?;

    Ok(app_config_dir.join(SETTINGS_FILE_NAME))
}

fn normalize_settings<R: Runtime>(
    app: &AppHandle<R>,
    settings: AppSettings,
    reject_duplicate_hotkeys: bool,
) -> Result<AppSettings, String> {
    let data_dir = if settings.data_dir.trim().is_empty() {
        default_data_dir(app)?
    } else {
        PathBuf::from(settings.data_dir)
    };

    let hotkeys = normalize_hotkeys(settings.hotkeys);
    if reject_duplicate_hotkeys && has_duplicate_hotkeys(&hotkeys) {
        return Err("errors.duplicateHotkeys".to_string());
    }

    Ok(AppSettings {
        locale: normalize_locale(settings.locale),
        clean_bracketed_content_on_capture: settings.clean_bracketed_content_on_capture,
        data_dir: normalize_data_dir(&data_dir)?,
        hotkeys: if has_duplicate_hotkeys(&hotkeys) {
            HotkeySettings::default()
        } else {
            hotkeys
        },
        search_engine: normalize_search_engine(settings.search_engine),
        window_x: sanitize_window_position(settings.window_x, settings.window_y).map(|(x, _)| x),
        window_y: sanitize_window_position(settings.window_x, settings.window_y).map(|(_, y)| y),
        window_width: sanitize_window_size(settings.window_width, settings.window_height)
            .map(|(width, _)| width),
        window_height: sanitize_window_size(settings.window_width, settings.window_height)
            .map(|(_, height)| height),
    })
}

fn sanitize_window_position(x: Option<i32>, y: Option<i32>) -> Option<(i32, i32)> {
    let (x, y) = (x?, y?);

    is_valid_window_position(x, y).then_some((x, y))
}

fn sanitize_window_size(width: Option<u32>, height: Option<u32>) -> Option<(u32, u32)> {
    let (width, height) = (width?, height?);

    is_valid_window_size(width, height).then_some((width, height))
}

fn is_valid_window_position(x: i32, y: i32) -> bool {
    x > WINDOWS_MINIMIZED_POSITION_SENTINEL && y > WINDOWS_MINIMIZED_POSITION_SENTINEL
}

fn is_valid_window_size(width: u32, height: u32) -> bool {
    width >= MIN_RESTORED_WINDOW_WIDTH && height >= MIN_RESTORED_WINDOW_HEIGHT
}

fn normalize_locale(locale: String) -> String {
    if is_supported_locale(&locale) {
        locale
    } else {
        DEFAULT_LOCALE.to_string()
    }
}

fn normalize_search_engine(search_engine: String) -> String {
    match search_engine.as_str() {
        "google" | "bing" | "baidu" => search_engine,
        _ => DEFAULT_SEARCH_ENGINE.to_string(),
    }
}

fn default_locale() -> String {
    DEFAULT_LOCALE.to_string()
}

fn default_search_engine() -> String {
    DEFAULT_SEARCH_ENGINE.to_string()
}

fn settings_with_initial_locale() -> AppSettings {
    AppSettings {
        locale: detect_initial_locale(),
        ..AppSettings::default()
    }
}

fn settings_json_has_supported_locale(contents: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(contents)
        .ok()
        .and_then(|value| {
            value
                .get("locale")
                .and_then(|locale| locale.as_str())
                .map(is_supported_locale)
        })
        .unwrap_or(false)
}

fn is_supported_locale(locale: &str) -> bool {
    matches!(locale, "zh-CN" | "en-US")
}

fn detect_initial_locale() -> String {
    detect_system_locale().unwrap_or_else(default_locale)
}

#[cfg(windows)]
fn detect_system_locale() -> Option<String> {
    use windows_sys::Win32::Globalization::GetUserDefaultLocaleName;

    let mut buffer = [0u16; 85];
    let written = unsafe { GetUserDefaultLocaleName(buffer.as_mut_ptr(), buffer.len() as i32) };
    if written > 0 {
        let length = buffer
            .iter()
            .position(|code_unit| *code_unit == 0)
            .unwrap_or(written as usize);
        let locale = String::from_utf16_lossy(&buffer[..length]);
        if let Some(locale) = map_system_locale(&locale) {
            return Some(locale);
        }
    }

    detect_env_locale()
}

#[cfg(not(windows))]
fn detect_system_locale() -> Option<String> {
    detect_env_locale()
}

fn detect_env_locale() -> Option<String> {
    ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find_map(|value| map_system_locale(&value))
}

fn map_system_locale(value: &str) -> Option<String> {
    let locale = value
        .trim()
        .split(['.', ':'])
        .next()
        .unwrap_or_default()
        .replace('_', "-")
        .to_ascii_lowercase();

    if locale.is_empty() || matches!(locale.as_str(), "c" | "posix") {
        return None;
    }

    if locale.starts_with("zh") {
        Some("zh-CN".to_string())
    } else {
        Some("en-US".to_string())
    }
}

fn default_title_hotkey() -> String {
    DEFAULT_TITLE_HOTKEY.to_string()
}

fn default_content_hotkey() -> String {
    DEFAULT_CONTENT_HOTKEY.to_string()
}

fn default_paragraph_hotkey() -> String {
    DEFAULT_PARAGRAPH_HOTKEY.to_string()
}

fn default_save_hotkey() -> String {
    DEFAULT_SAVE_HOTKEY.to_string()
}

fn default_data_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|error| error.to_string())
}

fn normalize_data_dir(path: &Path) -> Result<String, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(path)
    };

    Ok(strip_windows_verbatim_prefix(&absolute.to_string_lossy()))
}

fn strip_windows_verbatim_prefix(path: &str) -> String {
    if let Some(path) = path.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{path}");
    }

    path.strip_prefix(r"\\?\").unwrap_or(path).to_string()
}

fn normalize_hotkeys(hotkeys: HotkeySettings) -> HotkeySettings {
    HotkeySettings {
        title: normalize_hotkey(&hotkeys.title, DEFAULT_TITLE_HOTKEY),
        content: normalize_hotkey(&hotkeys.content, DEFAULT_CONTENT_HOTKEY),
        paragraph: normalize_hotkey(&hotkeys.paragraph, DEFAULT_PARAGRAPH_HOTKEY),
        save: normalize_hotkey(&hotkeys.save, DEFAULT_SAVE_HOTKEY),
    }
}

fn normalize_hotkey(input: &str, default_hotkey: &str) -> String {
    parse_hotkey(input)
        .or_else(|| parse_hotkey(default_hotkey))
        .map(format_hotkey)
        .unwrap_or_else(|| default_hotkey.to_string())
}

fn parse_hotkey(input: &str) -> Option<ParsedHotkey> {
    let compact = input.trim().replace(' ', "").to_ascii_uppercase();
    if compact.is_empty() {
        return None;
    }

    let parts = compact.split('+').collect::<Vec<_>>();
    if parts.len() != 2 && parts.len() != 3 {
        return None;
    }

    let key = parse_hotkey_key(parts.last().copied().unwrap_or_default())?;

    let mut ctrl = false;
    let mut alt = false;
    for modifier in &parts[..parts.len() - 1] {
        match *modifier {
            "CTRL" => {
                if ctrl {
                    return None;
                }
                ctrl = true;
            }
            "ALT" => {
                if alt {
                    return None;
                }
                alt = true;
            }
            _ => return None,
        }
    }

    if !((alt && !ctrl) || (ctrl && alt)) {
        return None;
    }

    Some(ParsedHotkey { ctrl, alt, key })
}

fn format_hotkey(parsed: ParsedHotkey) -> String {
    let mut parts = Vec::with_capacity(3);
    if parsed.ctrl {
        parts.push("Ctrl".to_string());
    }
    if parsed.alt {
        parts.push("Alt".to_string());
    }
    parts.push(parsed.key.label());
    parts.join("+")
}

fn has_duplicate_hotkeys(hotkeys: &HotkeySettings) -> bool {
    let values = [
        &hotkeys.title,
        &hotkeys.content,
        &hotkeys.paragraph,
        &hotkeys.save,
    ];

    values.iter().enumerate().any(|(index, value)| {
        values
            .iter()
            .skip(index + 1)
            .any(|other| value.eq_ignore_ascii_case(other))
    })
}

fn parse_hotkey_key(input: &str) -> Option<HotkeyKey> {
    if input.eq_ignore_ascii_case("ENTER") {
        return Some(HotkeyKey::Enter);
    }

    if input.len() != 1 {
        return None;
    }

    let key = input.chars().next()?;
    if key.is_ascii_alphanumeric() {
        Some(HotkeyKey::Alphanumeric(key.to_ascii_uppercase()))
    } else {
        None
    }
}

impl HotkeyKey {
    fn label(&self) -> String {
        match self {
            Self::Alphanumeric(key) => key.to_string(),
            Self::Enter => "Enter".to_string(),
        }
    }

    fn virtual_key(&self) -> u32 {
        match self {
            Self::Alphanumeric(key) => *key as u32,
            Self::Enter => 0x0D,
        }
    }
}

#[cfg(test)]
#[path = "../tests/store/settings.rs"]
mod tests;
