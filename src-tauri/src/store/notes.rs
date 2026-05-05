use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, params_from_iter, types::Value, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use uuid::Uuid;

use crate::core::text::{build_fts_query, build_like_pattern, clean_optional};
use crate::store::settings::current_data_dir;

pub(crate) const DB_FILE_NAME: &str = "esnip.sqlite3";
pub(crate) const DB_WAL_FILE_NAME: &str = "esnip.sqlite3-wal";
pub(crate) const DB_SHM_FILE_NAME: &str = "esnip.sqlite3-shm";

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
    total_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NotesCursor {
    updated_at: i64,
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
    let data_dir = current_data_dir(app)?;

    open_connection_at_dir(&data_dir)
}

pub(crate) fn open_connection_at_dir(data_dir: &Path) -> Result<Connection, String> {
    fs::create_dir_all(data_dir).map_err(|error| error.to_string())?;

    let db_path = data_dir.join(DB_FILE_NAME);
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

pub(crate) fn search_notes(
    conn: &Connection,
    query: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<NotesPage, String> {
    let page_size = limit.unwrap_or(80).clamp(1, 100);
    let page_offset = offset.unwrap_or(0).max(0);
    let parsed_query = parse_search_query(&query);

    if parsed_query.text.is_empty()
        && (!parsed_query.included_tags.is_empty() || !parsed_query.excluded_tags.is_empty())
    {
        return search_notes_by_tags(
            conn,
            &parsed_query.included_tags,
            &parsed_query.excluded_tags,
            page_size,
            page_offset,
        );
    }

    let Some(fts_query) = build_fts_query(&parsed_query.text) else {
        return Ok(NotesPage {
            notes: Vec::new(),
            next_cursor: None,
            total_count: 0,
        });
    };

    if parsed_query.text.trim().chars().count() < 3 {
        return search_notes_like(
            conn,
            &parsed_query.text,
            &parsed_query.included_tags,
            &parsed_query.excluded_tags,
            page_size,
            page_offset,
        );
    }

    let tag_filter_sql = build_tag_filter_sql(
        "notes",
        parsed_query.included_tags.len(),
        parsed_query.excluded_tags.len(),
        2,
    );
    let tag_param_count = parsed_query.included_tags.len() + parsed_query.excluded_tags.len();
    let mut values = Vec::with_capacity(3 + tag_param_count);
    values.push(Value::Text(fts_query));
    values.extend(parsed_query.included_tags.iter().cloned().map(Value::Text));
    values.extend(parsed_query.excluded_tags.iter().cloned().map(Value::Text));
    let total_count = count_notes_fts(conn, &tag_filter_sql, &values)?;
    values.push(Value::Integer(page_size));
    values.push(Value::Integer(page_offset));

    let mut stmt = conn
        .prepare(
            &format!(
                "SELECT notes.id, notes.title, notes.content, notes.kind, notes.tone, notes.tags_json, notes.created_at, notes.updated_at
             FROM notes_fts
             JOIN notes ON notes_fts.rowid = notes.rowid
             WHERE notes_fts MATCH ?1
               {tag_filter_sql}
             ORDER BY bm25(notes_fts), notes.updated_at DESC, notes.id DESC
             LIMIT ?{} OFFSET ?{}",
                tag_param_count + 2,
                tag_param_count + 3
            ),
        )
        .map_err(|error| error.to_string())?;

    let notes = collect_notes(
        stmt.query_map(params_from_iter(values), map_note_row)
            .map_err(|error| error.to_string())?,
    )?;

    Ok(NotesPage {
        notes,
        next_cursor: None,
        total_count,
    })
}

pub(crate) fn list_tags(
    conn: &Connection,
    prefix: String,
    limit: Option<i64>,
) -> Result<Vec<String>, String> {
    let cleaned_prefix = prefix.trim().trim_start_matches('#').trim();
    let like_pattern = build_like_pattern(cleaned_prefix);

    if let Some(page_size) = limit {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT json_each.value
                 FROM notes, json_each(notes.tags_json)
                 WHERE json_each.type = 'text'
                   AND (?1 = '' OR json_each.value LIKE ?2 ESCAPE '\\')
                 ORDER BY json_each.value COLLATE NOCASE
                 LIMIT ?3",
            )
            .map_err(|error| error.to_string())?;

        let tags = stmt
            .query_map(params![cleaned_prefix, like_pattern, page_size], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string());

        tags
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT json_each.value
                 FROM notes, json_each(notes.tags_json)
                 WHERE json_each.type = 'text'
                   AND (?1 = '' OR json_each.value LIKE ?2 ESCAPE '\\')
                 ORDER BY json_each.value COLLATE NOCASE",
            )
            .map_err(|error| error.to_string())?;

        let tags = stmt
            .query_map(params![cleaned_prefix, like_pattern], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string());

        tags
    }
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

    conn.execute(
        "INSERT INTO notes (id, title, content, kind, tone, tags_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, title, excerpt, kind, input.tone, tags_json, now, now],
    )
    .map_err(|error| error.to_string())?;

    get_note(conn, &id)
}

