import { invoke } from "@tauri-apps/api/core";
import type { Note, NoteInput, NotesCursor, NoteUpdateInput } from "./noteTypes";

type BackendNote = {
  id: string;
  title?: string | null;
  excerpt?: string | null;
  kind: Note["kind"];
  tone: Note["tone"];
  tags: string[];
  createdAt: number;
  updatedAt: number;
};

type BackendNotesPage = {
  notes: BackendNote[];
  nextCursor: NotesCursor | null;
  totalCount: number;
};

type BackendTagSuggestion = {
  label: string;
  count: number;
};

type BackendNoteKindCount = {
  value: Note["kind"];
  count: number;
};

export type NotesPage = {
  notes: Note[];
  nextCursor: NotesCursor | null;
  totalCount: number;
};

export type SuggestionItem = {
  label: string;
  count: number;
};

export async function listNotesPage(cursor: NotesCursor | null, limit = 80): Promise<NotesPage> {
  const page = await invoke<BackendNotesPage>("list_notes_page", {
    cursorUpdatedAt: cursor?.updatedAt ?? null,
    cursorId: cursor?.id ?? null,
    limit,
  });

  return {
    nextCursor: page.nextCursor,
    notes: page.notes.map(mapBackendNote),
    totalCount: page.totalCount,
  };
}

export async function searchNotes(query: string, limit = 80, offset = 0): Promise<NotesPage> {
  const page = await invoke<BackendNotesPage>("search_notes", {
    limit,
    offset,
    query,
  });

  return {
    nextCursor: page.nextCursor,
    notes: page.notes.map(mapBackendNote),
    totalCount: page.totalCount,
  };
}

export async function findNoteByTitle(title: string): Promise<Note | null> {
  const note = await invoke<BackendNote | null>("find_note_by_title", { title });

  return note ? mapBackendNote(note) : null;
}

export async function listTags(prefix: string, limit = 50): Promise<SuggestionItem[]> {
  return invoke<BackendTagSuggestion[]>("list_tags", { limit, prefix });
}

export async function listNoteKindCounts(): Promise<BackendNoteKindCount[]> {
  return invoke<BackendNoteKindCount[]>("list_note_kind_counts");
}

export async function createNote(input: NoteInput): Promise<Note> {
  return mapBackendNote(await invoke<BackendNote>("create_note", { input }));
}

export async function updateNote(input: NoteUpdateInput): Promise<Note> {
  return mapBackendNote(await invoke<BackendNote>("update_note", { input }));
}

export async function deleteNote(id: string): Promise<void> {
  await invoke("delete_note", { id });
}

function mapBackendNote(note: BackendNote): Note {
  return {
    id: note.id,
    title: note.title || undefined,
    excerpt: note.excerpt || undefined,
    kind: note.kind,
    tone: note.tone,
    tags: note.tags,
    createdAt: note.createdAt,
    updatedAt: note.updatedAt,
    time: "",
  };
}
