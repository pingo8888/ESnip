<script setup lang="ts">
defineProps<{
  x: number;
  y: number;
}>();

defineEmits<{
  close: [];
  delete: [];
  edit: [];
}>();
</script>

<template>
  <div
    class="note-context-menu"
    :style="{ left: `${x}px`, top: `${y}px` }"
    role="menu"
    aria-label="卡片菜单"
    @pointerdown.stop
  >
    <button type="button" role="menuitem" @click="$emit('edit')">编辑</button>
    <div class="menu-separator" aria-hidden="true"></div>
    <button type="button" class="danger-item" role="menuitem" @click="$emit('delete')">删除...</button>
  </div>
</template>

<style scoped src="./NoteContextMenu.scoped.css"></style>
