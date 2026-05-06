use tauri::State;

use crate::{
    app::state::DbState,
    store::notes::{
        NoteDto, NoteKindCountDto, NotesPage, SaveNoteInput, TagSuggestionDto, UpdateNoteInput,
    },
};

#[tauri::command]
pub(crate) fn list_notes_page(
    state: State<'_, DbState>,
    cursor_updated_at: Option<i64>,
    cursor_id: Option<String>,
    limit: Option<i64>,
) -> Result<NotesPage, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;

    crate::store::notes::list_notes_page(&conn, cursor_updated_at, cursor_id, limit)
}

#[tauri::command]
pub(crate) fn search_notes(
    state: State<'_, DbState>,
    query: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<NotesPage, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;

    crate::store::notes::search_notes(&conn, query, limit, offset)
}

#[tauri::command]
pub(crate) fn find_note_by_title(
    state: State<'_, DbState>,
    title: String,
) -> Result<Option<NoteDto>, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;

    crate::store::notes::find_note_by_title(&conn, title)
}

#[tauri::command]
pub(crate) fn list_tags(
    state: State<'_, DbState>,
    prefix: String,
    limit: Option<i64>,
) -> Result<Vec<TagSuggestionDto>, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;

    crate::store::notes::list_tags(&conn, prefix, limit)
}

#[tauri::command]
pub(crate) fn list_note_kind_counts(
    state: State<'_, DbState>,
) -> Result<Vec<NoteKindCountDto>, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;

    crate::store::notes::list_note_kind_counts(&conn)
}

#[tauri::command]
pub(crate) fn create_note(
    state: State<'_, DbState>,
    input: SaveNoteInput,
) -> Result<NoteDto, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;

    crate::store::notes::create_note(&conn, input)
}

#[tauri::command]
pub(crate) fn update_note(
    state: State<'_, DbState>,
    input: UpdateNoteInput,
) -> Result<NoteDto, String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;

    crate::store::notes::update_note(&conn, input)
}

#[tauri::command]
pub(crate) fn delete_note(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;

    crate::store::notes::delete_note(&conn, id)
}
