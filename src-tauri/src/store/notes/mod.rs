pub(crate) mod connection;
mod kinds;
mod migration;
pub(crate) mod repository;
mod schema;
pub(crate) mod search;
pub(crate) mod tags;
pub(crate) mod types;

pub(crate) use connection::{
    init_connection, open_connection_at_dir, DB_FILE_NAME, DB_SHM_FILE_NAME, DB_WAL_FILE_NAME,
};
pub(crate) use repository::{
    create_note, delete_note, find_note_by_title, list_notes_page, update_note,
};
pub(crate) use search::search_notes;
pub(crate) use tags::list_tags;
pub(crate) use types::{NoteDto, NotesPage, SaveNoteInput, UpdateNoteInput};
