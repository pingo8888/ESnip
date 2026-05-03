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
};

export type NotesPage = {
  notes: Note[];
  nextCursor: NotesCursor | null;
};

export async function listNotesPage(cursor: NotesCursor | null, limit = 80): Promise<NotesPage> {
  const page = await invoke<BackendNotesPage>("list_notes_page", {
    cursorCreatedAt: cursor?.createdAt ?? null,
    cursorId: cursor?.id ?? null,
    limit,
  });

  return {
    nextCursor: page.nextCursor,
    notes: page.notes.map(mapBackendNote),
  };
}

export async function searchNotes(query: string, limit = 80): Promise<NotesPage> {
  const page = await invoke<BackendNotesPage>("search_notes", {
    limit,
    query,
  });

  return {
    nextCursor: page.nextCursor,
    notes: page.notes.map(mapBackendNote),
  };
}

export async function findNoteByTitle(title: string): Promise<Note | null> {
  const note = await invoke<BackendNote | null>("find_note_by_title", { title });

  return note ? mapBackendNote(note) : null;
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
    time: formatRelativeTime(note.createdAt),
  };
}

function formatRelativeTime(timestamp: number) {
  const diffMs = Date.now() - timestamp;
  const minute = 60 * 1000;
  const hour = 60 * minute;
  const day = 24 * hour;

  if (diffMs < minute) {
    return "刚刚";
  }

  if (diffMs < hour) {
    return `${Math.floor(diffMs / minute)} 分钟前`;
  }

  if (diffMs < day) {
    return `${Math.floor(diffMs / hour)} 小时前`;
  }

  if (diffMs < 2 * day) {
    return "昨天";
  }

  return `${Math.floor(diffMs / day)} 天前`;
}
