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
  const totalCount = ref(0);
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
      totalCount.value = page.totalCount;
    } catch (caught) {
      error.value = caught instanceof Error ? caught.message : String(caught);
    } finally {
      isLoading.value = false;
    }
  }

  async function refreshNotes() {
    if (searchQuery.value.trim()) {
      await applySearchQuery(searchQuery.value);
      return;
    }

    await loadInitialNotes();
  }

  async function loadNextNotesPage() {
    if (isLoading.value) {
      return;
    }

    const requestId = requestSerial;
    const trimmedQuery = searchQuery.value.trim();
    const shouldUseSearch = canRunSearch(trimmedQuery);
    if (trimmedQuery) {
      if (!shouldUseSearch || !nextCursor.value || notes.value.length >= totalCount.value) {
        return;
      }
    } else if (!nextCursor.value) {
      return;
    }

    isLoading.value = true;
    error.value = null;

    try {
      const page = shouldUseSearch
        ? await searchNotes(trimmedQuery, 80, nextCursor.value)
        : await listNotesPage(nextCursor.value);

      if (requestId !== requestSerial) {
        return;
      }

      notes.value = [...notes.value, ...page.notes];
      nextCursor.value = page.nextCursor;
      if (page.totalCount >= 0) {
        totalCount.value = page.totalCount;
      }
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

  async function addNote(input: NoteInput) {
    const note = await createPersistedNote(input);

    if (searchQuery.value.trim()) {
      await applySearchQuery(searchQuery.value);
      return note;
    }

    notes.value = [note, ...notes.value.filter((item) => item.id !== note.id)];
    totalCount.value += 1;
    return note;
  }

  async function updateNote(input: NoteUpdateInput) {
    const note = await updatePersistedNote(input);

    if (searchQuery.value.trim()) {
      await applySearchQuery(searchQuery.value);
      return note;
    }

    const filtered = notes.value.filter((item) => item.id !== note.id);

    notes.value = [note, ...filtered];
    return note;
  }

  async function deleteNote(id: string) {
    await deletePersistedNote(id);

    if (searchQuery.value.trim()) {
      await applySearchQuery(searchQuery.value);
      return;
    }

    notes.value = notes.value.filter((note) => note.id !== id);
    totalCount.value = Math.max(0, totalCount.value - 1);
  }

  function setSearchQuery(query: string) {
    searchQuery.value = query;
    requestSerial += 1;

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
      if (trimmedQuery && !canRunSearch(trimmedQuery)) {
        notes.value = [];
        nextCursor.value = null;
        totalCount.value = 0;
        return;
      }

      const page = trimmedQuery ? await searchNotes(trimmedQuery) : await listNotesPage(null);

      if (requestId !== requestSerial) {
        return;
      }

      notes.value = page.notes;
      nextCursor.value = page.nextCursor;
      totalCount.value = page.totalCount;
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
    refreshNotes,
    searchQuery,
    setSearchQuery,
    totalCount,
    updateNote,
  };
}

function canRunSearch(query: string) {
  const parsed = parseSearchQuery(query);

  if (parsed.textTerms.length === 0) {
    return parsed.hasFilters;
  }

  return parsed.hasFilters || parsed.textTerms.every(isSearchableTextTerm);
}

function parseSearchQuery(query: string) {
  const terms = query.split(/\s+/).filter(Boolean);
  const textTerms: string[] = [];
  let hasFilters = false;

  for (const term of terms) {
    if (term.startsWith("#") || term.startsWith("!#") || term.startsWith("@") || term.startsWith("!@")) {
      hasFilters = true;
      continue;
    }

    textTerms.push(term);
  }

  return {
    hasFilters,
    textTerms,
  };
}

function isSearchableTextTerm(value: string) {
  return [...value].length >= 3 || countCjkCharacters(value) >= 2;
}

function countCjkCharacters(value: string) {
  return [...value].filter(isCjkCharacter).length;
}

function isCjkCharacter(char: string) {
  const codePoint = char.codePointAt(0) ?? 0;

  return (
    (codePoint >= 0x3400 && codePoint <= 0x9fff) ||
    (codePoint >= 0xf900 && codePoint <= 0xfaff) ||
    (codePoint >= 0x3040 && codePoint <= 0x30ff) ||
    (codePoint >= 0xac00 && codePoint <= 0xd7af)
  );
}
