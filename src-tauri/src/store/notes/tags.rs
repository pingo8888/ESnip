use rusqlite::{params, Connection};

use crate::{
    core::text::build_like_pattern,
    store::notes::types::{NoteKindCountDto, TagSuggestionDto},
};

pub(crate) fn list_tags(
    conn: &Connection,
    prefix: String,
    limit: Option<i64>,
) -> Result<Vec<TagSuggestionDto>, String> {
    let cleaned_prefix = prefix.trim().trim_start_matches('#').trim();
    let like_pattern = build_like_pattern(cleaned_prefix);

    if let Some(page_size) = limit {
        let mut stmt = conn
            .prepare(
                "SELECT tag, COUNT(DISTINCT note_id) AS count
                 FROM note_tags
                 WHERE ?1 = '' OR tag LIKE ?2 ESCAPE '\\'
                 GROUP BY tag
                 ORDER BY count DESC, tag COLLATE NOCASE
                 LIMIT ?3",
            )
            .map_err(|error| error.to_string())?;

        let tags = stmt
            .query_map(params![cleaned_prefix, like_pattern, page_size], |row| {
                Ok(TagSuggestionDto {
                    label: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string());

        tags
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT tag, COUNT(DISTINCT note_id) AS count
                 FROM note_tags
                 WHERE ?1 = '' OR tag LIKE ?2 ESCAPE '\\'
                 GROUP BY tag
                 ORDER BY count DESC, tag COLLATE NOCASE",
            )
            .map_err(|error| error.to_string())?;

        let tags = stmt
            .query_map(params![cleaned_prefix, like_pattern], |row| {
                Ok(TagSuggestionDto {
                    label: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string());

        tags
    }
}

pub(crate) fn list_note_kind_counts(conn: &Connection) -> Result<Vec<NoteKindCountDto>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT kind, COUNT(*) AS count
             FROM notes
             GROUP BY kind",
        )
        .map_err(|error| error.to_string())?;

    let counts = stmt
        .query_map([], |row| {
            Ok(NoteKindCountDto {
                value: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string());

    counts
}
