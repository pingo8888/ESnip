<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { Plus, Search, Settings, X } from "lucide-vue-next";
import NoteCard from "./NoteCard.vue";
import NoteContextMenu from "./NoteContextMenu.vue";
import type { Note } from "./noteTypes";
import TagSuggestInput from "../shared/TagSuggestInput.vue";

const props = defineProps<{
  masonryColumns: Note[][];
  resultCount: number;
  searchQuery: string;
}>();

const emit = defineEmits<{
  createNote: [];
  deleteNote: [note: Note];
  editNote: [note: Note];
  notesScrollReady: [el: HTMLElement | null];
  openSettings: [];
  updateSearchQuery: [query: string];
}>();

const contextMenu = ref<{
  note: Note;
  x: number;
  y: number;
} | null>(null);
const sectionTitle = computed(() => (props.searchQuery.trim() ? `${props.resultCount}条 搜索结果` : `${props.resultCount}张 摘录卡片`));

function openContextMenu(event: MouseEvent, note: Note) {
  const menuWidth = 198;
  const menuHeight = 78;
  const padding = 8;

  contextMenu.value = {
    note,
    x: Math.max(padding, Math.min(event.clientX, window.innerWidth - menuWidth - padding)),
    y: Math.max(padding, Math.min(event.clientY, window.innerHeight - menuHeight - padding)),
  };
}

function closeContextMenu() {
  contextMenu.value = null;
}

function editContextNote() {
  if (!contextMenu.value) {
    return;
  }

  emit("editNote", contextMenu.value.note);
  closeContextMenu();
}

function deleteContextNote() {
  if (!contextMenu.value) {
    return;
  }

  emit("deleteNote", contextMenu.value.note);
  closeContextMenu();
}

function handleGlobalPointerDown() {
  closeContextMenu();
}

function handleGlobalKeyDown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    closeContextMenu();
  }
}

function clearSearch() {
  emit("updateSearchQuery", "");
}

onMounted(() => {
  window.addEventListener("pointerdown", handleGlobalPointerDown);
  window.addEventListener("keydown", handleGlobalKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("pointerdown", handleGlobalPointerDown);
  window.removeEventListener("keydown", handleGlobalKeyDown);
});
</script>

<template>
  <main class="commonplace-shell">
    <div class="app-toolbar" aria-label="应用操作">
      <div class="app-actions">
        <button type="button" aria-label="添加词条" title="添加词条" @click="$emit('createNote')">
          <Plus aria-hidden="true" />
        </button>
        <button type="button" aria-label="设置" title="设置" @click="$emit('openSettings')">
          <Settings aria-hidden="true" />
        </button>
      </div>
    </div>

    <header class="hero">
      <label class="search-box" aria-label="搜索笔记">
        <span class="search-icon">
          <Search aria-hidden="true" />
        </span>
        <TagSuggestInput
          :model-value="searchQuery"
          type="search"
          placeholder="搜索你的摘录..."
          @update:model-value="emit('updateSearchQuery', $event)"
        />
        <button
          v-if="searchQuery"
          type="button"
          class="search-clear"
          aria-label="清空搜索"
          title="清空搜索"
          @click="clearSearch"
        >
          <X aria-hidden="true" />
        </button>
        <kbd>Enter</kbd>
      </label>
    </header>

    <section class="notes-section" aria-labelledby="recent-heading">
      <h2 id="recent-heading">{{ sectionTitle }}</h2>

      <div
        :ref="(el) => $emit('notesScrollReady', el as HTMLElement | null)"
        class="notes-scroll"
        @scroll="closeContextMenu"
      >
        <div class="notes-columns">
          <div v-for="(column, columnIndex) in masonryColumns" :key="columnIndex" class="notes-column">
            <NoteCard
              v-for="note in column"
              :key="note.id"
              context-menu-enabled
              :note="note"
              @open-context-menu="openContextMenu"
            />
          </div>
        </div>
      </div>
    </section>

    <NoteContextMenu
      v-if="contextMenu"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @close="closeContextMenu"
      @delete="deleteContextNote"
      @edit="editContextNote"
    />
  </main>
</template>

<style scoped src="./HomePageView.scoped.css"></style>
