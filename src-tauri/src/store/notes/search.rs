use rusqlite::{params_from_iter, types::Value, Connection};

use crate::{
    core::text::{build_fts_query, build_like_pattern},
    store::notes::{
        kinds::normalize_search_note_kind,
        repository::{collect_notes, map_note_row},
        types::{NoteDto, NotesCursor, NotesPage},
    },
};

pub(crate) fn search_notes(
    conn: &Connection,
    query: String,
    limit: Option<i64>,
    cursor_rank: Option<f64>,
    cursor_updated_at: Option<i64>,
    cursor_id: Option<String>,
) -> Result<NotesPage, String> {
    let page = SearchPage::new(limit, cursor_rank, cursor_updated_at, cursor_id);
    let parsed_query = parse_search_query(&query);

    if parsed_query.text.is_empty() && parsed_query.filters.has_filters() {
        return search_notes_by_filters(conn, &parsed_query.filters, page);
    }

    if !parsed_query.can_run_text_search() {
        return Ok(NotesPage {
            notes: Vec::new(),
            next_cursor: None,
            total_count: 0,
        });
    }

    let Some(fts_query) = build_fts_query(&parsed_query.text) else {
        return Ok(NotesPage {
            notes: Vec::new(),
            next_cursor: None,
            total_count: 0,
        });
    };

    if !parsed_query.can_use_fts() {
        return search_notes_like(conn, &parsed_query.text, &parsed_query.filters, page);
    }

    let filters = build_filter_sql("notes", &parsed_query.filters, 2);
    let mut values = Vec::with_capacity(6 + filters.param_count);
    values.push(Value::Text(fts_query));
    values.extend(filters.values.iter().cloned());
    let total_count = if page.fts_cursor().is_some() {
        -1
    } else {
        count_notes_fts(conn, &filters.sql, &values)?
    };

    let cursor_sql = if let Some(cursor) = page.fts_cursor() {
        let rank_param = filters.param_count + 2;
        let updated_at_param = filters.param_count + 3;
        let id_param = filters.param_count + 4;
        values.push(Value::Real(cursor.rank));
        values.push(Value::Integer(cursor.updated_at));
        values.push(Value::Text(cursor.id.to_string()));
        format!(
            " AND (
                bm25(notes_fts) > ?{rank_param}
                OR (bm25(notes_fts) = ?{rank_param} AND notes.updated_at < ?{updated_at_param})
                OR (bm25(notes_fts) = ?{rank_param} AND notes.updated_at = ?{updated_at_param} AND notes.id < ?{id_param})
              )"
        )
    } else {
        String::new()
    };

    let limit_param = values.len() + 1;
    values.push(Value::Integer(page.size));

    let mut stmt = conn
        .prepare(
            &format!(
                "SELECT notes.id, notes.title, notes.content, notes.kind, notes.tone, notes.tags_json, notes.created_at, notes.updated_at, bm25(notes_fts) AS rank
             FROM notes_fts
             JOIN notes ON notes_fts.rowid = notes.rowid
             WHERE notes_fts MATCH ?1
               {filter_sql}
               {cursor_sql}
             ORDER BY bm25(notes_fts), notes.updated_at DESC, notes.id DESC
             LIMIT ?{limit_param}",
                filter_sql = filters.sql,
                cursor_sql = cursor_sql,
            ),
        )
        .map_err(|error| error.to_string())?;

    let ranked_notes = stmt
        .query_map(params_from_iter(values), |row| {
            Ok((map_note_row(row)?, row.get::<_, f64>(8)?))
        })
        .map_err(|error| error.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())?;
    let next_cursor = ranked_notes.last().map(|(note, rank)| NotesCursor {
        updated_at: note.updated_at,
        id: note.id.clone(),
        rank: Some(*rank),
    });
    let notes = ranked_notes
        .into_iter()
        .map(|(note, _)| note)
        .collect::<Vec<_>>();

    Ok(NotesPage {
        notes,
        next_cursor,
        total_count,
    })
}

fn search_notes_like(
    conn: &Connection,
    query: &str,
    filters: &SearchFilters,
    page: SearchPage,
) -> Result<NotesPage, String> {
    let like_pattern = build_like_pattern(query);
    let filters = build_filter_sql("notes", filters, 2);
    let mut values = Vec::with_capacity(5 + filters.param_count);
    values.push(Value::Text(like_pattern));
    values.extend(filters.values.iter().cloned());
    let total_count = if page.has_cursor() {
        -1
    } else {
        count_notes_like(conn, &filters.sql, &values)?
    };
    let cursor_sql = push_updated_at_cursor_sql(&mut values, &page, filters.param_count + 2);
    let limit_param = values.len() + 1;
    values.push(Value::Integer(page.size));
    let mut stmt = conn
        .prepare(&format!(
            "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
             FROM notes
             WHERE (COALESCE(title, '') LIKE ?1 ESCAPE '\\'
                OR COALESCE(content, '') LIKE ?1 ESCAPE '\\')
               {filter_sql}
               {cursor_sql}
             ORDER BY updated_at DESC, id DESC
             LIMIT ?{limit_param}",
            filter_sql = filters.sql,
            cursor_sql = cursor_sql,
        ))
        .map_err(|error| error.to_string())?;

    let notes = collect_notes(
        stmt.query_map(params_from_iter(values), map_note_row)
            .map_err(|error| error.to_string())?,
    )?;
    let next_cursor = updated_at_next_cursor(&notes);

    Ok(NotesPage {
        notes,
        next_cursor,
        total_count,
    })
}

