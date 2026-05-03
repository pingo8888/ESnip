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

<style scoped>
.note-context-menu {
  position: fixed;
  z-index: 100;
  width: 198px;
  border: 1px solid #d9d0c2;
  padding: 6px 0;
  background: #fffdfa;
  box-shadow: 0 12px 28px rgba(39, 33, 27, 0.12);
  font-family: "Gill Sans", "Trebuchet MS", sans-serif;
}

.note-context-menu button {
  display: block;
  width: 100%;
  height: 32px;
  border: 0;
  padding: 0 14px;
  color: #2f2b25;
  background: transparent;
  cursor: default;
  font: inherit;
  font-size: 0.88rem;
  text-align: left;
}

.note-context-menu button:hover {
  background: rgba(117, 128, 111, 0.1);
}

.note-context-menu .danger-item {
  color: #a23f31;
}

.menu-separator {
  height: 1px;
  margin: 5px 10px;
  background: #e5ddcf;
}
</style>
