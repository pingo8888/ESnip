use super::*;

#[test]
fn normalize_note_kind_accepts_chinese_aliases_and_defaults_to_word() {
    assert_eq!(normalize_note_kind("词语"), "word");
    assert_eq!(normalize_note_kind("句子"), "sentence");
    assert_eq!(normalize_note_kind("段落"), "paragraph");
    assert_eq!(normalize_note_kind("unknown"), "word");
}

#[test]
fn normalize_search_note_kind_accepts_ascii_case_insensitive_aliases() {
    assert_eq!(normalize_search_note_kind("WORD").as_deref(), Some("word"));
    assert_eq!(
        normalize_search_note_kind("Sentence").as_deref(),
        Some("sentence")
    );
    assert_eq!(
        normalize_search_note_kind("PARAGRAPH").as_deref(),
        Some("paragraph")
    );
    assert_eq!(normalize_search_note_kind("unknown"), None);
}
