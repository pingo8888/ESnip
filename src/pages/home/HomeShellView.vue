<script setup lang="ts">
import { Minus, Square, X } from "lucide-vue-next";
import { useI18n } from "../../i18n";

const { t } = useI18n();

defineEmits<{
  closeWindow: [];
  minimizeWindow: [];
  titlebarMouseDown: [event: MouseEvent];
  toggleMaximizeWindow: [];
}>();
</script>

<template>
  <div class="window-frame">
    <header class="window-titlebar" @mousedown="$emit('titlebarMouseDown', $event)">
      <div class="window-drag-zone">
        <span class="window-app-name">{{ t("app.name") }}</span>
      </div>

      <div class="window-controls" :aria-label="t('window.controls')" @mousedown.stop>
        <button type="button" tabindex="-1" :aria-label="t('window.minimize')" :title="t('window.minimize')" @click="$emit('minimizeWindow')">
          <Minus aria-hidden="true" />
        </button>
        <button type="button" tabindex="-1" :aria-label="t('window.maximize')" :title="t('window.maximize')" @click="$emit('toggleMaximizeWindow')">
          <Square aria-hidden="true" />
        </button>
        <button type="button" tabindex="-1" class="window-close" :aria-label="t('window.close')" :title="t('window.close')" @click="$emit('closeWindow')">
          <X aria-hidden="true" />
        </button>
      </div>
    </header>

    <slot />
  </div>
</template>

<style scoped src="../HomePage.scoped.css"></style>
<style src="../../style.css"></style>
