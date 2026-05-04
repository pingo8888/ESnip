<script setup lang="ts">
import { useI18n } from "../../i18n";
import type { Note } from "./noteTypes";

defineProps<{
  note: Note;
}>();

defineEmits<{
  cancel: [];
  confirm: [];
}>();

const { t, translateNoteKind } = useI18n();
</script>

<template>
  <div class="delete-confirm-backdrop" role="presentation" @pointerdown.self="$emit('cancel')">
    <section class="delete-confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="delete-confirm-title">
      <h2 id="delete-confirm-title">{{ t("deleteDialog.title") }}</h2>

      <p class="delete-summary">
        {{ note.title || t("common.untitled") }}
        <span>· {{ translateNoteKind(note.kind) }}</span>
      </p>

      <p class="delete-copy">{{ t("deleteDialog.copy") }}</p>

      <div class="delete-actions">
        <button type="button" class="cancel-button" @click="$emit('cancel')">{{ t("common.cancel") }}</button>
        <button type="button" class="delete-button" @click="$emit('confirm')">{{ t("common.delete") }}</button>
      </div>
    </section>
  </div>
</template>

<style scoped src="./DeleteNoteConfirm.scoped.css"></style>