fn search_notes_by_filters(
    conn: &Connection,
    filters: &SearchFilters,
    page: SearchPage,
) -> Result<NotesPage, String> {
    let filters = build_filter_sql("notes", filters, 1);
    let mut values = filters.values.clone();
    let total_count = if page.has_cursor() {
        -1
    } else {
        count_notes_by_filters(conn, &filters.sql, &values)?
    };
    let cursor_sql = push_updated_at_cursor_sql(&mut values, &page, filters.param_count + 1);
    let limit_param = values.len() + 1;
    values.push(Value::Integer(page.size));

    let mut stmt = conn
        .prepare(&format!(
            "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
                 FROM notes
                 WHERE 1 = 1
                   {filter_sql}
                   {cursor_sql}
                 ORDER BY updated_at DESC, id DESC
                 LIMIT ?{limit_param}",
            filter_sql = filters.sql,
            cursor_sql = cursor_sql,
        ))
        .map_err(|error| error.to_string())?;

    let notes = collect_notes(
        stmt.query_map(params_from_iter(values), map_note_row)
            .map_err(|error| error.to_string())?,
    )?;
    let next_cursor = updated_at_next_cursor(&notes);

    Ok(NotesPage {
        notes,
        next_cursor,
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

fn count_notes_by_filters(
    conn: &Connection,
    filter_sql: &str,
    values: &[Value],
) -> Result<i64, String> {
    conn.query_row(
        &format!(
            "SELECT COUNT(*)
             FROM notes
             WHERE 1 = 1
               {filter_sql}"
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
                " AND EXISTS (SELECT 1 FROM note_tags WHERE note_tags.note_id = {table_alias}.id AND note_tags.tag = ?{} COLLATE NOCASE)",
                first_param_index + index
            )
        })
        .collect::<String>();
    let excluded_sql = (0..excluded_tag_count)
        .map(|index| {
            format!(
                " AND NOT EXISTS (SELECT 1 FROM note_tags WHERE note_tags.note_id = {table_alias}.id AND note_tags.tag = ?{} COLLATE NOCASE)",
                first_param_index + included_tag_count + index
            )
        })
        .collect::<String>();

    format!("{included_sql}{excluded_sql}")
}

fn build_kind_filter_sql(
    table_alias: &str,
    included_kind_count: usize,
    excluded_kind_count: usize,
    first_param_index: usize,
) -> String {
    let included_sql = (0..included_kind_count)
        .map(|index| format!(" AND {table_alias}.kind = ?{}", first_param_index + index))
        .collect::<String>();
    let excluded_sql = (0..excluded_kind_count)
        .map(|index| {
            format!(
                " AND {table_alias}.kind != ?{}",
                first_param_index + included_kind_count + index
            )
        })
        .collect::<String>();

    format!("{included_sql}{excluded_sql}")
}

#[derive(Clone)]
struct SearchPage {
    size: i64,
    cursor_rank: Option<f64>,
    cursor_updated_at: Option<i64>,
    cursor_id: Option<String>,
}

impl SearchPage {
    fn new(
        limit: Option<i64>,
        cursor_rank: Option<f64>,
        cursor_updated_at: Option<i64>,
        cursor_id: Option<String>,
    ) -> Self {
        Self {
            size: limit.unwrap_or(80).clamp(1, 100),
            cursor_rank,
            cursor_updated_at,
            cursor_id,
        }
    }

    fn has_cursor(&self) -> bool {
        self.cursor_updated_at.is_some() && self.cursor_id.is_some()
    }

    fn updated_at_cursor(&self) -> Option<UpdatedAtCursor<'_>> {
        Some(UpdatedAtCursor {
            updated_at: self.cursor_updated_at?,
            id: self.cursor_id.as_deref()?,
        })
    }

    fn fts_cursor(&self) -> Option<FtsCursor<'_>> {
        Some(FtsCursor {
            rank: self.cursor_rank?,
            updated_at: self.cursor_updated_at?,
            id: self.cursor_id.as_deref()?,
        })
    }
}

struct UpdatedAtCursor<'a> {
    updated_at: i64,
    id: &'a str,
}

struct FtsCursor<'a> {
    rank: f64,
    updated_at: i64,
    id: &'a str,
}

fn push_updated_at_cursor_sql(
    values: &mut Vec<Value>,
    page: &SearchPage,
    first_param_index: usize,
) -> String {
    let Some(cursor) = page.updated_at_cursor() else {
        return String::new();
    };

    values.push(Value::Integer(cursor.updated_at));
    values.push(Value::Text(cursor.id.to_string()));

    let updated_at_param = first_param_index;
    let id_param = first_param_index + 1;

    format!(
        " AND (notes.updated_at < ?{updated_at_param}
            OR (notes.updated_at = ?{updated_at_param} AND notes.id < ?{id_param}))"
    )
}

