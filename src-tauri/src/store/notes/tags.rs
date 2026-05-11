use rusqlite::{params, Connection};

use crate::{
    core::text::build_like_pattern,
    store::notes::{
        tag_index::sync_note_tags,
        types::{NoteKindCountDto, TagSuggestionDto},
    },
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

pub(crate) fn rename_tag(
    conn: &Connection,
    old_tag: String,
    new_tag: String,
) -> Result<Vec<TagSuggestionDto>, String> {
    let old_tag = clean_tag(&old_tag).ok_or_else(|| "errors.tagRequired".to_string())?;
    let new_tag = clean_tag(&new_tag).ok_or_else(|| "errors.newTagRequired".to_string())?;

    if old_tag == new_tag {
        return list_tags(conn, String::new(), None);
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let affected_notes = collect_notes_with_tag(&tx, &old_tag)?;

    if affected_notes.is_empty() {
        return Err("errors.tagNotFound".to_string());
    }

    for (note_id, tags_json) in affected_notes {
        let tags = serde_json::from_str::<Vec<String>>(&tags_json).unwrap_or_default();
        let next_tags = rename_tag_values(&tags, &old_tag, &new_tag);
        let next_tags_json =
            serde_json::to_string(&next_tags).map_err(|error| error.to_string())?;

        tx.execute(
            "UPDATE notes SET tags_json = ?1 WHERE id = ?2",
            params![next_tags_json, note_id],
        )
        .map_err(|error| error.to_string())?;
        sync_note_tags(&tx, &note_id, &next_tags)?;
    }

    tx.commit().map_err(|error| error.to_string())?;
    list_tags(conn, String::new(), None)
}

pub(crate) fn delete_tag(conn: &Connection, tag: String) -> Result<Vec<TagSuggestionDto>, String> {
    let tag = clean_tag(&tag).ok_or_else(|| "errors.tagRequired".to_string())?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let affected_notes = collect_notes_with_tag(&tx, &tag)?;

    if affected_notes.is_empty() {
        return Err("errors.tagNotFound".to_string());
    }

    for (note_id, tags_json) in affected_notes {
        let tags = serde_json::from_str::<Vec<String>>(&tags_json).unwrap_or_default();
        let next_tags = delete_tag_values(&tags, &tag);
        let next_tags_json =
            serde_json::to_string(&next_tags).map_err(|error| error.to_string())?;

        tx.execute(
            "UPDATE notes SET tags_json = ?1 WHERE id = ?2",
            params![next_tags_json, note_id],
        )
        .map_err(|error| error.to_string())?;
        sync_note_tags(&tx, &note_id, &next_tags)?;
    }

    tx.commit().map_err(|error| error.to_string())?;
    list_tags(conn, String::new(), None)
}

fn collect_notes_with_tag(conn: &Connection, tag: &str) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT notes.id, notes.tags_json
             FROM notes
             JOIN note_tags ON note_tags.note_id = notes.id
             WHERE note_tags.tag = ?1 COLLATE NOCASE
             ORDER BY notes.id",
        )
        .map_err(|error| error.to_string())?;

    let notes = stmt
        .query_map(params![tag], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|error| error.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string());

    notes
}

fn rename_tag_values(tags: &[String], old_tag: &str, new_tag: &str) -> Vec<String> {
    let renamed = tags
        .iter()
        .filter_map(|tag| {
            let cleaned_tag = clean_tag(tag)?;
            if cleaned_tag.eq_ignore_ascii_case(old_tag) {
                Some(new_tag.to_string())
            } else {
                Some(cleaned_tag)
            }
        })
        .collect::<Vec<_>>();

    dedupe_tags(renamed)
}

fn delete_tag_values(tags: &[String], deleted_tag: &str) -> Vec<String> {
    let filtered = tags
        .iter()
        .filter_map(|tag| {
            let cleaned_tag = clean_tag(tag)?;
            if cleaned_tag.eq_ignore_ascii_case(deleted_tag) {
                None
            } else {
                Some(cleaned_tag)
            }
        })
        .collect::<Vec<_>>();

    dedupe_tags(filtered)
}

fn dedupe_tags(tags: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();

    for tag in tags {
        if !deduped
            .iter()
            .any(|item: &String| item.eq_ignore_ascii_case(&tag))
        {
            deduped.push(tag);
        }
    }

    deduped
}

fn clean_tag(tag: &str) -> Option<String> {
    let tag = tag.trim().trim_start_matches('#').trim();

    if tag.is_empty() || tag.chars().any(char::is_whitespace) {
        None
    } else {
        Some(tag.to_string())
    }
}
