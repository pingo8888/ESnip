use serde::Serialize;
use tauri::AppHandle;

#[cfg(target_os = "windows")]
use tauri::{Emitter, Manager};

#[cfg(target_os = "windows")]
use crate::{
    app::{
        platform::tray::show_main_window,
        state::{HotkeyEnabled, HotkeyShutdown, HotkeyState},
    },
    core::text::clean_captured_text,
    store::settings::{
        hotkey_modifier_state, hotkey_virtual_key, HotkeySettings, DEFAULT_CONTENT_HOTKEY,
        DEFAULT_PARAGRAPH_HOTKEY, DEFAULT_TITLE_HOTKEY,
    },
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuickCapturePayload {
    title: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuickCaptureContentPayload {
    content: Option<String>,
    kind: Option<String>,
}

#[cfg(target_os = "windows")]
fn trigger_copy_shortcut() {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        keybd_event, KEYEVENTF_KEYUP, VK_CONTROL, VK_MENU,
    };

    unsafe {
        // The global hotkey includes Alt, so release it before sending Ctrl+C.
        keybd_event(VK_MENU as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_CONTROL as u8, 0, 0, 0);
        keybd_event(b'C', 0, 0, 0);
        keybd_event(b'C', 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
    }
}

#[cfg(target_os = "windows")]
fn capture_selected_text_from_system() -> Option<String> {
    use arboard::Clipboard;
    use std::{
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    use windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber;

    let mut clipboard = Clipboard::new().ok()?;
    let backup_text = clipboard.get_text().ok();
    let marker = format!(
        "__esnip_capture_marker_{}__",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );

    clipboard.set_text(marker.clone()).ok()?;
    let marker_sequence = unsafe { GetClipboardSequenceNumber() };
    thread::sleep(Duration::from_millis(35));
    trigger_copy_shortcut();

    let mut captured = String::new();
    let mut last_sequence = marker_sequence;
    for _ in 0..16 {
        thread::sleep(Duration::from_millis(15));
        let sequence = unsafe { GetClipboardSequenceNumber() };
        if sequence == last_sequence {
            continue;
        }
        last_sequence = sequence;
        if let Ok(text) = clipboard.get_text() {
            if text != marker {
                captured = text;
                break;
            }
        }
    }

    if captured.is_empty() {
        for _ in 0..8 {
            thread::sleep(Duration::from_millis(40));
            if let Ok(text) = clipboard.get_text() {
                if text != marker {
                    captured = text;
                    break;
                }
            }
        }
    }

    let cleaned = clean_captured_text(captured);
    if let Some(text) = backup_text {
        let _ = clipboard.set_text(text);
    } else if cleaned.is_none() {
        let should_clear_marker = clipboard
            .get_text()
            .map(|text| text == marker)
            .unwrap_or(false);
        if should_clear_marker {
            let _ = clipboard.set_text(String::new());
        }
    }

    cleaned
}

#[cfg(target_os = "windows")]
pub(crate) fn start_hotkey_listener<R: tauri::Runtime>(app: AppHandle<R>)
where
    AppHandle<R>: Send + 'static,
{
    let shutdown = app.state::<HotkeyShutdown>().0.clone();
    let hotkey_enabled = app.state::<HotkeyEnabled>().0.clone();
    std::thread::spawn(move || {
        use std::{sync::atomic::Ordering, thread, time::Duration};
        use windows_sys::Win32::UI::{
            Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL},
            WindowsAndMessaging::{PeekMessageW, MSG, PM_NOREMOVE, PM_REMOVE, WM_HOTKEY},
        };

        const HOTKEY_ID_TITLE: i32 = 1104;
        const HOTKEY_ID_CONTENT: i32 = 1105;
        const HOTKEY_ID_PARAGRAPH: i32 = 1106;
        let mut registered = [false; 3];
        let mut current_hotkeys: Option<HotkeySettings> = None;
        let mut was_enabled = false;
        let mut msg: MSG = unsafe { std::mem::zeroed() };

        // Ensure this thread owns a message queue before registering the hotkey.
        let _ = unsafe { PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_NOREMOVE) };

        while !shutdown.load(Ordering::Relaxed) {
            let desired_hotkeys = app
                .state::<HotkeyState>()
                .0
                .lock()
                .map(|hotkeys| hotkeys.clone())
                .unwrap_or_else(|_| {
                    eprintln!("读取快捷键状态失败：状态锁已中毒（poisoned）");
                    HotkeySettings::default()
                });

            if !hotkey_enabled.load(Ordering::Relaxed) {
                unregister_hotkeys(&mut registered);
                was_enabled = false;
            } else if !was_enabled || current_hotkeys.as_ref() != Some(&desired_hotkeys) {
                unregister_hotkeys(&mut registered);
                registered[0] = register_hotkey(
                    HOTKEY_ID_TITLE,
                    &desired_hotkeys.title,
                    DEFAULT_TITLE_HOTKEY,
                );
                registered[1] = register_hotkey(
                    HOTKEY_ID_CONTENT,
                    &desired_hotkeys.content,
                    DEFAULT_CONTENT_HOTKEY,
                );
                registered[2] = register_hotkey(
                    HOTKEY_ID_PARAGRAPH,
                    &desired_hotkeys.paragraph,
                    DEFAULT_PARAGRAPH_HOTKEY,
                );
                current_hotkeys = Some(desired_hotkeys);
                was_enabled = true;
            }

            while unsafe { PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
                if msg.message == WM_HOTKEY {
                    if !hotkey_enabled.load(Ordering::Relaxed) {
                        continue;
                    }
                    if msg.wParam == HOTKEY_ID_TITLE as usize {
                        let title = capture_selected_text_from_system();
                        let _ = show_main_window(&app);
                        if let Err(error) =
                            app.emit_to("main", "quick-capture", QuickCapturePayload { title })
                        {
                            eprintln!("发送取词事件失败: {error}");
                        }
                    } else if msg.wParam == HOTKEY_ID_CONTENT as usize {
                        let content = capture_selected_text_from_system();
                        let _ = show_main_window(&app);
                        if let Err(error) = app.emit_to(
                            "main",
                            "quick-capture-content",
                            QuickCaptureContentPayload {
                                content,
                                kind: Some("sentence".to_string()),
                            },
                        ) {
                            eprintln!("发送取词事件失败: {error}");
                        }
                    } else if msg.wParam == HOTKEY_ID_PARAGRAPH as usize {
                        let content = capture_selected_text_from_system();
                        let _ = show_main_window(&app);
                        if let Err(error) = app.emit_to(
                            "main",
                            "quick-capture-content",
                            QuickCaptureContentPayload {
                                content,
                                kind: Some("paragraph".to_string()),
                            },
                        ) {
                            eprintln!("发送取词事件失败: {error}");
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(20));
        }

        unregister_hotkeys(&mut registered);

        fn register_hotkey(id: i32, hotkey: &str, default_hotkey: &str) -> bool {
            let vk = hotkey_virtual_key(hotkey, default_hotkey);
            let (ctrl, alt) = hotkey_modifier_state(hotkey, default_hotkey);
            let mut modifiers = 0;

            if ctrl {
                modifiers |= MOD_CONTROL;
            }
            if alt {
                modifiers |= MOD_ALT;
            }

            if unsafe { RegisterHotKey(std::ptr::null_mut(), id, modifiers, vk) } == 0 {
                eprintln!("注册全局快捷键 {hotkey} 失败，可能已被其他程序占用");
                false
            } else {
                true
            }
        }

        fn unregister_hotkeys(registered: &mut [bool; 3]) {
            const HOTKEY_IDS: [i32; 3] = [HOTKEY_ID_TITLE, HOTKEY_ID_CONTENT, HOTKEY_ID_PARAGRAPH];

            for (index, id) in HOTKEY_IDS.iter().enumerate() {
                if registered[index] {
                    let _ = unsafe { UnregisterHotKey(std::ptr::null_mut(), *id) };
                    registered[index] = false;
                }
            }
        }
    });
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn start_hotkey_listener<R: tauri::Runtime>(_app: AppHandle<R>) {}
