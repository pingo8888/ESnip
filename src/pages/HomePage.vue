<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { initI18n } from "../i18n";
import { useAppUpdater } from "../updates/useAppUpdater";
import DeleteNoteConfirm from "./home/DeleteNoteConfirm.vue";
import HomeShellView from "./home/HomeShellView.vue";
import HomePageView from "./home/HomePageView.vue";
import type { Note, NoteInput, NoteKind, NoteUpdateInput } from "./home/noteTypes";
import { computeColumnLayout } from "./home/cardColumns";
import { findNoteByTitle } from "./home/notesRepository";
import { useNoteCollection } from "./home/useNoteCollection";
import NewCardPage from "./new-card/NewCardPage.vue";
import SettingsPage from "./settings/SettingsPage.vue";

type WorkPage = "home" | "new-card" | "edit-card";
type ActivePage = WorkPage | "settings";
type QuickCapturePayload = {
  title?: string | null;
};
type QuickCaptureContentPayload = {
  content?: string | null;
  kind?: NoteKind | null;
};

const activePage = ref<ActivePage>("home");
const settingsReturnPage = ref<WorkPage | null>(null);
const {
  notes,
  addNote,
  deleteNote,
  isSearchQueryTooShort,
  loadInitialNotes,
  loadNextNotesPage,
  refreshNotes,
  searchQuery,
  setSearchQuery,
  totalCount,
  updateNote,
} = useNoteCollection();
const { checkAndInstallUpdate, checkForUpdate, hasUpdate, isBusy: isUpdateBusy } = useAppUpdater();
const editingNote = ref<Note | null>(null);
const deletingNote = ref<Note | null>(null);
const newCardInitialTitle = ref("");
const newCardInitialContent = ref("");
const newCardInitialKind = ref<NoteKind>("word");
const newCardDraftKey = ref(0);
const notesScrollEl = ref<HTMLElement | null>(null);
const notesScrollWidth = ref(0);
const alwaysOnTop = ref(false);
const noteKindCountsVersion = ref(0);
let resizeObserver: ResizeObserver | undefined;
let unlistenQuickCapture: UnlistenFn | undefined;
let unlistenQuickCaptureContent: UnlistenFn | undefined;
let lastColumnCount = 0;
let noteColumnAssignments = new Map<string, number>();

const columnLayout = computed(() => computeColumnLayout(notesScrollWidth.value));
const columnCount = computed(() => columnLayout.value.columnCount);

const masonryColumns = ref<Note[][]>([]);

function estimateNoteHeight(note: Note) {
  const titleLines = note.title ? Math.ceil(note.title.length / 14) : 0;
  const excerptLines = note.excerpt
    ? note.excerpt.split(/\r?\n/).reduce((total, paragraph) => total + Math.max(1, Math.ceil(paragraph.length / 42)), 0)
    : 0;
  const tagLines = Math.ceil(note.tags.join("").length / 12);

  return 78 + titleLines * 24 + excerptLines * 21 + tagLines * 18;
}

function updateMasonryColumns() {
  const count = columnCount.value;
  const columns = Array.from({ length: count }, () => [] as Note[]);
  const columnHeights = Array.from({ length: count }, () => 0);

  if (count !== lastColumnCount) {
    noteColumnAssignments = new Map();
    lastColumnCount = count;
  }

  const nextAssignments = new Map<string, number>();

  for (const note of notes.value) {
    let columnIndex = noteColumnAssignments.get(note.id);

    if (columnIndex === undefined || columnIndex >= count) {
      columnIndex = columnHeights.indexOf(Math.min(...columnHeights));
    }

    columns[columnIndex].push(note);
    columnHeights[columnIndex] += estimateNoteHeight(note);
    nextAssignments.set(note.id, columnIndex);
  }

  noteColumnAssignments = nextAssignments;
  masonryColumns.value = columns;
}

function resetMasonryAssignments() {
  noteColumnAssignments = new Map();
}

function isAppendedNotes(previousNotes: Note[] | undefined, nextNotes: Note[]) {
  if (!previousNotes || nextNotes.length < previousNotes.length) {
    return false;
  }

  return previousNotes.every((note, index) => nextNotes[index]?.id === note.id);
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
  const el = notesScrollEl.value;

  if (!el) {
    notesScrollWidth.value = 0;
    return;
  }

  const styles = window.getComputedStyle(el);
  const horizontalPadding = parseFloat(styles.paddingLeft) + parseFloat(styles.paddingRight);
  notesScrollWidth.value = Math.max(0, el.clientWidth - horizontalPadding);
}

function showNewCardPage(initialTitle = "", initialContent = "", initialKind: NoteKind = "word") {
  editingNote.value = null;
  newCardInitialTitle.value = initialTitle;
  newCardInitialContent.value = initialContent;
  newCardInitialKind.value = initialKind;
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
    showNewCardPage("", "", "word");
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

  showNewCardPage(title, "", "word");
}

