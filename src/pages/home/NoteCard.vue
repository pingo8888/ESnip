<script setup lang="ts">
import { useI18n } from "../../i18n";
import type { Note } from "./noteTypes";

defineProps<{
  contextMenuEnabled?: boolean;
  note: Note;
}>();

defineEmits<{
  openContextMenu: [event: MouseEvent, note: Note];
}>();

const { formatRelativeTime, translateNoteKind } = useI18n();

function splitParagraphs(value: string) {
  return value.split(/\r?\n/);
}
</script>

<template>
  <article
    class="note-card"
    :class="[`note-card--${note.tone}`, { 'note-card--untitled': !note.title }]"
    @contextmenu.prevent="contextMenuEnabled && $emit('openContextMenu', $event, note)"
  >
    <div class="note-accent" aria-hidden="true"></div>

    <div class="note-meta">
      <span class="note-kind">{{ translateNoteKind(note.kind) }}</span>
      <time>{{ formatRelativeTime(note.createdAt) }}</time>
    </div>

    <h3 v-if="note.title">{{ note.title }}</h3>
    <div v-if="note.excerpt" class="note-excerpt">
      <p v-for="(paragraph, index) in splitParagraphs(note.excerpt)" :key="index">{{ paragraph }}</p>
    </div>

    <footer v-if="note.tags.length > 0">
      <span v-for="tag in note.tags" :key="tag">#{{ tag }}</span>
    </footer>
  </article>
</template>

<style scoped src="./NoteCard.scoped.css"></style>
