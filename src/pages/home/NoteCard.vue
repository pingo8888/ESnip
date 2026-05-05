<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "../../i18n";
import type { Note } from "./noteTypes";
import { splitHighlightParts } from "./searchHighlight";

const props = withDefaults(
  defineProps<{
    highlightTerms?: string[];
    contextMenuEnabled?: boolean;
    note: Note;
  }>(),
  {
    highlightTerms: () => [],
  },
);

defineEmits<{
  openContextMenu: [event: MouseEvent, note: Note];
}>();

const { formatRelativeTime, translateNoteKind } = useI18n();

const titleParts = computed(() => splitHighlightParts(props.note.title ?? "", props.highlightTerms));

function paragraphParts(value: string) {
  return splitHighlightParts(value, props.highlightTerms);
}

function splitParagraphs(value: string) {
  return value.split(/\r?\n/);
}
</script>

<template>
  <article
    class="note-card"
    :class="[
      `note-card--${note.tone}`,
      {
        'note-card--untitled': !note.title,
      },
    ]"
    @contextmenu.prevent="contextMenuEnabled && $emit('openContextMenu', $event, note)"
  >
    <div class="note-accent" aria-hidden="true"></div>

    <div class="note-meta">
      <span class="note-kind">{{ translateNoteKind(note.kind) }}</span>
      <time>{{ formatRelativeTime(note.updatedAt) }}</time>
    </div>

    <h3 v-if="note.title">
      <template v-for="(part, index) in titleParts" :key="index">
        <mark v-if="part.highlighted" class="note-highlight">{{ part.text }}</mark>
        <template v-else>{{ part.text }}</template>
      </template>
    </h3>
    <div v-if="note.excerpt" class="note-excerpt">
      <p v-for="(paragraph, paragraphIndex) in splitParagraphs(note.excerpt)" :key="paragraphIndex">
        <template v-for="(part, partIndex) in paragraphParts(paragraph)" :key="partIndex">
          <mark v-if="part.highlighted" class="note-highlight">{{ part.text }}</mark>
          <template v-else>{{ part.text }}</template>
        </template>
      </p>
    </div>

    <footer v-if="note.tags.length > 0">
      <span v-for="tag in note.tags" :key="tag">#{{ tag }}</span>
    </footer>
  </article>
</template>

<style scoped src="./NoteCard.scoped.css"></style>
