<script setup lang="ts">
import { useI18n } from "../../i18n";
import type { TagSummary } from "./tagSettingsRepository";

defineProps<{
  tag: TagSummary;
}>();

defineEmits<{
  cancel: [];
  confirm: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="tag-delete-backdrop" role="presentation" @pointerdown.self="$emit('cancel')">
    <section class="tag-delete-dialog" role="dialog" aria-modal="true" aria-labelledby="tag-delete-title">
      <h2 id="tag-delete-title">{{ t("settings.tags.deleteTitle") }}</h2>

      <p class="tag-delete-summary">
        #{{ tag.label }}
        <span>· {{ t("settings.tags.cardCount", { count: tag.count }) }}</span>
      </p>

      <p class="tag-delete-copy">{{ t("settings.tags.deleteCopy") }}</p>

      <div class="tag-delete-actions">
        <button type="button" class="cancel-button" @click="$emit('cancel')">{{ t("common.cancel") }}</button>
        <button type="button" class="delete-button" @click="$emit('confirm')">{{ t("common.delete") }}</button>
      </div>
    </section>
  </div>
</template>

<style scoped src="./SettingsTagDeleteConfirm.scoped.css"></style>
