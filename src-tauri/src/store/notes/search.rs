use rusqlite::{params_from_iter, types::Value, Connection};

use crate::{
    core::text::{build_fts_query, build_like_pattern},
    store::notes::{
        kinds::normalize_search_note_kind,
        repository::{collect_notes, map_note_row},
        types::NotesPage,
    },
};

pub(crate) fn search_notes(
    conn: &Connection,
    query: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<NotesPage, String> {
    let page = SearchPage::new(limit, offset);
    let parsed_query = parse_search_query(&query);

    if parsed_query.text.is_empty() && parsed_query.filters.has_filters() {
        return search_notes_by_filters(conn, &parsed_query.filters, page);
    }

    let Some(fts_query) = build_fts_query(&parsed_query.text) else {
        return Ok(NotesPage {
            notes: Vec::new(),
            next_cursor: None,
            total_count: 0,
        });
    };

    if parsed_query.text.trim().chars().count() < 3 {
        return search_notes_like(conn, &parsed_query.text, &parsed_query.filters, page);
    }

    let filters = build_filter_sql("notes", &parsed_query.filters, 2);
    let mut values = Vec::with_capacity(3 + filters.param_count);
    values.push(Value::Text(fts_query));
    values.extend(filters.values.iter().cloned());
    let total_count = count_notes_fts(conn, &filters.sql, &values)?;
    values.push(Value::Integer(page.size));
    values.push(Value::Integer(page.offset));

    let mut stmt = conn
        .prepare(
            &format!(
                "SELECT notes.id, notes.title, notes.content, notes.kind, notes.tone, notes.tags_json, notes.created_at, notes.updated_at
             FROM notes_fts
             JOIN notes ON notes_fts.rowid = notes.rowid
             WHERE notes_fts MATCH ?1
               {filter_sql}
             ORDER BY bm25(notes_fts), notes.updated_at DESC, notes.id DESC
             LIMIT ?{} OFFSET ?{}",
                filters.param_count + 2,
                filters.param_count + 3,
                filter_sql = filters.sql
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

fn search_notes_like(
    conn: &Connection,
    query: &str,
    filters: &SearchFilters,
    page: SearchPage,
) -> Result<NotesPage, String> {
    let like_pattern = build_like_pattern(query);
    let filters = build_filter_sql("notes", filters, 2);
    let mut values = Vec::with_capacity(3 + filters.param_count);
    values.push(Value::Text(like_pattern));
    values.extend(filters.values.iter().cloned());
    let total_count = count_notes_like(conn, &filters.sql, &values)?;
    values.push(Value::Integer(page.size));
    values.push(Value::Integer(page.offset));
    let mut stmt = conn
        .prepare(&format!(
            "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
             FROM notes
             WHERE (COALESCE(title, '') LIKE ?1 ESCAPE '\\'
                OR COALESCE(content, '') LIKE ?1 ESCAPE '\\')
               {filter_sql}
             ORDER BY updated_at DESC, id DESC
             LIMIT ?{} OFFSET ?{}",
            filters.param_count + 2,
            filters.param_count + 3,
            filter_sql = filters.sql
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

fn search_notes_by_filters(
    conn: &Connection,
    filters: &SearchFilters,
    page: SearchPage,
) -> Result<NotesPage, String> {
    let filters = build_filter_sql("notes", filters, 1);
    let mut values = filters.values.clone();
    let total_count = count_notes_by_filters(conn, &filters.sql, &values)?;
    values.push(Value::Integer(page.size));
    values.push(Value::Integer(page.offset));

    let mut stmt = conn
        .prepare(&format!(
            "SELECT id, title, content, kind, tone, tags_json, created_at, updated_at
                 FROM notes
                 WHERE 1 = 1
                   {filter_sql}
                 ORDER BY updated_at DESC, id DESC
                 LIMIT ?{} OFFSET ?{}",
            filters.param_count + 1,
            filters.param_count + 2,
            filter_sql = filters.sql
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

#[derive(Clone, Copy)]
struct SearchPage {
    size: i64,
    offset: i64,
}

impl SearchPage {
    fn new(limit: Option<i64>, offset: Option<i64>) -> Self {
        Self {
            size: limit.unwrap_or(80).clamp(1, 100),
            offset: offset.unwrap_or(0).max(0),
        }
    }
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
    filters: SearchFilters,
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

    ParsedSearchQuery {
        text: text_terms.join(" "),
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
