use std::{fs, path::Path};

use rusqlite::Connection;
use tauri::{AppHandle, Runtime};

use crate::store::{notes::schema::init_schema, settings::current_data_dir};

pub(crate) const DB_FILE_NAME: &str = "esnip.sqlite3";
pub(crate) const DB_WAL_FILE_NAME: &str = "esnip.sqlite3-wal";
pub(crate) const DB_SHM_FILE_NAME: &str = "esnip.sqlite3-shm";

pub(crate) fn init_connection<R: Runtime>(app: &AppHandle<R>) -> Result<Connection, String> {
    let data_dir = current_data_dir(app)?;

    open_connection_at_dir(&data_dir)
}

pub(crate) fn open_connection_at_dir(data_dir: &Path) -> Result<Connection, String> {
    fs::create_dir_all(data_dir).map_err(|error| error.to_string())?;

    let db_path = data_dir.join(DB_FILE_NAME);
    let conn = Connection::open(db_path).map_err(|error| error.to_string())?;

    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|error| error.to_string())?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|error| error.to_string())?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| error.to_string())?;
    conn.pragma_update(None, "busy_timeout", 5_000)
        .map_err(|error| error.to_string())?;
    conn.pragma_update(None, "temp_store", "MEMORY")
        .map_err(|error| error.to_string())?;
    conn.pragma_update(None, "mmap_size", 268_435_456_i64)
        .map_err(|error| error.to_string())?;
    init_schema(&conn)?;

    Ok(conn)
}
