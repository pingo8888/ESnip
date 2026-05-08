use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::{
    core::text::clean_optional,
    store::notes::{
        kinds::normalize_note_kind,
        tag_index::sync_note_tags,
        types::{NoteDto, NotesCursor, NotesPage, SaveNoteInput, UpdateNoteInput},
    },
};

pub(crate) fn list_notes_page(
    conn: &Connection,
    cursor_updated_at: Option<i64>,
    cursor_id: Option<String>,
    limit: Option<i64>,
) -> Result<NotesPage, String> {
    let page_size = limit.unwrap_or(80).clamp(1, 100);

    let total_count = conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get::<_, i64>(0))
        .map_err(|error| error.to_string())?;

    let notes = if let (Some(updated_at), Some(id)) = (cursor_updated_at, cursor_id) {
        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
                 FROM notes
                 WHERE updated_at < ?1 OR (updated_at = ?1 AND id < ?2)
                 ORDER BY updated_at DESC, id DESC
                 LIMIT ?3",
            )
            .map_err(|error| error.to_string())?;

        let notes = collect_notes(
            stmt.query_map(params![updated_at, id, page_size], map_note_row)
                .map_err(|error| error.to_string())?,
        );

        notes
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
                 FROM notes
                 ORDER BY updated_at DESC, id DESC
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
        updated_at: note.updated_at,
        id: note.id.clone(),
    });

    Ok(NotesPage {
        notes,
        next_cursor,
        total_count,
    })
}

pub(crate) fn find_note_by_title(
    conn: &Connection,
    title: String,
) -> Result<Option<NoteDto>, String> {
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
    let kind = normalize_note_kind(&input.kind);
    let tags_json = serde_json::to_string(&input.tags).map_err(|error| error.to_string())?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;

    tx.execute(
        "INSERT INTO notes (id, title, content, kind, tone, tags_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, title, excerpt, kind, input.tone, tags_json, now, now],
    )
    .map_err(|error| error.to_string())?;
    sync_note_tags(&tx, &id, &input.tags)?;
    tx.commit().map_err(|error| error.to_string())?;

    get_note(conn, &id)
}

pub(crate) fn update_note(conn: &Connection, input: UpdateNoteInput) -> Result<NoteDto, String> {
    let now = now_millis();
    let title = clean_optional(input.title);
    let excerpt = clean_optional(input.excerpt);
    let kind = normalize_note_kind(&input.kind);
    let tags_json = serde_json::to_string(&input.tags).map_err(|error| error.to_string())?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;

    let updated = tx
        .execute(
            "UPDATE notes
             SET title = ?1, content = ?2, kind = ?3, tone = ?4, tags_json = ?5, updated_at = ?6
             WHERE id = ?7",
            params![title, excerpt, kind, input.tone, tags_json, now, input.id],
        )
        .map_err(|error| error.to_string())?;

    if updated == 0 {
        return Err("errors.noteNotFound".to_string());
    }

    sync_note_tags(&tx, &input.id, &input.tags)?;
    tx.commit().map_err(|error| error.to_string())?;

    get_note(conn, &input.id)
}

pub(crate) fn delete_note(conn: &Connection, id: String) -> Result<(), String> {
    conn.execute("DELETE FROM note_tags WHERE note_id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn get_note(conn: &Connection, id: &str) -> Result<NoteDto, String> {
    conn.query_row(
        "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
         FROM notes
         WHERE id = ?1",
        params![id],
        map_note_row,
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "errors.noteNotFound".to_string())
}

pub(super) fn collect_notes<T>(rows: T) -> Result<Vec<NoteDto>, String>
where
    T: Iterator<Item = rusqlite::Result<NoteDto>>,
{
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())
}

pub(super) fn map_note_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteDto> {
    let tags_json: String = row.get(5)?;
    let tags = serde_json::from_str::<Vec<String>>(&tags_json).unwrap_or_default();

    Ok(NoteDto {
        id: row.get(0)?,
        title: row.get(1)?,
        excerpt: row.get(2)?,
        kind: normalize_note_kind(&row.get::<_, String>(3)?).to_string(),
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
