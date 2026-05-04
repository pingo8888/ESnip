pub(crate) fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim().to_string();

        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub(crate) fn clean_captured_text(text: String) -> Option<String> {
    let cleaned = text.trim().to_string();

    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

pub(crate) fn build_fts_query(value: &str) -> Option<String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return None;
    }

    let terms = trimmed
        .split_whitespace()
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect::<Vec<_>>();

    if terms.is_empty() {
        Some(format!("\"{}\"", trimmed.replace('"', "\"\"")))
    } else {
        Some(terms.join(" AND "))
    }
}

pub(crate) fn build_like_pattern(value: &str) -> String {
    let escaped = value
        .trim()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");

    format!("%{}%", escaped)
}
