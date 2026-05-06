use rusqlite::{Connection, OptionalExtension};

pub(super) fn migrate_note_kind_values(conn: &Connection) -> Result<(), String> {
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
        DROP TABLE IF EXISTS note_tags;
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

pub(super) fn migrate_fts_table(conn: &Connection) -> Result<(), String> {
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
