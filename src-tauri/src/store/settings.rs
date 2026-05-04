use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

const SETTINGS_FILE_NAME: &str = "settings.json";
const DEFAULT_LOCALE: &str = "zh-CN";
pub(crate) const DEFAULT_TITLE_HOTKEY: &str = "Alt+W";
pub(crate) const DEFAULT_CONTENT_HOTKEY: &str = "Alt+S";
pub(crate) const DEFAULT_PARAGRAPH_HOTKEY: &str = "Alt+P";

#[derive(Clone, Copy, Debug)]
struct ParsedHotkey {
    ctrl: bool,
    alt: bool,
    key: char,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSettings {
    #[serde(default = "default_locale")]
    locale: String,
    #[serde(default)]
    data_dir: String,
    #[serde(default)]
    hotkeys: HotkeySettings,
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
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            locale: default_locale(),
            data_dir: String::new(),
            hotkeys: HotkeySettings::default(),
        }
    }
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            title: default_title_hotkey(),
            content: default_content_hotkey(),
            paragraph: default_paragraph_hotkey(),
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
}

pub(crate) fn get_app_settings<R: Runtime>(app: &AppHandle<R>) -> Result<AppSettings, String> {
    let settings_path = settings_file_path(app)?;

    if !settings_path.exists() {
        return normalize_settings(app, AppSettings::default(), false);
    }

    let contents = fs::read_to_string(settings_path).map_err(|error| error.to_string())?;
    let settings = serde_json::from_str::<AppSettings>(&contents).unwrap_or_default();

    normalize_settings(app, settings, false)
}

pub(crate) fn update_app_settings<R: Runtime>(
    app: &AppHandle<R>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let settings_path = settings_file_path(app)?;
    let settings = normalize_settings(app, settings, true)?;

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let contents = serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?;
    fs::write(settings_path, contents).map_err(|error| error.to_string())?;

    Ok(settings)
}

pub(crate) fn current_data_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let settings = get_app_settings(app)?;

    Ok(PathBuf::from(settings.data_dir))
}

pub(crate) fn update_data_dir<R: Runtime>(
    app: &AppHandle<R>,
    data_dir: &Path,
) -> Result<AppSettings, String> {
    let mut settings = get_app_settings(app)?;
    settings.data_dir = normalize_data_dir(data_dir)?;

    update_app_settings(app, settings)
}

pub(crate) fn hotkey_virtual_key(hotkey: &str, default_hotkey: &str) -> u32 {
    parse_hotkey(hotkey)
        .or_else(|| parse_hotkey(default_hotkey))
        .map(|parsed| parsed.key as u32)
        .unwrap_or('W' as u32)
}

pub(crate) fn hotkey_modifier_state(hotkey: &str, default_hotkey: &str) -> (bool, bool) {
    let parsed = parse_hotkey(hotkey)
        .or_else(|| parse_hotkey(default_hotkey))
        .unwrap_or(ParsedHotkey {
            ctrl: false,
            alt: true,
            key: 'W',
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
        return Err("快捷键不能重复".to_string());
    }

    Ok(AppSettings {
        locale: normalize_locale(settings.locale),
        data_dir: normalize_data_dir(&data_dir)?,
        hotkeys: if has_duplicate_hotkeys(&hotkeys) {
            HotkeySettings::default()
        } else {
            hotkeys
        },
    })
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

fn default_title_hotkey() -> String {
    DEFAULT_TITLE_HOTKEY.to_string()
}

fn default_content_hotkey() -> String {
    DEFAULT_CONTENT_HOTKEY.to_string()
}

fn default_paragraph_hotkey() -> String {
    DEFAULT_PARAGRAPH_HOTKEY.to_string()
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

    let key_raw = parts.last().copied().unwrap_or_default();
    if key_raw.len() != 1 {
        return None;
    }

    let key = key_raw.chars().next()?;
    if !key.is_ascii_alphanumeric() {
        return None;
    }

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
    parts.push(parsed.key.to_ascii_uppercase().to_string());
    parts.join("+")
}

fn has_duplicate_hotkeys(hotkeys: &HotkeySettings) -> bool {
    hotkeys.title.eq_ignore_ascii_case(&hotkeys.content)
        || hotkeys.title.eq_ignore_ascii_case(&hotkeys.paragraph)
        || hotkeys.content.eq_ignore_ascii_case(&hotkeys.paragraph)
}
