use std::collections::HashSet;

use rusqlite::Connection;

use super::*;
use crate::store::notes::test_utils::{create_test_note, open_test_connection};

#[test]
fn filter_only_search_supports_tags_and_kinds() {
    let conn = open_test_connection();
    create_test_note(
        &conn,
        Some("Rust word"),
        Some("systems note"),
        "word",
        &["tech"],
        300,
    );
    create_test_note(
        &conn,
        Some("Rust sentence"),
        Some("systems note"),
        "sentence",
        &["tech", "skip"],
        200,
    );
    create_test_note(
        &conn,
        Some("Garden paragraph"),
        Some("life note"),
        "paragraph",
        &["life"],
        100,
    );

    assert_eq!(
        search_titles(&conn, "#tech"),
        vec!["Rust word", "Rust sentence"]
    );
    assert_eq!(search_titles(&conn, "#tech !#skip"), vec!["Rust word"]);
    assert_eq!(search_titles(&conn, "@词语"), vec!["Rust word"]);
    assert_eq!(
        search_titles(&conn, "!@句子"),
        vec!["Rust word", "Garden paragraph"]
    );
}

#[test]
fn short_query_rules_are_consistent_with_like_fallback() {
    let conn = open_test_connection();
    create_test_note(
        &conn,
        Some("工学理论"),
        Some("两字中文短查询需要走 LIKE"),
        "word",
        &["topic"],
        100,
    );

    assert!(search_titles(&conn, "工").is_empty());
    assert!(search_titles(&conn, "工 学").is_empty());
    assert_eq!(search_titles(&conn, "工学"), vec!["工学理论"]);
    assert_eq!(search_titles(&conn, "#topic 工"), vec!["工学理论"]);
}

#[test]
fn text_search_supports_slash_or_groups() {
    let conn = open_test_connection();
    create_test_note(
        &conn,
        Some("Alchemy guide"),
        Some("reference"),
        "word",
        &[],
        300,
    );
    create_test_note(
        &conn,
        Some("Garden guide"),
        Some("reference"),
        "word",
        &[],
        200,
    );
    create_test_note(
        &conn,
        Some("Kitchen note"),
        Some("reference"),
        "word",
        &[],
        100,
    );

    assert_titles_match(
        search_titles(&conn, "alchemy/garden"),
        &["Alchemy guide", "Garden guide"],
    );
}

#[test]
fn text_search_combines_slash_or_groups_with_space_and() {
    let conn = open_test_connection();
    create_test_note(
        &conn,
        Some("Alchemy reference"),
        Some("old guide"),
        "word",
        &[],
        300,
    );
    create_test_note(
        &conn,
        Some("Garden reference"),
        Some("green guide"),
        "word",
        &[],
        200,
    );
    create_test_note(
        &conn,
        Some("Alchemy draft"),
        Some("loose note"),
        "word",
        &[],
        100,
    );

    assert_titles_match(
        search_titles(&conn, "alchemy/garden reference"),
        &["Alchemy reference", "Garden reference"],
    );
}

#[test]
fn text_slash_or_groups_work_with_filters() {
    let conn = open_test_connection();
    create_test_note(
        &conn,
        Some("Alchemy guide"),
        Some("reference"),
        "word",
        &["tech"],
        300,
    );
    create_test_note(
        &conn,
        Some("Garden guide"),
        Some("reference"),
        "word",
        &["life"],
        200,
    );
    create_test_note(
        &conn,
        Some("Garden tech"),
        Some("reference"),
        "word",
        &["tech"],
        100,
    );

    assert_eq!(
        search_titles(&conn, "#tech alchemy/garden"),
        vec!["Alchemy guide", "Garden tech"]
    );
}

#[test]
fn slash_is_not_or_for_tag_filters() {
    let conn = open_test_connection();
    create_test_note(
        &conn,
        Some("Tech note"),
        Some("reference"),
        "word",
        &["tech"],
        300,
    );
    create_test_note(
        &conn,
        Some("Life note"),
        Some("reference"),
        "word",
        &["life"],
        200,
    );

    assert!(search_titles(&conn, "#tech/#life").is_empty());
}

#[test]
fn slash_or_short_terms_follow_existing_short_query_rules() {
    let conn = open_test_connection();
    create_test_note(
        &conn,
        Some("工学理论"),
        Some("短查询"),
        "word",
        &["topic"],
        100,
    );

    assert!(search_titles(&conn, "工/学").is_empty());
    assert_eq!(search_titles(&conn, "#topic 工/学"), vec!["工学理论"]);
}

