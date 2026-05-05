<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { ArrowBigUp, Plus, Search, Settings, X } from "lucide-vue-next";
import { useI18n } from "../../i18n";
import NoteCard from "./NoteCard.vue";
import NoteContextMenu from "./NoteContextMenu.vue";
import type { Note } from "./noteTypes";
import TagSuggestInput from "../shared/TagSuggestInput.vue";

const props = defineProps<{
  masonryColumns: Note[][];
  resultCount: number;
  searchQuery: string;
  updateAvailable: boolean;
  updateBusy: boolean;
}>();

const emit = defineEmits<{
  createNote: [];
  deleteNote: [note: Note];
  editNote: [note: Note];
  loadMoreNotes: [];
  notesScrollReady: [el: HTMLElement | null];
  openSettings: [];
  startUpdate: [];
  updateSearchQuery: [query: string];
}>();

const { t } = useI18n();
const contextMenu = ref<{
  note: Note;
  x: number;
  y: number;
} | null>(null);
const sectionTitle = computed(() =>
  props.searchQuery.trim()
    ? t("home.searchResults", { count: props.resultCount })
    : t("home.cardsCount", { count: props.resultCount }),
);

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

function handleNotesScroll(event: Event) {
  closeContextMenu();

  const el = event.currentTarget as HTMLElement;
  const distanceToBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
  if (distanceToBottom <= 320) {
    emit("loadMoreNotes");
  }
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
    <div class="app-toolbar" :aria-label="t('home.appActions')">
      <div class="app-actions app-actions--left">
        <button
          v-if="updateAvailable"
          type="button"
          tabindex="-1"
          :aria-label="t('home.updateAvailable')"
          :disabled="updateBusy"
          :title="t('home.updateAvailable')"
          @click="$emit('startUpdate')"
        >
          <ArrowBigUp aria-hidden="true" />
        </button>
      </div>
      <div class="app-actions">
        <button type="button" tabindex="-1" :aria-label="t('home.addNote')" :title="t('home.addNote')" @click="$emit('createNote')">
          <Plus aria-hidden="true" />
        </button>
        <button type="button" tabindex="-1" :aria-label="t('common.settings')" :title="t('common.settings')" @click="$emit('openSettings')">
          <Settings aria-hidden="true" />
        </button>
      </div>
    </div>

    <header class="hero">
      <label class="search-box" :aria-label="t('home.searchAria')">
        <span class="search-icon">
          <Search aria-hidden="true" />
        </span>
        <TagSuggestInput
          :model-value="searchQuery"
          :tag-prefixes="['!#', '#']"
          type="search"
          :placeholder="t('home.searchPlaceholder')"
          @update:model-value="emit('updateSearchQuery', $event)"
        />
        <button
          v-if="searchQuery"
          type="button"
          tabindex="-1"
          class="search-clear"
          :aria-label="t('home.clearSearch')"
          :title="t('home.clearSearch')"
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
        tabindex="-1"
        @scroll="handleNotesScroll"
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
