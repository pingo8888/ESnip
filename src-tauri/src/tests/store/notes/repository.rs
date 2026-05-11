use rusqlite::params;

use super::*;
use crate::store::notes::test_utils::{
    create_test_note, note_input, note_tags, open_test_connection, tag_count,
};

#[test]
fn create_note_normalizes_empty_fields_kind_and_tag_index() {
    let conn = open_test_connection();

    let note = create_note(
        &conn,
        note_input(
            Some("   "),
            Some("   "),
            "词语",
            &["  Alpha  ", "alpha", "  "],
        ),
    )
    .expect("create note");

    assert_eq!(note.title, None);
    assert_eq!(note.excerpt, None);
    assert_eq!(note.kind, "word");
    assert_eq!(note_tags(&conn, &note.id), vec!["Alpha"]);
}

#[test]
fn update_note_replaces_tag_index() {
    let conn = open_test_connection();
    let note = create_test_note(
        &conn,
        Some("before"),
        Some("body"),
        "sentence",
        &["old"],
        100,
    );

    let updated = update_note(
        &conn,
        UpdateNoteInput {
            id: note.id.clone(),
            title: Some("after".to_string()),
            excerpt: Some("next body".to_string()),
            kind: "段落".to_string(),
            tone: "ink".to_string(),
            tags: vec!["new".to_string()],
        },
    )
    .expect("update note");

    assert_eq!(updated.title.as_deref(), Some("after"));
    assert_eq!(updated.kind, "paragraph");
    assert_eq!(tag_count(&conn, "old"), 0);
    assert_eq!(note_tags(&conn, &note.id), vec!["new"]);
}

#[test]
fn delete_note_cascades_tag_index() {
    let conn = open_test_connection();
    let note = create_test_note(
        &conn,
        Some("delete me"),
        Some("body"),
        "word",
        &["gone"],
        100,
    );
    let rowid: i64 = conn
        .query_row(
            "SELECT rowid FROM notes WHERE id = ?1",
            params![note.id],
            |row| row.get(0),
        )
        .expect("load note rowid");

    delete_note(&conn, note.id.clone()).expect("delete note");

    let note_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE id = ?1",
            params![note.id],
            |row| row.get(0),
        )
        .expect("count note");
    let fts_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes_fts WHERE rowid = ?1",
            params![rowid],
            |row| row.get(0),
        )
        .expect("count FTS row");

    assert_eq!(note_count, 0);
    assert_eq!(tag_count(&conn, "gone"), 0);
    assert_eq!(fts_count, 0);
}

#[test]
fn list_notes_page_uses_cursor_and_counts_only_first_page() {
    let conn = open_test_connection();
    create_test_note(&conn, Some("old"), Some("body"), "word", &[], 100);
    create_test_note(&conn, Some("middle"), Some("body"), "word", &[], 200);
    create_test_note(&conn, Some("new"), Some("body"), "word", &[], 300);

    let first = list_notes_page(&conn, None, None, Some(2)).expect("first page");
    assert_eq!(first.total_count, 3);
    assert_eq!(first.notes.len(), 2);
    assert_eq!(first.notes[0].title.as_deref(), Some("new"));
    assert_eq!(first.notes[1].title.as_deref(), Some("middle"));

    let cursor = first.next_cursor.expect("next cursor");
    let second = list_notes_page(
        &conn,
        Some(cursor.updated_at),
        Some(cursor.id.clone()),
        Some(2),
    )
    .expect("second page");

    assert_eq!(second.total_count, -1);
    assert_eq!(second.notes.len(), 1);
    assert_eq!(second.notes[0].title.as_deref(), Some("old"));
}
