<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { ArrowLeft, Settings } from "lucide-vue-next";
import { useI18n } from "../../i18n";
import { isHotkeyEvent } from "../../settings/hotkeys";
import { useAppSettings } from "../../settings/useAppSettings";
import { computeColumnLayout } from "../home/cardColumns";
import NoteCard from "../home/NoteCard.vue";
import type { Note, NoteInput, NoteKind, NoteTone, NoteUpdateInput } from "../home/noteTypes";
import TagSuggestInput from "../shared/TagSuggestInput.vue";

const props = withDefaults(
  defineProps<{
    draftKey?: number;
    active?: boolean;
    initialNote?: Note | null;
    initialTitle?: string;
    initialContent?: string;
    initialKind?: NoteKind;
    mode?: "create" | "edit";
  }>(),
  {
    draftKey: 0,
    active: true,
    initialNote: null,
    initialTitle: "",
    initialContent: "",
    initialKind: "词语",
    mode: "create",
  },
);

const noteKinds: NoteKind[] = ["词语", "句子", "段落"];
const noteTones: NoteTone[] = ["sage", "ochre", "clay", "ink"];

const titleInputRef = ref<HTMLInputElement | null>(null);
const kind = ref<NoteKind>("词语");
const tone = ref<NoteTone>("sage");
const title = ref("");
const excerpt = ref("");
const tagsInput = ref("");
const tagSuggestionsOpen = ref(false);
const { t, translateNoteKind } = useI18n();
const { hotkeys } = useAppSettings();

const windowWidth = ref(window.innerWidth);

function onWindowResize() {
  windowWidth.value = window.innerWidth;
}

const previewCardWidth = computed(() => {
  const scrollPadding = 54; // .notes-scroll horizontal padding: 24px + 30px
  const containerWidth = windowWidth.value - scrollPadding;

  return `${computeColumnLayout(containerWidth).cardWidth}px`;
});

const emit = defineEmits<{
  cancel: [];
  openSettings: [];
  save: [note: NoteInput | NoteUpdateInput];
}>();

const pageTitle = computed(() => (props.mode === "edit" ? t("newCard.editTitle") : t("newCard.createTitle")));

const parsedTags = computed(() => parseTagsInput(tagsInput.value));

const previewNote = computed<Note>(() => ({
  id: props.initialNote?.id ?? "preview",
  title: title.value.trim() || undefined,
  excerpt: excerpt.value.trim() || undefined,
  time: t("time.justNow"),
  tags: parsedTags.value,
  kind: kind.value,
  tone: tone.value,
  createdAt: props.initialNote?.createdAt ?? Date.now(),
  updatedAt: props.initialNote?.updatedAt ?? Date.now(),
}));

function saveCard() {
  const input: NoteInput = {
    title: title.value.trim() || undefined,
    excerpt: excerpt.value.trim() || undefined,
    tags: parsedTags.value,
    kind: kind.value,
    tone: tone.value,
  };

  emit(
    "save",
    props.mode === "edit" && props.initialNote
      ? {
          ...input,
          id: props.initialNote.id,
        }
      : input,
  );
}

function handlePageKeydown(event: KeyboardEvent) {
  if (!props.active || event.defaultPrevented || event.isComposing) {
    return;
  }

  if (isHotkeyEvent(event, hotkeys.value.save)) {
    event.preventDefault();
    saveCard();
    return;
  }

  if (event.key === "Escape" && !tagSuggestionsOpen.value) {
    event.preventDefault();
    emit("cancel");
  }
}