function handleQuickCaptureContent(payload: QuickCaptureContentPayload) {
  const content = payload.content?.trim() ?? "";
  showNewCardPage("", content, payload.kind ?? "sentence");
}

async function saveNewNote(note: NoteInput) {
  await addNote(note);
  noteKindCountsVersion.value += 1;
  activePage.value = "home";

  void nextTick(updateNotesScrollWidth);
}

async function saveEditedNote(note: NoteInput | NoteUpdateInput) {
  const currentEditingNote = editingNote.value;

  if (!("id" in note) || !currentEditingNote) {
    return;
  }

  const previousKind = currentEditingNote.kind;
  await updateNote(note);
  if (previousKind !== note.kind) {
    noteKindCountsVersion.value += 1;
  }
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
  noteKindCountsVersion.value += 1;
  deletingNote.value = null;

  void nextTick(updateNotesScrollWidth);
}

async function handleDataDirChanged() {
  await loadInitialNotes();
  noteKindCountsVersion.value += 1;
  void nextTick(updateNotesScrollWidth);
}

async function handleTagsChanged() {
  await refreshNotes();
  void nextTick(updateNotesScrollWidth);
}

onMounted(() => {
  void initI18n();
  void loadInitialNotes();
  void loadAlwaysOnTopState();
  void checkForUpdate({ silent: true });
  void listen<QuickCapturePayload>("quick-capture", (event) => {
    void handleQuickCapture(event.payload);
  }).then((unlisten) => {
    unlistenQuickCapture = unlisten;
  });
  void listen<QuickCaptureContentPayload>("quick-capture-content", (event) => {
    handleQuickCaptureContent(event.payload);
  }).then((unlisten) => {
    unlistenQuickCaptureContent = unlisten;
  });
});

onUnmounted(() => {
  resizeObserver?.disconnect();
  unlistenQuickCapture?.();
  unlistenQuickCaptureContent?.();
});

watch(
  notes,
  (nextNotes, previousNotes) => {
    if (!isAppendedNotes(previousNotes, nextNotes)) {
      resetMasonryAssignments();
    }

    updateMasonryColumns();
  },
  { immediate: true },
);

watch(columnCount, () => {
  updateMasonryColumns();
});

async function minimizeWindow() {
  await getCurrentWindow().minimize();
}

async function loadAlwaysOnTopState() {
  try {
    alwaysOnTop.value = await getCurrentWindow().isAlwaysOnTop();
  } catch (error) {
    console.error("Failed to load always-on-top state", error);
  }
}

async function toggleAlwaysOnTop() {
  const nextAlwaysOnTop = !alwaysOnTop.value;

  try {
    await getCurrentWindow().setAlwaysOnTop(nextAlwaysOnTop);
    alwaysOnTop.value = nextAlwaysOnTop;
  } catch (error) {
    console.error("Failed to toggle always-on-top state", error);
  }
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
    :always-on-top="alwaysOnTop"
    @close-window="closeWindow"
    @minimize-window="minimizeWindow"
    @titlebar-mouse-down="handleTitlebarMouseDown"
    @toggle-always-on-top="toggleAlwaysOnTop"
    @toggle-maximize-window="toggleMaximizeWindow"
  >
    <HomePageView
      v-show="activePage === 'home'"
      :masonry-columns="masonryColumns"
      :note-kind-counts-version="noteKindCountsVersion"
      :column-width="columnLayout.cardWidth"
      :is-search-query-too-short="isSearchQueryTooShort"
      :result-count="totalCount"
      :search-query="searchQuery"
      :update-available="hasUpdate"
      :update-busy="isUpdateBusy"
      @create-note="showNewCardPage"
      @delete-note="requestDeleteNote"
      @edit-note="showEditCardPage"
      @load-more-notes="loadNextNotesPage"
      @notes-scroll-ready="setNotesScrollElement"
      @open-settings="showSettingsPage"
      @start-update="checkAndInstallUpdate"
      @update-search-query="setSearchQuery"
    />
    <NewCardPage
      v-if="activePage === 'new-card' || settingsReturnPage === 'new-card'"
      v-show="activePage === 'new-card'"
      :draft-key="newCardDraftKey"
      :active="activePage === 'new-card'"
      :initial-title="newCardInitialTitle"
      :initial-content="newCardInitialContent"
      :initial-kind="newCardInitialKind"
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

    <SettingsPage
      v-if="activePage === 'settings'"
      @back="returnFromSettings"
      @data-dir-changed="handleDataDirChanged"
      @tags-changed="handleTagsChanged"
    />

    <DeleteNoteConfirm
      v-if="deletingNote"
      :note="deletingNote"
      @cancel="cancelDeleteNote"
      @confirm="confirmDeleteNote"
    />
  </HomeShellView>
</template>
