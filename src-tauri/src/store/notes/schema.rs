use rusqlite::Connection;

use crate::store::notes::{
    migration::{migrate_fts_table, migrate_note_kind_values},
    tag_index::rebuild_note_tags_index,
};

pub(super) fn init_schema(conn: &Connection) -> Result<(), String> {
    migrate_note_kind_values(conn)?;
    migrate_fts_table(conn)?;
    let should_rebuild_note_tags = !table_exists(conn, "note_tags")?;

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

        CREATE TABLE IF NOT EXISTS note_tags (
            note_id TEXT NOT NULL,
            tag TEXT NOT NULL COLLATE NOCASE,
            PRIMARY KEY (note_id, tag),
            FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_note_tags_tag
            ON note_tags (tag COLLATE NOCASE);
        CREATE INDEX IF NOT EXISTS idx_note_tags_note_id
            ON note_tags (note_id);

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
    if should_rebuild_note_tags {
        rebuild_note_tags_index(conn)?;
    }

    Ok(())
}

fn table_exists(conn: &Connection, table_name: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [table_name],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value != 0)
    .map_err(|error| error.to_string())
}

pub(super) fn rebuild_fts_index(conn: &Connection) -> Result<(), String> {
    conn.execute("INSERT INTO notes_fts(notes_fts) VALUES('rebuild')", [])
        .map_err(|error| error.to_string())?;
    Ok(())
}
