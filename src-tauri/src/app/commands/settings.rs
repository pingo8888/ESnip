use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use rusqlite::Connection;
use tauri::{AppHandle, State};

use crate::{
    app::{
        platform::set_app_chrome_labels,
        state::{DbState, HotkeyEnabled, HotkeyState},
    },
    store::{
        notes::{open_connection_at_dir, DB_FILE_NAME, DB_SHM_FILE_NAME, DB_WAL_FILE_NAME},
        settings::{current_data_dir, update_data_dir, AppSettings},
    },
};

const DATABASE_FILES: [&str; 3] = [DB_FILE_NAME, DB_WAL_FILE_NAME, DB_SHM_FILE_NAME];

#[tauri::command]
pub(crate) fn get_app_settings(app: AppHandle) -> Result<AppSettings, String> {
    crate::store::settings::get_app_settings(&app)
}

#[tauri::command]
pub(crate) fn update_app_settings(
    app: AppHandle,
    hotkey_state: State<'_, HotkeyState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let settings = crate::store::settings::update_app_settings(&app, settings)?;
    let mut hotkey_guard = hotkey_state.0.lock().map_err(|error| error.to_string())?;
    *hotkey_guard = settings.hotkeys().clone();

    Ok(settings)
}

#[tauri::command]
pub(crate) fn set_hotkeys_enabled(
    hotkey_enabled: State<'_, HotkeyEnabled>,
    enabled: bool,
) -> Result<(), String> {
    hotkey_enabled.set_enabled(enabled);
    Ok(())
}

#[tauri::command]
pub(crate) fn choose_data_dir() -> Result<Option<String>, String> {
    choose_folder()
}

#[tauri::command]
pub(crate) fn migrate_data_dir(
    app: AppHandle,
    state: State<'_, DbState>,
    target_dir: String,
) -> Result<AppSettings, String> {
    let old_dir = current_data_dir(&app)?;
    let new_dir = normalize_target_dir(target_dir)?;

    if same_path(&old_dir, &new_dir) {
        return update_data_dir(&app, &new_dir);
    }

    ensure_target_is_available(&old_dir, &new_dir)?;

    let mut conn = state.conn.lock().map_err(|error| error.to_string())?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|error| error.to_string())?;

    let placeholder = Connection::open_in_memory().map_err(|error| error.to_string())?;
    let old_conn = std::mem::replace(&mut *conn, placeholder);
    drop(old_conn);

    let moved_files = match move_database_files(&old_dir, &new_dir) {
        Ok(files) => files,
        Err(error) => {
            restore_connection(&mut conn, &old_dir);
            return Err(error);
        }
    };

    let new_conn = match open_connection_at_dir(&new_dir) {
        Ok(new_conn) => new_conn,
        Err(error) => {
            rollback_database_files(&moved_files);
            restore_connection(&mut conn, &old_dir);
            return Err(error);
        }
    };

    let next_settings = match update_data_dir(&app, &new_dir) {
        Ok(settings) => settings,
        Err(error) => {
            drop(new_conn);
            rollback_database_files(&moved_files);
            restore_connection(&mut conn, &old_dir);
            return Err(error);
        }
    };

    let placeholder = std::mem::replace(&mut *conn, new_conn);
    drop(placeholder);

    Ok(next_settings)
}

#[tauri::command]
pub(crate) fn reveal_data_dir(app: AppHandle) -> Result<(), String> {
    let data_dir = current_data_dir(&app)?;
    fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
    reveal_folder(&data_dir)
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

fn normalize_target_dir(target_dir: String) -> Result<PathBuf, String> {
    let trimmed = target_dir.trim();

    if trimmed.is_empty() {
        return Err("errors.dataDirEmpty".to_string());
    }

    let path = PathBuf::from(trimmed);
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    path.canonicalize().map_err(|error| error.to_string())
}

fn ensure_target_is_available(old_dir: &Path, new_dir: &Path) -> Result<(), String> {
    for file_name in DATABASE_FILES {
        let source = old_dir.join(file_name);
        let target = new_dir.join(file_name);

        if source.exists() && target.exists() && !same_path(&source, &target) {
            return Err(format!("errors.dataDirTargetExists|{file_name}"));
        }
    }

    Ok(())
}

fn move_database_files(old_dir: &Path, new_dir: &Path) -> Result<Vec<(PathBuf, PathBuf)>, String> {
    fs::create_dir_all(new_dir).map_err(|error| error.to_string())?;

    let mut moved_files = Vec::new();

    for file_name in DATABASE_FILES {
        let source = old_dir.join(file_name);
        let target = new_dir.join(file_name);

        if !source.exists() {
            continue;
        }

        if let Err(error) = move_file(&source, &target) {
            rollback_database_files(&moved_files);
            return Err(error);
        }

        moved_files.push((source, target));
    }

    Ok(moved_files)
}

fn move_file(source: &Path, target: &Path) -> Result<(), String> {
    fs::rename(source, target)
        .or_else(|_| {
            fs::copy(source, target)?;
            fs::remove_file(source)
        })
        .map_err(|error| error.to_string())
}

fn rollback_database_files(moved_files: &[(PathBuf, PathBuf)]) {
    for (source, target) in moved_files.iter().rev() {
        if target.exists() && !source.exists() {
            let _ = move_file(target, source);
        }
    }
}

fn restore_connection(conn: &mut Connection, data_dir: &Path) {
    if let Ok(restored_conn) = open_connection_at_dir(data_dir) {
        let placeholder = std::mem::replace(conn, restored_conn);
        drop(placeholder);
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    let left = left.canonicalize().unwrap_or_else(|_| left.to_path_buf());
    let right = right.canonicalize().unwrap_or_else(|_| right.to_path_buf());

    left == right
}

#[cfg(target_os = "windows")]
fn choose_folder() -> Result<Option<String>, String> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$dialog = New-Object System.Windows.Forms.FolderBrowserDialog
$dialog.Description = 'Select ESnip data folder'
$dialog.ShowNewFolderButton = $true
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
  Write-Output $dialog.SelectedPath
}
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-STA", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|error| error.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let folder = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok((!folder.is_empty()).then_some(folder))
}

#[cfg(not(target_os = "windows"))]
fn choose_folder() -> Result<Option<String>, String> {
    Err("errors.folderSelectionUnsupported".to_string())
}

#[cfg(target_os = "windows")]
fn reveal_folder(path: &Path) -> Result<(), String> {
    Command::new("explorer")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
fn reveal_folder(path: &Path) -> Result<(), String> {
    Command::new("open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn reveal_folder(path: &Path) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}
