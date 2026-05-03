<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import DeleteNoteConfirm from "./home/DeleteNoteConfirm.vue";
import HomeShellView from "./home/HomeShellView.vue";
import HomePageView from "./home/HomePageView.vue";
import { testNotes, type Note } from "./home/notes.fixture";
import { useNoteCollection } from "./home/useNoteCollection";
import NewCardPage from "./new-card/NewCardPage.vue";

type ActivePage = "home" | "new-card" | "edit-card";

const activePage = ref<ActivePage>("home");
const { notes, addNote, deleteNote, updateNote } = useNoteCollection(testNotes);
const editingNote = ref<Note | null>(null);
const deletingNote = ref<Note | null>(null);
const notesScrollEl = ref<HTMLElement | null>(null);
const notesScrollWidth = ref(0);
let resizeObserver: ResizeObserver | undefined;

const columnCount = computed(() => {
  const width = notesScrollWidth.value;

  if (width < 620) {
    return 1;
  }

  const minColumnWidth = width < 920 ? 190 : 210;
  const columnGap = 14;

  return Math.max(1, Math.floor((width + columnGap) / (minColumnWidth + columnGap)));
});

const masonryColumns = computed(() => {
  const columns = Array.from({ length: columnCount.value }, () => [] as Note[]);
  const columnHeights = Array.from({ length: columnCount.value }, () => 0);

  for (const note of notes.value) {
    const shortestColumnIndex = columnHeights.indexOf(Math.min(...columnHeights));

    columns[shortestColumnIndex].push(note);
    columnHeights[shortestColumnIndex] += estimateNoteHeight(note);
  }

  return columns;
});

function estimateNoteHeight(note: Note) {
  const titleLines = note.title ? Math.ceil(note.title.length / 14) : 0;
  const excerptLines = note.excerpt
    ? note.excerpt.split(/\r?\n/).reduce((total, paragraph) => total + Math.max(1, Math.ceil(paragraph.length / 42)), 0)
    : 0;
  const tagLines = Math.ceil(note.tags.join("").length / 12);

  return 78 + titleLines * 24 + excerptLines * 21 + tagLines * 18;
}

function setNotesScrollElement(el: HTMLElement | null) {
  if (notesScrollEl.value === el) {
    return;
  }

  resizeObserver?.disconnect();
  notesScrollEl.value = el;
  updateNotesScrollWidth();

  if (!el) {
    return;
  }

  resizeObserver = new ResizeObserver(updateNotesScrollWidth);
  resizeObserver.observe(el);

  void nextTick(updateNotesScrollWidth);
}

function updateNotesScrollWidth() {
  notesScrollWidth.value = notesScrollEl.value?.clientWidth ?? 0;
}

function showNewCardPage() {
  editingNote.value = null;
  activePage.value = "new-card";
}

function showHomePage() {
  editingNote.value = null;
  activePage.value = "home";
}

function showEditCardPage(note: Note) {
  editingNote.value = note;
  activePage.value = "edit-card";
}

function saveNewNote(note: Note) {
  addNote(note);
  activePage.value = "home";

  void nextTick(updateNotesScrollWidth);
}

function saveEditedNote(note: Note) {
  updateNote(note);
  editingNote.value = null;
  activePage.value = "home";

  void nextTick(updateNotesScrollWidth);
}

function requestDeleteNote(note: Note) {
  deletingNote.value = note;
}

function cancelDeleteNote() {
  deletingNote.value = null;
}

function confirmDeleteNote() {
  if (!deletingNote.value) {
    return;
  }

  deleteNote(deletingNote.value.id);
  deletingNote.value = null;

  void nextTick(updateNotesScrollWidth);
}

onUnmounted(() => {
  resizeObserver?.disconnect();
});

async function minimizeWindow() {
  await getCurrentWindow().minimize();
}

async function toggleMaximizeWindow() {
  await getCurrentWindow().toggleMaximize();
}

async function closeWindow() {
  await getCurrentWindow().close();
}

async function handleTitlebarMouseDown(event: MouseEvent) {
  if (event.button !== 0) {
    return;
  }

  const appWindow = getCurrentWindow();

  if (event.detail === 2) {
    await appWindow.toggleMaximize();
    return;
  }

  await appWindow.startDragging();
}
</script>

<template>
  <HomeShellView
    @close-window="closeWindow"
    @minimize-window="minimizeWindow"
    @titlebar-mouse-down="handleTitlebarMouseDown"
    @toggle-maximize-window="toggleMaximizeWindow"
  >
    <HomePageView
      v-if="activePage === 'home'"
      :masonry-columns="masonryColumns"
      @create-note="showNewCardPage"
      @delete-note="requestDeleteNote"
      @edit-note="showEditCardPage"
      @notes-scroll-ready="setNotesScrollElement"
    />
    <NewCardPage v-else-if="activePage === 'new-card'" mode="create" @cancel="showHomePage" @save="saveNewNote" />
    <NewCardPage
      v-else
      mode="edit"
      :initial-note="editingNote"
      @cancel="showHomePage"
      @save="saveEditedNote"
    />

    <DeleteNoteConfirm
      v-if="deletingNote"
      :note="deletingNote"
      @cancel="cancelDeleteNote"
      @confirm="confirmDeleteNote"
    />
  </HomeShellView>
</template>
