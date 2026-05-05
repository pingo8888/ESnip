use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NoteDto {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) excerpt: Option<String>,
    pub(crate) kind: String,
    pub(crate) tone: String,
    pub(crate) tags: Vec<String>,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NotesPage {
    pub(crate) notes: Vec<NoteDto>,
    pub(crate) next_cursor: Option<NotesCursor>,
    pub(crate) total_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NotesCursor {
    pub(crate) updated_at: i64,
    pub(crate) id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaveNoteInput {
    pub(crate) title: Option<String>,
    pub(crate) excerpt: Option<String>,
    pub(crate) kind: String,
    pub(crate) tone: String,
    pub(crate) tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateNoteInput {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) excerpt: Option<String>,
    pub(crate) kind: String,
    pub(crate) tone: String,
    pub(crate) tags: Vec<String>,
}
