use rusqlite::{params, Connection};

use crate::core::text::build_like_pattern;

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
