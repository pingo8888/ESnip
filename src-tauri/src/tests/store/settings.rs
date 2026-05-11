use super::*;

#[test]
fn normalize_hotkey_accepts_supported_modifier_combinations() {
    assert_eq!(normalize_hotkey(" alt + x ", DEFAULT_TITLE_HOTKEY), "Alt+X");
    assert_eq!(
        normalize_hotkey("ctrl + alt + x", DEFAULT_TITLE_HOTKEY),
        "Ctrl+Alt+X"
    );
    assert_eq!(
        normalize_hotkey("ALT+ENTER", DEFAULT_SAVE_HOTKEY),
        "Alt+Enter"
    );
}

#[test]
fn normalize_hotkey_rejects_unsupported_combinations() {
    assert_eq!(normalize_hotkey("Ctrl+X", DEFAULT_TITLE_HOTKEY), "Alt+W");
    assert_eq!(
        normalize_hotkey("Shift+Alt+X", DEFAULT_TITLE_HOTKEY),
        "Alt+W"
    );
    assert_eq!(normalize_hotkey("Alt+Space", DEFAULT_TITLE_HOTKEY), "Alt+W");
}

#[test]
fn duplicate_hotkeys_are_detected_case_insensitively() {
    assert!(has_duplicate_hotkeys(&HotkeySettings {
        title: "Alt+W".to_string(),
        content: "alt+w".to_string(),
        paragraph: "Alt+P".to_string(),
        save: "Alt+Enter".to_string(),
    }));
    assert!(!has_duplicate_hotkeys(&HotkeySettings::default()));
}

#[test]
fn apply_user_settings_preserves_window_state() {
    let mut current = AppSettings {
        locale: "zh-CN".to_string(),
        data_dir: "D:\\old".to_string(),
        hotkeys: HotkeySettings::default(),
        search_engine: "google".to_string(),
        window_x: Some(10),
        window_y: Some(20),
        window_width: Some(800),
        window_height: Some(600),
    };
    let incoming = AppSettings {
        locale: "en-US".to_string(),
        data_dir: "D:\\new".to_string(),
        hotkeys: HotkeySettings {
            title: "Alt+Q".to_string(),
            content: "Ctrl+Alt+C".to_string(),
            paragraph: "Ctrl+Alt+P".to_string(),
            save: "Ctrl+Alt+S".to_string(),
        },
        search_engine: "bing".to_string(),
        window_x: None,
        window_y: None,
        window_width: None,
        window_height: None,
    };

    current.apply_user_settings(&incoming);

    assert_eq!(current.locale, "en-US");
    assert_eq!(current.data_dir, "D:\\new");
    assert_eq!(current.hotkeys.title, "Alt+Q");
    assert_eq!(current.hotkeys.content, "Ctrl+Alt+C");
    assert_eq!(current.hotkeys.paragraph, "Ctrl+Alt+P");
    assert_eq!(current.hotkeys.save, "Ctrl+Alt+S");
    assert_eq!(current.search_engine, "bing");
    assert_eq!(current.window_x, Some(10));
    assert_eq!(current.window_y, Some(20));
    assert_eq!(current.window_width, Some(800));
    assert_eq!(current.window_height, Some(600));
}

#[test]
fn windows_verbatim_path_prefix_is_stripped() {
    assert_eq!(strip_windows_verbatim_prefix(r"\\?\D:\test"), r"D:\test");
    assert_eq!(
        strip_windows_verbatim_prefix(r"\\?\UNC\server\share"),
        r"\\server\share"
    );
    assert_eq!(strip_windows_verbatim_prefix(r"D:\test"), r"D:\test");
}