pub(crate) fn update_note(conn: &Connection, input: UpdateNoteInput) -> Result<NoteDto, String> {
    let now = now_millis();
    let title = clean_optional(input.title);
    let excerpt = clean_optional(input.excerpt);
    let kind = normalize_note_kind(&input.kind);
    let tags_json = serde_json::to_string(&input.tags).map_err(|error| error.to_string())?;

    let updated = conn
        .execute(
            "UPDATE notes
             SET title = ?1, content = ?2, kind = ?3, tone = ?4, tags_json = ?5, updated_at = ?6
             WHERE id = ?7",
            params![title, excerpt, kind, input.tone, tags_json, now, input.id],
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

fn search_notes_like(
    conn: &Connection,
    query: &str,
    included_tags: &[String],
    excluded_tags: &[String],
    page_size: i64,
    page_offset: i64,
) -> Result<NotesPage, String> {
    let like_pattern = build_like_pattern(query);
    let tag_filter_sql =
        build_tag_filter_sql("notes", included_tags.len(), excluded_tags.len(), 2);
    let tag_param_count = included_tags.len() + excluded_tags.len();
    let mut values = Vec::with_capacity(3 + tag_param_count);
    values.push(Value::Text(like_pattern));
    values.extend(included_tags.iter().cloned().map(Value::Text));
    values.extend(excluded_tags.iter().cloned().map(Value::Text));
    let total_count = count_notes_like(conn, &tag_filter_sql, &values)?;
    values.push(Value::Integer(page_size));
    values.push(Value::Integer(page_offset));
    let mut stmt = conn
        .prepare(&format!(
            "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
             FROM notes
             WHERE (COALESCE(title, '') LIKE ?1 ESCAPE '\\'
                OR COALESCE(content, '') LIKE ?1 ESCAPE '\\')
               {tag_filter_sql}
             ORDER BY updated_at DESC, id DESC
             LIMIT ?{} OFFSET ?{}",
            tag_param_count + 2,
            tag_param_count + 3
        ))
        .map_err(|error| error.to_string())?;

    let notes = collect_notes(
        stmt.query_map(params_from_iter(values), map_note_row)
            .map_err(|error| error.to_string())?,
    )?;

    Ok(NotesPage {
        notes,
        next_cursor: None,
        total_count,
    })
}

fn search_notes_by_tags(
    conn: &Connection,
    included_tags: &[String],
    excluded_tags: &[String],
    page_size: i64,
    page_offset: i64,
) -> Result<NotesPage, String> {
    let tag_filter_sql =
        build_tag_filter_sql("notes", included_tags.len(), excluded_tags.len(), 1);
    let tag_param_count = included_tags.len() + excluded_tags.len();
    let mut values = included_tags
        .iter()
        .chain(excluded_tags.iter())
        .cloned()
        .map(Value::Text)
        .collect::<Vec<_>>();
    let total_count = count_notes_by_tags(conn, &tag_filter_sql, &values)?;
    values.push(Value::Integer(page_size));
    values.push(Value::Integer(page_offset));

    let mut stmt = conn
        .prepare(&format!(
            "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
                 FROM notes
                 WHERE 1 = 1
                   {tag_filter_sql}
                 ORDER BY updated_at DESC, id DESC
                 LIMIT ?{} OFFSET ?{}",
            tag_param_count + 1,
            tag_param_count + 2
        ))
        .map_err(|error| error.to_string())?;

    let notes = collect_notes(
        stmt.query_map(params_from_iter(values), map_note_row)
            .map_err(|error| error.to_string())?,
    )?;

    Ok(NotesPage {
        notes,
        next_cursor: None,
        total_count,
    })
}

fn count_notes_fts(
    conn: &Connection,
    tag_filter_sql: &str,
    values: &[Value],
) -> Result<i64, String> {
    conn.query_row(
        &format!(
            "SELECT COUNT(*)
             FROM notes_fts
             JOIN notes ON notes_fts.rowid = notes.rowid
             WHERE notes_fts MATCH ?1
               {tag_filter_sql}"
        ),
        params_from_iter(values.iter()),
        |row| row.get::<_, i64>(0),
    )
    .map_err(|error| error.to_string())
}

fn count_notes_like(
    conn: &Connection,
    tag_filter_sql: &str,
    values: &[Value],
) -> Result<i64, String> {
    conn.query_row(
        &format!(
            "SELECT COUNT(*)
             FROM notes
             WHERE (COALESCE(title, '') LIKE ?1 ESCAPE '\\'
                OR COALESCE(content, '') LIKE ?1 ESCAPE '\\')
               {tag_filter_sql}"
        ),
        params_from_iter(values.iter()),
        |row| row.get::<_, i64>(0),
    )
    .map_err(|error| error.to_string())
}

fn count_notes_by_tags(
    conn: &Connection,
    tag_filter_sql: &str,
    values: &[Value],
) -> Result<i64, String> {
    conn.query_row(
        &format!(
            "SELECT COUNT(*)
             FROM notes
             WHERE 1 = 1
               {tag_filter_sql}"
        ),
        params_from_iter(values.iter()),
        |row| row.get::<_, i64>(0),
    )
    .map_err(|error| error.to_string())
}

fn build_tag_filter_sql(
    table_alias: &str,
    included_tag_count: usize,
    excluded_tag_count: usize,
    first_param_index: usize,
) -> String {
    let included_sql = (0..included_tag_count)
        .map(|index| {
            format!(
                " AND EXISTS (SELECT 1 FROM json_each({table_alias}.tags_json) WHERE json_each.value = ?{} COLLATE NOCASE)",
                first_param_index + index
            )
        })
        .collect::<String>();
    let excluded_sql = (0..excluded_tag_count)
        .map(|index| {
            format!(
                " AND NOT EXISTS (SELECT 1 FROM json_each({table_alias}.tags_json) WHERE json_each.value = ?{} COLLATE NOCASE)",
                first_param_index + included_tag_count + index
            )
        })
        .collect::<String>();

    format!("{included_sql}{excluded_sql}")
}

struct ParsedSearchQuery {
    text: String,
    included_tags: Vec<String>,
    excluded_tags: Vec<String>,
}

fn parse_search_query(query: &str) -> ParsedSearchQuery {
    let mut text_terms = Vec::new();
    let mut included_tags = Vec::new();
    let mut excluded_tags = Vec::new();

    for term in query.split_whitespace() {
        if let Some(tag) = term.strip_prefix("!#").and_then(clean_search_tag) {
            remove_tag(&mut included_tags, &tag);
            push_unique_tag(&mut excluded_tags, tag);
        } else if let Some(tag) = term.strip_prefix('#').and_then(clean_search_tag) {
            if !excluded_tags
                .iter()
                .any(|item: &String| item.eq_ignore_ascii_case(&tag))
            {
                push_unique_tag(&mut included_tags, tag);
            }
        } else {
            text_terms.push(term);
        }
    }

    ParsedSearchQuery {
        text: text_terms.join(" "),
        included_tags,
        excluded_tags,
    }
}

fn push_unique_tag(tags: &mut Vec<String>, tag: String) {
    if !tags.iter().any(|item| item.eq_ignore_ascii_case(&tag)) {
        tags.push(tag);
    }
}

fn remove_tag(tags: &mut Vec<String>, tag: &str) {
    tags.retain(|item| !item.eq_ignore_ascii_case(tag));
}

fn clean_search_tag(value: &str) -> Option<String> {
    let tag = value
        .trim()
        .trim_matches(|ch: char| ch == ',' || ch == '，')
        .trim();

    if tag.is_empty() {
        None
    } else {
        Some(tag.to_string())
    }
}

fn normalize_note_kind(kind: &str) -> &'static str {
    match kind {
        "sentence" | "句子" => "sentence",
        "paragraph" | "段落" => "paragraph",
        "word" | "词语" => "word",
        _ => "word",
    }
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    migrate_note_kind_values(conn)?;
    migrate_fts_table(conn)?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT,
            content TEXT,
            kind TEXT NOT NULL CHECK (kind IN ('word', 'sentence', 'paragraph')),
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

fn migrate_note_kind_values(conn: &Connection) -> Result<(), String> {
    if !notes_table_exists(conn)? {
        return Ok(());
    }

    if note_kind_check_uses_chinese(conn)? {
        rebuild_notes_table_with_english_kinds(conn)?;
    } else {
        normalize_existing_note_kind_values(conn)?;
    }

    Ok(())
}

fn notes_table_exists(conn: &Connection) -> Result<bool, String> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'notes')",
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value != 0)
    .map_err(|error| error.to_string())
}

