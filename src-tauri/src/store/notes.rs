use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};
use uuid::Uuid;

use crate::core::text::{build_fts_query, build_like_pattern, clean_optional};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NoteDto {
    id: String,
    title: Option<String>,
    excerpt: Option<String>,
    kind: String,
    tone: String,
    tags: Vec<String>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NotesPage {
    notes: Vec<NoteDto>,
    next_cursor: Option<NotesCursor>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NotesCursor {
    created_at: i64,
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaveNoteInput {
    title: Option<String>,
    excerpt: Option<String>,
    kind: String,
    tone: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateNoteInput {
    id: String,
    title: Option<String>,
    excerpt: Option<String>,
    kind: String,
    tone: String,
    tags: Vec<String>,
}

pub(crate) fn init_connection<R: Runtime>(app: &AppHandle<R>) -> Result<Connection, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;

    let db_path = app_data_dir.join("esnip.sqlite3");
    let conn = Connection::open(db_path).map_err(|error| error.to_string())?;

    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|error| error.to_string())?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| error.to_string())?;
    init_schema(&conn)?;

    Ok(conn)
}

pub(crate) fn list_notes_page(
    conn: &Connection,
    cursor_created_at: Option<i64>,
    cursor_id: Option<String>,
    limit: Option<i64>,
) -> Result<NotesPage, String> {
    let page_size = limit.unwrap_or(80).clamp(1, 100);

    let notes = if let (Some(created_at), Some(id)) = (cursor_created_at, cursor_id) {
        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
                 FROM notes
                 WHERE created_at < ?1 OR (created_at = ?1 AND id < ?2)
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?3",
            )
            .map_err(|error| error.to_string())?;

        let notes = collect_notes(
            stmt.query_map(params![created_at, id, page_size], map_note_row)
                .map_err(|error| error.to_string())?,
        );

        notes
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
                 FROM notes
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?1",
            )
            .map_err(|error| error.to_string())?;

        let notes = collect_notes(
            stmt.query_map(params![page_size], map_note_row)
                .map_err(|error| error.to_string())?,
        );

        notes
    }?;

    let next_cursor = notes.last().map(|note| NotesCursor {
        created_at: note.created_at,
        id: note.id.clone(),
    });

    Ok(NotesPage { notes, next_cursor })
}

pub(crate) fn search_notes(conn: &Connection, query: String, limit: Option<i64>) -> Result<NotesPage, String> {
    let page_size = limit.unwrap_or(80).clamp(1, 100);
    let Some(fts_query) = build_fts_query(&query) else {
        return Ok(NotesPage {
            notes: Vec::new(),
            next_cursor: None,
        });
    };

    if query.trim().chars().count() < 3 {
        return search_notes_like(conn, &query, page_size);
    }

    let mut stmt = conn
        .prepare(
            "SELECT notes.id, notes.title, notes.content, notes.kind, notes.tone, notes.tags_json, notes.created_at, notes.updated_at
             FROM notes_fts
             JOIN notes ON notes_fts.rowid = notes.rowid
             WHERE notes_fts MATCH ?1
             ORDER BY bm25(notes_fts), notes.created_at DESC, notes.id DESC
             LIMIT ?2",
        )
        .map_err(|error| error.to_string())?;

    let notes = collect_notes(
        stmt.query_map(params![fts_query, page_size], map_note_row)
            .map_err(|error| error.to_string())?,
    )?;

    Ok(NotesPage {
        notes,
        next_cursor: None,
    })
}

pub(crate) fn find_note_by_title(conn: &Connection, title: String) -> Result<Option<NoteDto>, String> {
    let Some(clean_title) = clean_optional(Some(title)) else {
        return Ok(None);
    };

    conn.query_row(
        "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
         FROM notes
         WHERE title = ?1 COLLATE NOCASE
         ORDER BY updated_at DESC, id DESC
         LIMIT 1",
        params![clean_title],
        map_note_row,
    )
    .optional()
    .map_err(|error| error.to_string())
}

pub(crate) fn create_note(conn: &Connection, input: SaveNoteInput) -> Result<NoteDto, String> {
    let now = now_millis();
    let id = Uuid::new_v4().to_string();
    let title = clean_optional(input.title);
    let excerpt = clean_optional(input.excerpt);
    let tags_json = serde_json::to_string(&input.tags).map_err(|error| error.to_string())?;

    conn.execute(
        "INSERT INTO notes (id, title, content, kind, tone, tags_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, title, excerpt, input.kind, input.tone, tags_json, now, now],
    )
    .map_err(|error| error.to_string())?;

    get_note(conn, &id)
}

