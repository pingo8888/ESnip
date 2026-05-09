<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ArrowBigUp, Plus, Search, Settings, X } from "lucide-vue-next";
import { useI18n } from "../../i18n";
import { noteKindDefinitions } from "../../notes/noteKinds";
import { useAppSettings } from "../../settings/useAppSettings";
import type { SearchEngine } from "../../settings/appSettingsRepository";
import { updateMeasuredHomeCardSize } from "./homeCardMetrics";
import NoteCard from "./NoteCard.vue";
import NoteContextMenu from "./NoteContextMenu.vue";
import type { Note } from "./noteTypes";
import TagSuggestInput from "../shared/TagSuggestInput.vue";
import { listNoteKindCounts, type SuggestionItem } from "./notesRepository";
import { parseHighlightTerms } from "./searchHighlight";

const props = defineProps<{
  columnWidth: number;
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
const { searchEngine } = useAppSettings();
const contextMenu = ref<{
  note: Note;
  x: number;
  y: number;
} | null>(null);
const hasRequestedNearBottomLoad = ref(false);
const notesScrollEl = ref<HTMLElement | null>(null);
const noteKindCounts = ref<Record<string, number>>({});
const sectionTitle = computed(() =>
  props.searchQuery.trim()
    ? t("home.searchResults", { count: props.resultCount })
    : t("home.cardsCount", { count: props.resultCount }),
);
const highlightTerms = computed(() => parseHighlightTerms(props.searchQuery));
const noteKindSearchSuggestions = computed<SuggestionItem[]>(() =>
  noteKindDefinitions.map((definition) => ({
    count: noteKindCounts.value[definition.value] ?? 0,
    label: t(definition.labelKey),
  })),
);
const searchStaticSuggestions = computed(() => ({
  "!@": noteKindSearchSuggestions.value,
  "@": noteKindSearchSuggestions.value,
}));

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

async function searchNoteTitle(title: string) {
  const keyword = title.trim();
  if (!keyword) {
    return;
  }

  try {
    await openUrl(buildSearchUrl(searchEngine.value, keyword));
  } catch (error) {
    console.error("Failed to open search URL", error);
  }
}

function buildSearchUrl(engine: SearchEngine, keyword: string) {
  const query = encodeURIComponent(keyword);

  switch (engine) {
    case "baidu":
      return `https://www.baidu.com/s?wd=${query}`;
    case "bing":
      return `https://www.bing.com/search?q=${query}`;
    case "google":
    default:
      return `https://www.google.com/search?q=${query}`;
  }
}

async function refreshNoteKindCounts() {
  try {
    const counts = await listNoteKindCounts();
    noteKindCounts.value = Object.fromEntries(counts.map((item) => [item.value, item.count]));
  } catch (error) {
    console.error("Failed to load note kind counts", error);
  }
}

function handleNotesScroll(event: Event) {
  closeContextMenu();

  const el = event.currentTarget as HTMLElement;
  const distanceToBottom = el.scrollHeight - el.scrollTop - el.clientHeight;

  if (distanceToBottom > 320) {
    hasRequestedNearBottomLoad.value = false;
    return;
  }

  if (hasRequestedNearBottomLoad.value) {
    return;
  }

  if (distanceToBottom <= 320) {
    hasRequestedNearBottomLoad.value = true;
    emit("loadMoreNotes");
  }
}

function setNotesScrollRef(el: HTMLElement | null) {
  notesScrollEl.value = el;
  emit("notesScrollReady", el);
  void nextTick(measureHomeCardSize);
}

function measureHomeCardSize() {
  const card = notesScrollEl.value?.querySelector<HTMLElement>(".note-card");

  if (!card) {
    return;
  }

  const rect = card.getBoundingClientRect();
  updateMeasuredHomeCardSize({
    height: rect.height,
    width: rect.width,
  });
}

onMounted(() => {
  window.addEventListener("pointerdown", handleGlobalPointerDown);
  window.addEventListener("keydown", handleGlobalKeyDown);
  void refreshNoteKindCounts();
  void nextTick(measureHomeCardSize);
});

onUnmounted(() => {
  window.removeEventListener("pointerdown", handleGlobalPointerDown);
  window.removeEventListener("keydown", handleGlobalKeyDown);
});

watch(
  () => props.resultCount,
  () => {
    void refreshNoteKindCounts();
  },
);

watch(
  () => [props.columnWidth, props.masonryColumns, props.searchQuery] as const,
  () => {
    void nextTick(measureHomeCardSize);
  },
  { flush: "post" },
);
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
          :static-suggestions="searchStaticSuggestions"
          :tag-prefixes="['!@', '!#', '@', '#']"
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
        :ref="(el) => setNotesScrollRef(el as HTMLElement | null)"
        class="notes-scroll thin-scrollbar"
        tabindex="-1"
        @scroll="handleNotesScroll"
      >
        <div class="notes-columns">
          <div
            v-for="(column, columnIndex) in masonryColumns"
            :key="columnIndex"
            class="notes-column"
            :style="{ width: `${columnWidth}px` }"
          >
            <NoteCard
              v-for="note in column"
              :key="note.id"
              context-menu-enabled
              :highlight-terms="highlightTerms"
              :note="note"
              @open-context-menu="openContextMenu"
              @search-title="searchNoteTitle"
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