#[test]
fn fts_search_uses_cursor_without_duplicates() {
    let conn = open_test_connection();
    create_test_note(
        &conn,
        Some("Alchemy newest"),
        Some("alchemy reference"),
        "word",
        &[],
        300,
    );
    create_test_note(
        &conn,
        Some("Alchemy middle"),
        Some("alchemy reference"),
        "word",
        &[],
        200,
    );
    create_test_note(
        &conn,
        Some("Alchemy oldest"),
        Some("alchemy reference"),
        "word",
        &[],
        100,
    );

    let first = search_notes(&conn, "alchemy".to_string(), Some(2), None, None, None)
        .expect("first FTS page");
    assert_eq!(first.total_count, 3);
    assert_eq!(first.notes.len(), 2);
    let cursor = first.next_cursor.expect("FTS cursor");
    assert!(cursor.rank.is_some());

    let second = search_notes(
        &conn,
        "alchemy".to_string(),
        Some(2),
        cursor.rank,
        Some(cursor.updated_at),
        Some(cursor.id),
    )
    .expect("second FTS page");

    assert_eq!(second.total_count, -1);
    assert_eq!(second.notes.len(), 1);
    let ids = first
        .notes
        .iter()
        .chain(second.notes.iter())
        .map(|note| note.id.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(ids.len(), 3);
}

#[test]
fn fts_cursor_uses_updated_at_when_ranks_are_tied() {
    let conn = open_test_connection();
    create_test_note(
        &conn,
        Some("identical alchemy"),
        Some("identical alchemy"),
        "word",
        &[],
        300,
    );
    create_test_note(
        &conn,
        Some("identical alchemy"),
        Some("identical alchemy"),
        "word",
        &[],
        200,
    );
    create_test_note(
        &conn,
        Some("identical alchemy"),
        Some("identical alchemy"),
        "word",
        &[],
        100,
    );

    let first = search_notes(&conn, "alchemy".to_string(), Some(2), None, None, None)
        .expect("first tied-rank FTS page");
    assert_eq!(
        first
            .notes
            .iter()
            .map(|note| note.updated_at)
            .collect::<Vec<_>>(),
        vec![300, 200]
    );

    let cursor = first.next_cursor.expect("tied-rank FTS cursor");
    let second = search_notes(
        &conn,
        "alchemy".to_string(),
        Some(2),
        cursor.rank,
        Some(cursor.updated_at),
        Some(cursor.id),
    )
    .expect("second tied-rank FTS page");

    assert_eq!(second.notes.len(), 1);
    assert_eq!(second.notes[0].updated_at, 100);
}

#[test]
fn like_search_uses_updated_at_cursor() {
    let conn = open_test_connection();
    create_test_note(&conn, Some("炼金甲"), Some("短中文查询"), "word", &[], 300);
    create_test_note(&conn, Some("炼金乙"), Some("短中文查询"), "word", &[], 200);
    create_test_note(&conn, Some("炼金丙"), Some("短中文查询"), "word", &[], 100);

    let first = search_notes(&conn, "炼金".to_string(), Some(2), None, None, None)
        .expect("first LIKE page");
    assert_eq!(first.total_count, 3);
    assert_eq!(first.notes.len(), 2);
    let cursor = first.next_cursor.expect("LIKE cursor");
    assert!(cursor.rank.is_none());

    let second = search_notes(
        &conn,
        "炼金".to_string(),
        Some(2),
        cursor.rank,
        Some(cursor.updated_at),
        Some(cursor.id),
    )
    .expect("second LIKE page");

    assert_eq!(second.total_count, -1);
    assert_eq!(second.notes.len(), 1);
    assert_eq!(second.notes[0].title.as_deref(), Some("炼金丙"));
}

fn search_titles(conn: &Connection, query: &str) -> Vec<String> {
    search_notes(conn, query.to_string(), Some(20), None, None, None)
        .expect("search notes")
        .notes
        .into_iter()
        .filter_map(|note| note.title)
        .collect()
}

fn assert_titles_match(actual: Vec<String>, expected: &[&str]) {
    let actual = actual.into_iter().collect::<HashSet<_>>();
    let expected = expected
        .iter()
        .map(|title| title.to_string())
        .collect::<HashSet<_>>();

    assert_eq!(actual, expected);
}
