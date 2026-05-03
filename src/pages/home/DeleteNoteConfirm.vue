<script setup lang="ts">
import type { Note } from "./noteTypes";

defineProps<{
  note: Note;
}>();

defineEmits<{
  cancel: [];
  confirm: [];
}>();
</script>

<template>
  <div class="delete-confirm-backdrop" role="presentation" @pointerdown.self="$emit('cancel')">
    <section class="delete-confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="delete-confirm-title">
      <h2 id="delete-confirm-title">删除这张卡片？</h2>

      <p class="delete-summary">
        {{ note.title || "无标题" }}
        <span>· {{ note.kind }}</span>
      </p>

      <p class="delete-copy">确定移除卡片？此操作无法撤销。</p>

      <div class="delete-actions">
        <button type="button" class="cancel-button" @click="$emit('cancel')">取消</button>
        <button type="button" class="delete-button" @click="$emit('confirm')">删除</button>
      </div>
    </section>
  </div>
</template>

<style scoped src="./DeleteNoteConfirm.scoped.css"></style>
