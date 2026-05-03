<script setup lang="ts">
import { Minus, Square, X } from "lucide-vue-next";
import HomePageView from "./HomePageView.vue";
import type { Note } from "./notes.fixture";

defineProps<{
  masonryColumns: Note[][];
}>();

defineEmits<{
  closeWindow: [];
  minimizeWindow: [];
  notesScrollReady: [el: HTMLElement | null];
  titlebarMouseDown: [event: MouseEvent];
  toggleMaximizeWindow: [];
}>();
</script>

<template>
  <div class="window-frame">
    <header class="window-titlebar" @mousedown="$emit('titlebarMouseDown', $event)">
      <div class="window-drag-zone">
        <span class="window-app-name">简摘</span>
      </div>

      <div class="window-controls" aria-label="窗口控制" @mousedown.stop>
        <button type="button" aria-label="最小化" title="最小化" @click="$emit('minimizeWindow')">
          <Minus aria-hidden="true" />
        </button>
        <button type="button" aria-label="最大化或还原" title="最大化或还原" @click="$emit('toggleMaximizeWindow')">
          <Square aria-hidden="true" />
        </button>
        <button type="button" class="window-close" aria-label="关闭" title="关闭" @click="$emit('closeWindow')">
          <X aria-hidden="true" />
        </button>
      </div>
    </header>

    <HomePageView :masonry-columns="masonryColumns" @notes-scroll-ready="$emit('notesScrollReady', $event)" />
  </div>
</template>

<style scoped src="../HomePage.scoped.css"></style>
<style src="../../style.css"></style>
