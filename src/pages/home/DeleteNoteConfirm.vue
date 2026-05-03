<script setup lang="ts">
import { ref } from "vue";
import type { Note } from "./noteTypes";

defineProps<{
  note: Note;
}>();

defineEmits<{
  cancel: [];
  confirm: [];
}>();

const isFinalStep = ref(false);
</script>

<template>
  <div class="delete-confirm-backdrop" role="presentation" @pointerdown.self="$emit('cancel')">
    <section class="delete-confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="delete-confirm-title">
      <h2 id="delete-confirm-title">{{ isFinalStep ? "再次确认删除？" : "删除这张卡片？" }}</h2>

      <p class="delete-summary">
        {{ note.title || "无标题" }}
        <span>· {{ note.kind }}</span>
      </p>

      <p class="delete-copy">
        {{
          isFinalStep
            ? "删除后无法恢复。再次点击删除将永久移除这张卡片。"
            : "这张卡片将从你的摘录库中移除。此操作无法撤销。"
        }}
      </p>

      <div class="delete-actions">
        <button type="button" class="cancel-button" @click="$emit('cancel')">取消</button>
        <button
          type="button"
          class="delete-button"
          @click="isFinalStep ? $emit('confirm') : (isFinalStep = true)"
        >
          删除
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.delete-confirm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 120;
  display: grid;
  place-items: center;
  background: rgba(47, 43, 37, 0.12);
}

.delete-confirm-dialog {
  width: min(420px, calc(100vw - 40px));
  border: 1px solid #d8cdbc;
  padding: 26px 28px 24px;
  background: #f6f1e8;
  box-shadow: 0 24px 70px rgba(43, 35, 27, 0.2);
}

.delete-confirm-dialog h2 {
  margin: 0 0 12px;
  color: #181512;
  font-size: 1.75rem;
  font-weight: 500;
  line-height: 1.05;
  letter-spacing: -0.07em;
}

.delete-summary {
  margin: 0 0 12px;
  color: #4d453c;
  font-family: "Gill Sans", "Trebuchet MS", sans-serif;
  font-size: 0.94rem;
}

.delete-summary span {
  color: #a89d90;
}

.delete-copy {
  margin: 0;
  color: #6b6258;
  font-family: "Gill Sans", "Trebuchet MS", sans-serif;
  font-size: 1rem;
  line-height: 1.45;
}

.delete-actions {
  display: flex;
  justify-content: flex-end;
  gap: 9px;
  margin-top: 22px;
}

.delete-actions button {
  height: 40px;
  border: 1px solid #d8cdbc;
  padding: 0 20px;
  cursor: default;
  font-family: "Gill Sans", "Trebuchet MS", sans-serif;
  font-size: 0.9rem;
  font-weight: 700;
}

.cancel-button {
  color: #5e554b;
  background: rgba(255, 254, 250, 0.48);
}

.delete-button {
  border-color: #a84536 !important;
  color: #fffaf2;
  background: #a84536;
}
</style>
