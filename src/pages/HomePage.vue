<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import HomeShellView from "./home/HomeShellView.vue";
import { testNotes, type Note } from "./home/notes.fixture";

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

  for (const note of testNotes) {
    const shortestColumnIndex = columnHeights.indexOf(Math.min(...columnHeights));

    columns[shortestColumnIndex].push(note);
    columnHeights[shortestColumnIndex] += estimateNoteHeight(note);
  }

  return columns;
});

function estimateNoteHeight(note: Note) {
  const titleLines = note.title ? Math.ceil(note.title.length / 14) : 0;
  const excerptLines = note.excerpt ? Math.ceil(note.excerpt.length / 42) : 0;
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
    :masonry-columns="masonryColumns"
    @close-window="closeWindow"
    @minimize-window="minimizeWindow"
    @notes-scroll-ready="setNotesScrollElement"
    @titlebar-mouse-down="handleTitlebarMouseDown"
    @toggle-maximize-window="toggleMaximizeWindow"
  />
</template>
