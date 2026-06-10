<script setup lang="ts">
import { Minus, Moon, Pin, PinOff, Square, Sun, X } from "lucide-vue-next";
import { useI18n } from "../../i18n";
import type { AppTheme } from "../../settings/appSettingsRepository";

const { t } = useI18n();

defineProps<{
  alwaysOnTop: boolean;
  theme: AppTheme;
}>();

defineEmits<{
  closeWindow: [];
  minimizeWindow: [];
  titlebarMouseDown: [event: MouseEvent];
  toggleAlwaysOnTop: [];
  toggleMaximizeWindow: [];
  toggleTheme: [];
}>();
</script>

<template>
  <div class="window-frame">
    <header class="window-titlebar" @mousedown="$emit('titlebarMouseDown', $event)">
      <div class="window-drag-zone">
        <span class="window-app-name">{{ t("app.name") }}</span>
      </div>

      <div class="window-controls" :aria-label="t('window.controls')" @mousedown.stop>
        <button
          type="button"
          tabindex="-1"
          :aria-label="theme === 'dark' ? t('window.switchToLightMode') : t('window.switchToDarkMode')"
          :title="theme === 'dark' ? t('window.switchToLightMode') : t('window.switchToDarkMode')"
          @click="$emit('toggleTheme')"
        >
          <Sun v-if="theme === 'dark'" aria-hidden="true" />
          <Moon v-else aria-hidden="true" />
        </button>
        <button
          type="button"
          tabindex="-1"
          class="window-pin"
          :aria-label="alwaysOnTop ? t('window.unpin') : t('window.pin')"
          :title="alwaysOnTop ? t('window.unpin') : t('window.pin')"
          @click="$emit('toggleAlwaysOnTop')"
        >
          <Pin v-if="alwaysOnTop" aria-hidden="true" />
          <PinOff v-else aria-hidden="true" />
        </button>
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
