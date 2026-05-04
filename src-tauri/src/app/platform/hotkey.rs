use serde::Serialize;
use tauri::AppHandle;

#[cfg(target_os = "windows")]
use tauri::{Emitter, Manager};

#[cfg(target_os = "windows")]
use crate::{
    app::{platform::tray::show_main_window, state::HotkeyShutdown},
    core::text::clean_captured_text,
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
    use std::{thread, time::{Duration, SystemTime, UNIX_EPOCH}};
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
    std::thread::spawn(move || {
        use std::{sync::atomic::Ordering, thread, time::Duration};
        use windows_sys::Win32::UI::{
            Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_ALT},
            WindowsAndMessaging::{PeekMessageW, MSG, PM_NOREMOVE, PM_REMOVE, WM_HOTKEY},
        };

        const HOTKEY_ID_TITLE: i32 = 1104;
        const HOTKEY_ID_CONTENT: i32 = 1105;
        const VK_W: u32 = b'W' as u32;
        const VK_S: u32 = b'S' as u32;
        let mut is_title_registered = false;
        let mut is_content_registered = false;
        let mut msg: MSG = unsafe { std::mem::zeroed() };

        // Ensure this thread owns a message queue before registering the hotkey.
        let _ = unsafe { PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_NOREMOVE) };

        if unsafe { RegisterHotKey(std::ptr::null_mut(), HOTKEY_ID_TITLE, MOD_ALT, VK_W) } == 0 {
            eprintln!("注册全局快捷键 Alt+W 失败，可能已被其他程序占用");
        } else {
            is_title_registered = true;
        }

        if unsafe { RegisterHotKey(std::ptr::null_mut(), HOTKEY_ID_CONTENT, MOD_ALT, VK_S) } == 0 {
            eprintln!("注册全局快捷键 Alt+S 失败，可能已被其他程序占用");
        } else {
            is_content_registered = true;
        }

        while !shutdown.load(Ordering::Relaxed) {
            while unsafe { PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
                if msg.message == WM_HOTKEY {
                    if msg.wParam == HOTKEY_ID_TITLE as usize {
                        let title = capture_selected_text_from_system();
                        let _ = show_main_window(&app);
                        if let Err(error) = app.emit_to("main", "quick-capture", QuickCapturePayload { title }) {
                            eprintln!("发送取词事件失败: {error}");
                        }
                    } else if msg.wParam == HOTKEY_ID_CONTENT as usize {
                        let content = capture_selected_text_from_system();
                        let _ = show_main_window(&app);
                        if let Err(error) = app.emit_to("main", "quick-capture-content", QuickCaptureContentPayload { content }) {
                            eprintln!("发送取词事件失败: {error}");
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(20));
        }

        if is_title_registered {
            let _ = unsafe { UnregisterHotKey(std::ptr::null_mut(), HOTKEY_ID_TITLE) };
        }
        if is_content_registered {
            let _ = unsafe { UnregisterHotKey(std::ptr::null_mut(), HOTKEY_ID_CONTENT) };
        }
    });
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn start_hotkey_listener<R: tauri::Runtime>(_app: AppHandle<R>) {}