pub(crate) fn update_note(conn: &Connection, input: UpdateNoteInput) -> Result<NoteDto, String> {
    let now = now_millis();
    let title = clean_optional(input.title);
    let excerpt = clean_optional(input.excerpt);
    let tags_json = serde_json::to_string(&input.tags).map_err(|error| error.to_string())?;

    let updated = conn
        .execute(
            "UPDATE notes
             SET title = ?1, content = ?2, kind = ?3, tone = ?4, tags_json = ?5, updated_at = ?6
             WHERE id = ?7",
            params![title, excerpt, input.kind, input.tone, tags_json, now, input.id],
        )
        .map_err(|error| error.to_string())?;

    if updated == 0 {
        return Err("Note not found".to_string());
    }

    get_note(conn, &input.id)
}

pub(crate) fn delete_note(conn: &Connection, id: String) -> Result<(), String> {
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn search_notes_like(conn: &Connection, query: &str, page_size: i64) -> Result<NotesPage, String> {
    let like_pattern = build_like_pattern(query);
    let mut stmt = conn
        .prepare(
            "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
             FROM notes
             WHERE COALESCE(title, '') LIKE ?1 ESCAPE '\\'
                OR COALESCE(content, '') LIKE ?1 ESCAPE '\\'
             ORDER BY created_at DESC, id DESC
             LIMIT ?2",
        )
        .map_err(|error| error.to_string())?;

    let notes = collect_notes(
        stmt.query_map(params![like_pattern, page_size], map_note_row)
            .map_err(|error| error.to_string())?,
    )?;

    Ok(NotesPage {
        notes,
        next_cursor: None,
    })
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    migrate_fts_table(conn)?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT,
            content TEXT,
            kind TEXT NOT NULL CHECK (kind IN ('词语', '句子', '段落')),
            tone TEXT NOT NULL CHECK (tone IN ('sage', 'ochre', 'clay', 'ink')),
            tags_json TEXT NOT NULL DEFAULT '[]',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            source TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_notes_created_at_id
            ON notes (created_at DESC, id DESC);
        CREATE INDEX IF NOT EXISTS idx_notes_updated_at_id
            ON notes (updated_at DESC, id DESC);
        CREATE INDEX IF NOT EXISTS idx_notes_kind
            ON notes (kind);
        CREATE INDEX IF NOT EXISTS idx_notes_title
            ON notes (title COLLATE NOCASE);

        CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            title,
            content,
            content='notes',
            content_rowid='rowid',
            tokenize='trigram'
        );

        CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
            INSERT INTO notes_fts(rowid, title, content)
            VALUES (new.rowid, new.title, new.content);
        END;

        CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, title, content)
            VALUES ('delete', old.rowid, old.title, old.content);
        END;

        CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, title, content)
            VALUES ('delete', old.rowid, old.title, old.content);
            INSERT INTO notes_fts(rowid, title, content)
            VALUES (new.rowid, new.title, new.content);
        END;
        ",
    )
    .map_err(|error| error.to_string())?;

    rebuild_fts_index(conn)?;

    Ok(())
}

fn migrate_fts_table(conn: &Connection) -> Result<(), String> {
    let current_sql = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'notes_fts'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    if current_sql
        .as_deref()
        .is_some_and(|sql| !sql.to_lowercase().contains("tokenize='trigram'"))
    {
        conn.execute_batch(
            "
            DROP TRIGGER IF EXISTS notes_ai;
            DROP TRIGGER IF EXISTS notes_ad;
            DROP TRIGGER IF EXISTS notes_au;
            DROP TABLE IF EXISTS notes_fts;
            ",
        )
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn rebuild_fts_index(conn: &Connection) -> Result<(), String> {
    conn.execute("INSERT INTO notes_fts(notes_fts) VALUES('rebuild')", [])
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn get_note(conn: &Connection, id: &str) -> Result<NoteDto, String> {
    conn.query_row(
        "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
         FROM notes
         WHERE id = ?1",
        params![id],
        map_note_row,
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "Note not found".to_string())
}

fn collect_notes<T>(rows: T) -> Result<Vec<NoteDto>, String>
where
    T: Iterator<Item = rusqlite::Result<NoteDto>>,
{
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())
}

fn map_note_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteDto> {
    let tags_json: String = row.get(5)?;
    let tags = serde_json::from_str::<Vec<String>>(&tags_json).unwrap_or_default();

    Ok(NoteDto {
        id: row.get(0)?,
        title: row.get(1)?,
        excerpt: row.get(2)?,
        kind: row.get(3)?,
        tone: row.get(4)?,
        tags,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}
