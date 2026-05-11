use rusqlite::{params, Connection};

use super::{
    create_note, schema,
    types::{NoteDto, SaveNoteInput},
};

pub(crate) fn open_test_connection() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory database");
    conn.pragma_update(None, "foreign_keys", "ON")
        .expect("enable foreign keys");
    schema::init_schema(&conn).expect("initialize test schema");
    conn
}

pub(crate) fn note_input(
    title: Option<&str>,
    excerpt: Option<&str>,
    kind: &str,
    tags: &[&str],
) -> SaveNoteInput {
    SaveNoteInput {
        title: title.map(str::to_string),
        excerpt: excerpt.map(str::to_string),
        kind: kind.to_string(),
        tone: "sage".to_string(),
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
    }
}

pub(crate) fn create_test_note(
    conn: &Connection,
    title: Option<&'static str>,
    excerpt: Option<&'static str>,
    kind: &str,
    tags: &[&str],
    updated_at: i64,
) -> NoteDto {
    let note = create_note(conn, note_input(title, excerpt, kind, tags)).expect("create note");
    set_note_updated_at(conn, &note.id, updated_at);

    NoteDto { updated_at, ..note }
}

pub(crate) fn set_note_updated_at(conn: &Connection, note_id: &str, updated_at: i64) {
    conn.execute(
        "UPDATE notes SET created_at = ?1, updated_at = ?1 WHERE id = ?2",
        params![updated_at, note_id],
    )
    .expect("set note timestamp");
}

pub(crate) fn note_tags(conn: &Connection, note_id: &str) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT tag FROM note_tags WHERE note_id = ?1 ORDER BY tag COLLATE NOCASE")
        .expect("prepare note tag query");

    stmt.query_map(params![note_id], |row| row.get::<_, String>(0))
        .expect("query note tags")
        .collect::<rusqlite::Result<Vec<_>>>()
        .expect("collect note tags")
}

pub(crate) fn tag_count(conn: &Connection, tag: &str) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM note_tags WHERE tag = ?1 COLLATE NOCASE",
        params![tag],
        |row| row.get(0),
    )
    .expect("count tag")
}