fn updated_at_next_cursor(notes: &[NoteDto]) -> Option<NotesCursor> {
    notes.last().map(|note| NotesCursor {
        updated_at: note.updated_at,
        id: note.id.clone(),
        rank: None,
    })
}

#[derive(Default)]
struct SearchFilters {
    included_tags: Vec<String>,
    excluded_tags: Vec<String>,
    included_kinds: Vec<String>,
    excluded_kinds: Vec<String>,
}

impl SearchFilters {
    fn has_filters(&self) -> bool {
        !self.included_tags.is_empty()
            || !self.excluded_tags.is_empty()
            || !self.included_kinds.is_empty()
            || !self.excluded_kinds.is_empty()
    }
}

struct BuiltFilters {
    sql: String,
    values: Vec<Value>,
    param_count: usize,
}

fn build_filter_sql(
    table_alias: &str,
    filters: &SearchFilters,
    first_param_index: usize,
) -> BuiltFilters {
    let tag_filter_sql = build_tag_filter_sql(
        table_alias,
        filters.included_tags.len(),
        filters.excluded_tags.len(),
        first_param_index,
    );
    let tag_param_count = filters.included_tags.len() + filters.excluded_tags.len();
    let kind_filter_sql = build_kind_filter_sql(
        table_alias,
        filters.included_kinds.len(),
        filters.excluded_kinds.len(),
        first_param_index + tag_param_count,
    );
    let param_count = tag_param_count + filters.included_kinds.len() + filters.excluded_kinds.len();
    let values = filters
        .included_tags
        .iter()
        .chain(filters.excluded_tags.iter())
        .chain(filters.included_kinds.iter())
        .chain(filters.excluded_kinds.iter())
        .cloned()
        .map(Value::Text)
        .collect::<Vec<_>>();

    BuiltFilters {
        sql: format!("{tag_filter_sql}{kind_filter_sql}"),
        values,
        param_count,
    }
}

struct ParsedSearchQuery {
    text: String,
    text_terms: Vec<String>,
    filters: SearchFilters,
}

impl ParsedSearchQuery {
    fn can_run_text_search(&self) -> bool {
        if self.text_terms.is_empty() {
            return false;
        }

        self.filters.has_filters()
            || self
                .text_terms
                .iter()
                .all(|term| is_like_searchable_term(term))
    }

    fn can_use_fts(&self) -> bool {
        self.text_terms.iter().all(|term| term.chars().count() >= 3)
    }
}

fn parse_search_query(query: &str) -> ParsedSearchQuery {
    let mut text_terms = Vec::new();
    let mut filters = SearchFilters::default();

    for term in query.split_whitespace() {
        if let Some(filter) = term.strip_prefix("!@").and_then(clean_search_tag) {
            if let Some(kind) = normalize_search_note_kind(&filter) {
                push_unique_value(&mut filters.excluded_kinds, kind);
            }
        } else if let Some(filter) = term.strip_prefix('@').and_then(clean_search_tag) {
            if let Some(kind) = normalize_search_note_kind(&filter) {
                push_unique_value(&mut filters.included_kinds, kind);
            }
        } else if let Some(filter) = term.strip_prefix("!#").and_then(clean_search_tag) {
            remove_value(&mut filters.included_tags, &filter);
            push_unique_value(&mut filters.excluded_tags, filter);
        } else if let Some(filter) = term.strip_prefix('#').and_then(clean_search_tag) {
            if !filters
                .excluded_tags
                .iter()
                .any(|item: &String| item.eq_ignore_ascii_case(&filter))
            {
                push_unique_value(&mut filters.included_tags, filter);
            }
        } else {
            text_terms.push(term);
        }
    }

    let text = text_terms.join(" ");

    ParsedSearchQuery {
        text,
        text_terms: text_terms
            .into_iter()
            .map(|term| term.to_string())
            .collect(),
        filters,
    }
}

fn push_unique_value(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|item| item.eq_ignore_ascii_case(&value)) {
        values.push(value);
    }
}

fn remove_value(values: &mut Vec<String>, value: &str) {
    values.retain(|item| !item.eq_ignore_ascii_case(value));
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

fn is_like_searchable_term(value: &str) -> bool {
    value.chars().count() >= 3 || count_cjk_chars(value) >= 2
}

fn count_cjk_chars(value: &str) -> usize {
    value
        .chars()
        .filter(|ch| {
            matches!(
                *ch,
                '\u{3400}'..='\u{9fff}'
                    | '\u{f900}'..='\u{faff}'
                    | '\u{3040}'..='\u{30ff}'
                    | '\u{ac00}'..='\u{d7af}'
            )
        })
        .count()
}

#[cfg(test)]
#[path = "../../tests/store/notes/search.rs"]
mod tests;
