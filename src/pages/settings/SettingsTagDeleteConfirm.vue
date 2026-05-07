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
  <div class="confirm-backdrop" role="presentation" @pointerdown.self="$emit('cancel')">
    <section class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="tag-delete-title">
      <h2 id="tag-delete-title">{{ t("settings.tags.deleteTitle") }}</h2>

      <p class="confirm-summary">
        #{{ tag.label }}
        <span>· {{ t("settings.tags.cardCount", { count: tag.count }) }}</span>
      </p>

      <p class="confirm-copy">{{ t("settings.tags.deleteCopy") }}</p>

      <div class="confirm-actions">
        <button type="button" class="confirm-cancel-button" @click="$emit('cancel')">{{ t("common.cancel") }}</button>
        <button type="button" class="confirm-danger-button" @click="$emit('confirm')">{{ t("common.delete") }}</button>
      </div>
    </section>
  </div>
</template>