fn note_kind_check_uses_chinese(conn: &Connection) -> Result<bool, String> {
    let sql = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'notes'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .unwrap_or_default();

    Ok(sql.contains("'词语'") || sql.contains("'句子'") || sql.contains("'段落'"))
}

fn rebuild_notes_table_with_english_kinds(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        DROP TRIGGER IF EXISTS notes_ai;
        DROP TRIGGER IF EXISTS notes_ad;
        DROP TRIGGER IF EXISTS notes_au;
        DROP TABLE IF EXISTS notes_fts;
        DROP TABLE IF EXISTS notes_legacy_kind;
        ALTER TABLE notes RENAME TO notes_legacy_kind;

        CREATE TABLE notes (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT,
            content TEXT,
            kind TEXT NOT NULL CHECK (kind IN ('word', 'sentence', 'paragraph')),
            tone TEXT NOT NULL CHECK (tone IN ('sage', 'ochre', 'clay', 'ink')),
            tags_json TEXT NOT NULL DEFAULT '[]',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            source TEXT
        );

        INSERT INTO notes (id, title, content, kind, tone, tags_json, created_at, updated_at, source)
        SELECT id,
               title,
               content,
               CASE kind
                   WHEN '词语' THEN 'word'
                   WHEN '句子' THEN 'sentence'
                   WHEN '段落' THEN 'paragraph'
                   WHEN 'word' THEN 'word'
                   WHEN 'sentence' THEN 'sentence'
                   WHEN 'paragraph' THEN 'paragraph'
                   ELSE 'word'
               END,
               tone,
               tags_json,
               created_at,
               updated_at,
               source
          FROM notes_legacy_kind;

        DROP TABLE notes_legacy_kind;
        ",
    )
    .map_err(|error| error.to_string())
}

fn normalize_existing_note_kind_values(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        UPDATE notes
           SET kind = CASE kind
               WHEN '词语' THEN 'word'
               WHEN '句子' THEN 'sentence'
               WHEN '段落' THEN 'paragraph'
               WHEN 'word' THEN 'word'
               WHEN 'sentence' THEN 'sentence'
               WHEN 'paragraph' THEN 'paragraph'
               ELSE 'word'
           END;
        ",
    )
    .map_err(|error| error.to_string())?;
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
