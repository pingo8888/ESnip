<script setup lang="ts">
import { useI18n } from "../../i18n";

defineProps<{
  x: number;
  y: number;
}>();

defineEmits<{
  close: [];
  delete: [];
  edit: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div
    class="note-context-menu"
    :style="{ left: `${x}px`, top: `${y}px` }"
    role="menu"
    :aria-label="t('menu.card')"
    @pointerdown.stop
  >
    <button type="button" role="menuitem" @click="$emit('edit')">{{ t("common.edit") }}</button>
    <div class="menu-separator" aria-hidden="true"></div>
    <button type="button" class="danger-item" role="menuitem" @click="$emit('delete')">{{ t("common.delete") }}...</button>
  </div>
</template>

<style scoped src="./NoteContextMenu.scoped.css"></style>
