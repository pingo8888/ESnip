import { ref } from "vue";
import type { Note } from "./notes.fixture";

export function useNoteCollection(initialNotes: Note[]) {
  const notes = ref<Note[]>([...initialNotes]);

  function addNote(note: Note) {
    notes.value = [note, ...notes.value];
  }

  function updateNote(note: Note) {
    notes.value = notes.value.map((item) => (item.id === note.id ? note : item));
  }

  function deleteNote(id: string) {
    notes.value = notes.value.filter((note) => note.id !== id);
  }

  return {
    addNote,
    deleteNote,
    notes,
    updateNote,
  };
}
