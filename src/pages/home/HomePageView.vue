<script setup lang="ts">
import { Plus, Search, Settings } from "lucide-vue-next";
import type { Note } from "./notes.fixture";

defineProps<{
  masonryColumns: Note[][];
}>();

defineEmits<{
  notesScrollReady: [el: HTMLElement | null];
}>();
</script>

<template>
  <main class="commonplace-shell">
    <div class="app-toolbar" aria-label="应用操作">
      <div class="app-actions">
        <button type="button" aria-label="添加词条" title="添加词条">
          <Plus aria-hidden="true" />
        </button>
        <button type="button" aria-label="设置" title="设置">
          <Settings aria-hidden="true" />
        </button>
      </div>
    </div>

    <header class="hero">
      <label class="search-box" aria-label="搜索笔记">
        <span class="search-icon">
          <Search aria-hidden="true" />
        </span>
        <input type="search" placeholder="搜索你的摘录..." />
        <kbd>Enter</kbd>
      </label>
    </header>

    <section class="notes-section" aria-labelledby="recent-heading">
      <h2 id="recent-heading">最近添加</h2>

      <div :ref="(el) => $emit('notesScrollReady', el as HTMLElement | null)" class="notes-scroll">
        <div class="notes-columns">
          <div v-for="(column, columnIndex) in masonryColumns" :key="columnIndex" class="notes-column">
            <article
              v-for="note in column"
              :key="note.id"
              class="note-card"
              :class="`note-card--${note.tone}`"
            >
              <div class="note-accent" aria-hidden="true"></div>

              <div class="note-meta">
                <span class="note-kind">{{ note.kind }}</span>
                <time>{{ note.time }}</time>
              </div>

              <h3 v-if="note.title">{{ note.title }}</h3>
              <p v-if="note.excerpt">{{ note.excerpt }}</p>

              <footer v-if="note.tags.length > 0">
                <span v-for="tag in note.tags" :key="tag">{{ tag }}</span>
              </footer>
            </article>
          </div>
        </div>
      </div>
    </section>
  </main>
</template>

<style scoped src="./HomePageView.scoped.css"></style>
