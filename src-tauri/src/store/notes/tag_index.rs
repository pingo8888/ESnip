use rusqlite::{params, Connection};

pub(super) fn rebuild_note_tags_index(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM note_tags", [])
        .map_err(|error| error.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, tags_json FROM notes")
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| error.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())?;

    for row in rows {
        let (note_id, tags_json) = row;
        let tags = serde_json::from_str::<Vec<String>>(&tags_json).unwrap_or_default();
        sync_note_tags(conn, &note_id, &tags)?;
    }

    Ok(())
}

pub(super) fn sync_note_tags(
    conn: &Connection,
    note_id: &str,
    tags: &[String],
) -> Result<(), String> {
    conn.execute("DELETE FROM note_tags WHERE note_id = ?1", params![note_id])
        .map_err(|error| error.to_string())?;

    for tag in normalize_tags(tags) {
        conn.execute(
            "INSERT OR IGNORE INTO note_tags (note_id, tag) VALUES (?1, ?2)",
            params![note_id, tag],
        )
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn normalize_tags(tags: &[String]) -> Vec<String> {
    let mut normalized_tags = Vec::new();

    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty()
            || normalized_tags
                .iter()
                .any(|item: &String| item.eq_ignore_ascii_case(tag))
        {
            continue;
        }

        normalized_tags.push(tag.to_string());
    }

    normalized_tags
}
