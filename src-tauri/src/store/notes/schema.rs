use rusqlite::Connection;

use crate::store::notes::migration::{migrate_fts_table, migrate_note_kind_values};

pub(super) fn init_schema(conn: &Connection) -> Result<(), String> {
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

pub(super) fn rebuild_fts_index(conn: &Connection) -> Result<(), String> {
    conn.execute("INSERT INTO notes_fts(notes_fts) VALUES('rebuild')", [])
        .map_err(|error| error.to_string())?;
    Ok(())
}
