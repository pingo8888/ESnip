pub(crate) mod connection;
mod kinds;
mod migration;
pub(crate) mod repository;
mod schema;
pub(crate) mod search;
mod tag_index;
pub(crate) mod tags;
pub(crate) mod types;

pub(crate) use connection::{
    init_connection, open_connection_at_dir, DB_FILE_NAME, DB_SHM_FILE_NAME, DB_WAL_FILE_NAME,
};
pub(crate) use repository::{
    create_note, delete_note, find_note_by_title, list_notes_page, update_note,
};
pub(crate) use search::search_notes;
pub(crate) use tags::{delete_tag, list_note_kind_counts, list_tags, rename_tag};
pub(crate) use types::{
    NoteDto, NoteKindCountDto, NotesPage, SaveNoteInput, TagSuggestionDto, UpdateNoteInput,
};

#[cfg(test)]
#[path = "../../tests/store/notes/test_utils.rs"]
pub(crate) mod test_utils;
