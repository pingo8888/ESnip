<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import DeleteNoteConfirm from "./home/DeleteNoteConfirm.vue";
import HomeShellView from "./home/HomeShellView.vue";
import HomePageView from "./home/HomePageView.vue";
import type { Note, NoteInput, NoteUpdateInput } from "./home/noteTypes";
import { findNoteByTitle } from "./home/notesRepository";
import { useNoteCollection } from "./home/useNoteCollection";
import NewCardPage from "./new-card/NewCardPage.vue";
import SettingsPage from "./settings/SettingsPage.vue";

type WorkPage = "home" | "new-card" | "edit-card";
type ActivePage = WorkPage | "settings";
type QuickCapturePayload = {
  title?: string | null;
};

const activePage = ref<ActivePage>("home");
const settingsReturnPage = ref<WorkPage | null>(null);
const { notes, addNote, deleteNote, loadInitialNotes, searchQuery, setSearchQuery, updateNote } = useNoteCollection();
const editingNote = ref<Note | null>(null);
const deletingNote = ref<Note | null>(null);
const newCardInitialTitle = ref("");
const newCardDraftKey = ref(0);
const notesScrollEl = ref<HTMLElement | null>(null);
const notesScrollWidth = ref(0);
let resizeObserver: ResizeObserver | undefined;
let unlistenQuickCapture: UnlistenFn | undefined;

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

function showNewCardPage(initialTitle = "") {
  editingNote.value = null;
  newCardInitialTitle.value = initialTitle;
  newCardDraftKey.value += 1;
  activePage.value = "new-card";
}

function showHomePage() {
  editingNote.value = null;
  activePage.value = "home";
}

function showSettingsPage() {
  if (activePage.value !== "settings") {
    settingsReturnPage.value = activePage.value;
  }

  activePage.value = "settings";
}

function returnFromSettings() {
  activePage.value = settingsReturnPage.value ?? "home";
  settingsReturnPage.value = null;
}

function showEditCardPage(note: Note) {
  editingNote.value = note;
  activePage.value = "edit-card";
}

async function handleQuickCapture(payload: QuickCapturePayload) {
  const title = payload.title?.trim() ?? "";

  if (!title) {
    showNewCardPage();
    return;
  }

  try {
    const note = await findNoteByTitle(title);

    if (note) {
      showEditCardPage(note);
      return;
    }
  } catch (error) {
    console.error("Failed to find captured note title", error);
  }

  showNewCardPage(title);
}

async function saveNewNote(note: NoteInput) {
  await addNote(note);
  activePage.value = "home";

  void nextTick(updateNotesScrollWidth);
}

async function saveEditedNote(note: NoteInput | NoteUpdateInput) {
  if (!("id" in note)) {
    return;
  }

  await updateNote(note);
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

async function confirmDeleteNote() {
  if (!deletingNote.value) {
    return;
  }

  await deleteNote(deletingNote.value.id);
  deletingNote.value = null;

  void nextTick(updateNotesScrollWidth);
}

onMounted(() => {
  void loadInitialNotes();
  void listen<QuickCapturePayload>("quick-capture", (event) => {
    void handleQuickCapture(event.payload);
  }).then((unlisten) => {
    unlistenQuickCapture = unlisten;
  });
});

onUnmounted(() => {
  resizeObserver?.disconnect();
  unlistenQuickCapture?.();
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
      v-show="activePage === 'home'"
      :masonry-columns="masonryColumns"
      :result-count="notes.length"
      :search-query="searchQuery"
      @create-note="showNewCardPage"
      @delete-note="requestDeleteNote"
      @edit-note="showEditCardPage"
      @notes-scroll-ready="setNotesScrollElement"
      @open-settings="showSettingsPage"
      @update-search-query="setSearchQuery"
    />
    <NewCardPage
      v-if="activePage === 'new-card' || settingsReturnPage === 'new-card'"
      v-show="activePage === 'new-card'"
      :draft-key="newCardDraftKey"
      :active="activePage === 'new-card'"
      :initial-title="newCardInitialTitle"
      mode="create"
      @cancel="showHomePage"
      @open-settings="showSettingsPage"
      @save="saveNewNote"
    />
    <NewCardPage
      v-if="(activePage === 'edit-card' || settingsReturnPage === 'edit-card') && editingNote"
      v-show="activePage === 'edit-card'"
      mode="edit"
      :active="activePage === 'edit-card'"
      :initial-note="editingNote"
      @cancel="showHomePage"
      @open-settings="showSettingsPage"
      @save="saveEditedNote"
    />

    <SettingsPage v-if="activePage === 'settings'" @back="returnFromSettings" />

    <DeleteNoteConfirm
      v-if="deletingNote"
      :note="deletingNote"
      @cancel="cancelDeleteNote"
      @confirm="confirmDeleteNote"
    />
  </HomeShellView>
</template>
