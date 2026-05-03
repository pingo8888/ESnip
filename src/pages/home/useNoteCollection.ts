import { ref } from "vue";
import type { Note, NoteInput, NotesCursor, NoteUpdateInput } from "./noteTypes";
import {
  createNote as createPersistedNote,
  deleteNote as deletePersistedNote,
  listNotesPage,
  searchNotes,
  updateNote as updatePersistedNote,
} from "./notesRepository";

export function useNoteCollection() {
  const notes = ref<Note[]>([]);
  const nextCursor = ref<NotesCursor | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const searchQuery = ref("");
  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  let requestSerial = 0;

  async function loadInitialNotes() {
    isLoading.value = true;
    error.value = null;
    const requestId = ++requestSerial;

    try {
      const page = await listNotesPage(null);

      if (requestId !== requestSerial) {
        return;
      }

      notes.value = page.notes;
      nextCursor.value = page.nextCursor;
    } catch (caught) {
      error.value = caught instanceof Error ? caught.message : String(caught);
    } finally {
      isLoading.value = false;
    }
  }

  async function loadNextNotesPage() {
    if (searchQuery.value.trim() || !nextCursor.value || isLoading.value) {
      return;
    }

    isLoading.value = true;
    error.value = null;

    try {
      const page = await listNotesPage(nextCursor.value);

      notes.value = [...notes.value, ...page.notes];
      nextCursor.value = page.nextCursor;
    } catch (caught) {
      error.value = caught instanceof Error ? caught.message : String(caught);
    } finally {
      isLoading.value = false;
    }
  }

  async function addNote(input: NoteInput) {
    const note = await createPersistedNote(input);

    notes.value = [note, ...notes.value];
    return note;
  }

  async function updateNote(input: NoteUpdateInput) {
    const note = await updatePersistedNote(input);
    const index = notes.value.findIndex((item) => item.id === note.id);

    notes.value =
      index >= 0
        ? notes.value.map((item) => (item.id === note.id ? note : item))
        : [note, ...notes.value];
    return note;
  }

  async function deleteNote(id: string) {
    await deletePersistedNote(id);
    notes.value = notes.value.filter((note) => note.id !== id);
  }

  function setSearchQuery(query: string) {
    searchQuery.value = query;

    if (searchTimer) {
      clearTimeout(searchTimer);
    }

    searchTimer = setTimeout(() => {
      void applySearchQuery(query);
    }, 220);
  }

  async function applySearchQuery(query: string) {
    const trimmedQuery = query.trim();
    const requestId = ++requestSerial;

    isLoading.value = true;
    error.value = null;

    try {
      const page = trimmedQuery ? await searchNotes(trimmedQuery) : await listNotesPage(null);

      if (requestId !== requestSerial) {
        return;
      }

      notes.value = page.notes;
      nextCursor.value = trimmedQuery ? null : page.nextCursor;
    } catch (caught) {
      if (requestId !== requestSerial) {
        return;
      }

      error.value = caught instanceof Error ? caught.message : String(caught);
    } finally {
      if (requestId === requestSerial) {
        isLoading.value = false;
      }
    }
  }

  return {
    addNote,
    deleteNote,
    error,
    isLoading,
    loadInitialNotes,
    loadNextNotesPage,
    nextCursor,
    notes,
    searchQuery,
    setSearchQuery,
    updateNote,
  };
}
