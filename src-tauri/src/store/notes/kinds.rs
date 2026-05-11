struct NoteKindDef {
    value: &'static str,
    aliases: &'static [&'static str],
}

const NOTE_KIND_DEFS: &[NoteKindDef] = &[
    NoteKindDef {
        value: "word",
        aliases: &["word", "词语"],
    },
    NoteKindDef {
        value: "sentence",
        aliases: &["sentence", "句子"],
    },
    NoteKindDef {
        value: "paragraph",
        aliases: &["paragraph", "段落"],
    },
];

pub(super) fn normalize_search_note_kind(kind: &str) -> Option<String> {
    find_note_kind_value(kind).map(str::to_string)
}

pub(super) fn normalize_note_kind(kind: &str) -> &'static str {
    find_note_kind_value(kind).unwrap_or("word")
}

fn find_note_kind_value(kind: &str) -> Option<&'static str> {
    let cleaned_kind = kind.trim();

    NOTE_KIND_DEFS
        .iter()
        .find(|definition| {
            definition.value.eq_ignore_ascii_case(cleaned_kind)
                || definition
                    .aliases
                    .iter()
                    .any(|alias| matches_kind_alias(alias, cleaned_kind))
        })
        .map(|definition| definition.value)
}

fn matches_kind_alias(alias: &str, kind: &str) -> bool {
    if alias.is_ascii() && kind.is_ascii() {
        alias.eq_ignore_ascii_case(kind)
    } else {
        alias == kind
    }
}

#[cfg(test)]
#[path = "../../tests/store/notes/kinds.rs"]
mod tests;
