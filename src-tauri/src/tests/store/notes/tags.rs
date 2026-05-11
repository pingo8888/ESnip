use rusqlite::{params, Connection};

use super::*;
use crate::store::notes::test_utils::{create_test_note, open_test_connection, tag_count};

#[test]
fn list_tags_orders_by_count_and_applies_limit() {
    let conn = open_test_connection();
    create_test_note(
        &conn,
        Some("one"),
        Some("body"),
        "word",
        &["alpha", "beta"],
        100,
    );
    create_test_note(&conn, Some("two"), Some("body"), "word", &["alpha"], 200);
    create_test_note(&conn, Some("three"), Some("body"), "word", &["gamma"], 300);

    let tags = list_tags(&conn, String::new(), Some(2)).expect("list tags");

    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].label, "alpha");
    assert_eq!(tags[0].count, 2);
    assert_eq!(tags[1].label, "beta");
    assert_eq!(tags[1].count, 1);
}

#[test]
fn rename_tag_updates_note_json_and_tag_index() {
    let conn = open_test_connection();
    let note = create_test_note(
        &conn,
        Some("rename"),
        Some("body"),
        "word",
        &["old", "keep"],
        100,
    );

    rename_tag(&conn, "old".to_string(), "new".to_string()).expect("rename tag");

    assert_eq!(tag_count(&conn, "old"), 0);
    assert_eq!(tag_count(&conn, "new"), 1);
    assert_eq!(stored_tags_json(&conn, &note.id), vec!["new", "keep"]);
}

#[test]
fn delete_tag_updates_note_json_and_errors_when_missing() {
    let conn = open_test_connection();
    let note = create_test_note(
        &conn,
        Some("delete"),
        Some("body"),
        "word",
        &["removed", "keep"],
        100,
    );

    delete_tag(&conn, "removed".to_string()).expect("delete tag");

    assert_eq!(tag_count(&conn, "removed"), 0);
    assert_eq!(stored_tags_json(&conn, &note.id), vec!["keep"]);
    assert_eq!(
        delete_tag(&conn, "missing".to_string()).unwrap_err(),
        "errors.tagNotFound"
    );
}

fn stored_tags_json(conn: &Connection, note_id: &str) -> Vec<String> {
    let tags_json: String = conn
        .query_row(
            "SELECT tags_json FROM notes WHERE id = ?1",
            params![note_id],
            |row| row.get(0),
        )
        .expect("load tags json");

    serde_json::from_str(&tags_json).expect("parse tags json")
}