function parseTagsInput(value: string) {
  return value
    .split(/(?=#)|[,，\s]+/)
    .map((tag) => tag.trim().replace(/^#+/, "").trim())
    .filter(Boolean);
}

function formatTagsInput(tags: string[]) {
  return tags.map((tag) => `#${tag}`).join(" ");
}

watch(
  () => [props.mode, props.initialNote, props.initialTitle, props.initialContent, props.initialKind, props.draftKey] as const,
  ([mode, note, initialTitle, initialContent, initialKind]) => {
    kind.value = note?.kind ?? initialKind;
    tone.value = note?.tone ?? "sage";
    title.value = mode === "edit" ? (note?.title ?? "") : initialTitle;
    excerpt.value = mode === "edit" ? (note?.excerpt ?? "") : initialContent;
    tagsInput.value = note ? formatTagsInput(note.tags) : "";
  },
  { immediate: true },
);

onMounted(() => {
  window.addEventListener("keydown", handlePageKeydown, true);
  window.addEventListener("resize", onWindowResize);
  titleInputRef.value?.focus();
});

onUnmounted(() => {
  window.removeEventListener("keydown", handlePageKeydown, true);
  window.removeEventListener("resize", onWindowResize);
});
</script>

<template>
  <main class="new-card-shell" @keydown.capture="handlePageKeydown">
    <div class="new-card-toolbar" :aria-label="t('newCard.actions')">
      <button type="button" class="back-button" @click="$emit('cancel')">
        <ArrowLeft aria-hidden="true" />
        <span>{{ t("common.back") }}</span>
      </button>

      <button type="button" class="icon-button" :aria-label="t('common.settings')" :title="t('common.settings')" @click="$emit('openSettings')">
        <Settings aria-hidden="true" />
      </button>
    </div>

    <div class="new-card-scroll">
      <form class="new-card-form" @submit.prevent="saveCard">
        <header class="form-heading">
          <h1>{{ pageTitle }}</h1>
          <div aria-hidden="true"></div>
        </header>

        <section class="field-group" aria-labelledby="kind-label">
          <div id="kind-label" class="field-label">{{ t("newCard.kind") }}</div>
          <div class="kind-options">
            <button
              v-for="item in noteKinds"
              :key="item"
              type="button"
              :class="{ 'is-active': kind === item }"
              @click="kind = item"
            >
              {{ translateNoteKind(item) }}
            </button>
          </div>
          <p class="field-help">{{ t("newCard.kindHelp") }}</p>
        </section>

        <label class="field-group">
          <span class="field-label">{{ t("newCard.title") }}</span>
          <input ref="titleInputRef" v-model="title" type="text" :placeholder="t('newCard.titlePlaceholder')" />
        </label>

        <label class="field-group">
          <span class="field-label">{{ t("newCard.content") }}</span>
          <textarea v-model="excerpt" rows="5" :placeholder="t('newCard.contentPlaceholder')"></textarea>
        </label>

        <div class="split-row">
          <label class="field-group">
            <span class="field-label">{{ t("newCard.tags") }}</span>
            <TagSuggestInput
              v-model="tagsInput"
              :placeholder="t('newCard.tagsPlaceholder')"
              @suggestion-open-change="tagSuggestionsOpen = $event"
            />
          </label>

          <section class="field-group accent-group" aria-labelledby="tone-label">
            <div id="tone-label" class="field-label">{{ t("newCard.accent") }}</div>
            <div class="tone-options">
              <button
                v-for="item in noteTones"
                :key="item"
                type="button"
                :class="[`tone-option--${item}`, { 'is-active': tone === item }]"
                :aria-label="t('newCard.selectTone', { tone: item })"
                @click="tone = item"
              ></button>
            </div>
          </section>
        </div>

        <section class="field-group preview-group" :style="{ width: previewCardWidth }" aria-labelledby="preview-label">
          <div id="preview-label" class="field-label">{{ t("newCard.preview") }}</div>

          <NoteCard :note="previewNote" />
        </section>

        <div class="form-actions">
          <button type="button" class="secondary-action" @click="$emit('cancel')">{{ t("newCard.cancel") }}</button>
          <button type="submit" class="primary-action">{{ t("newCard.save") }}</button>
        </div>
      </form>
    </div>
  </main>
</template>

<style scoped src="./NewCardPage.scoped.css"></style>
