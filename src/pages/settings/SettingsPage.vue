<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref } from "vue";
import { ArrowLeft, ChevronDown, Settings } from "lucide-vue-next";
import { useI18n } from "../../i18n";
import {
  chooseDataDir,
  migrateDataDir,
  revealDataDir,
  setHotkeysEnabled,
  type HotkeyAction,
} from "../../settings/appSettingsRepository";
import { useAppSettings } from "../../settings/useAppSettings";
import type { Locale, MessageKey } from "../../i18n/types";

type SettingsTab = "general" | "shortcuts";

const appVersion = "0.1.0";

const tabs: SettingsTab[] = ["general", "shortcuts"];
const tabLabelKeys: Record<SettingsTab, MessageKey> = {
  general: "settings.tabs.general",
  shortcuts: "settings.tabs.shortcuts",
};

const activeTab = ref<SettingsTab>("general");
const isLanguageSelectOpen = ref(false);
const isMigratingDataDir = ref(false);
const capturingHotkey = ref<HotkeyAction | null>(null);
const hotkeyDraft = ref("");
const storageMessage = ref("");
const shortcutMessages = reactive<Record<HotkeyAction, string>>({
  content: "",
  paragraph: "",
  title: "",
});
const { languageOptions, locale, selectedLanguageLabel, setLocale, t } = useI18n();
const { dataDir, hotkeys, replaceSettings, resetHotkey, setHotkey } = useAppSettings();

const emit = defineEmits<{
  back: [];
  dataDirChanged: [];
}>();

const activeTabLabel = computed(() => t(tabLabelKeys[activeTab.value]));
const shortcutItems = computed(() => [
  {
    action: "title" as const,
    description: t("settings.shortcuts.titleDescription"),
    hotkey: hotkeys.value.title,
    title: t("settings.shortcuts.titleTitle"),
  },
  {
    action: "content" as const,
    description: t("settings.shortcuts.contentDescription"),
    hotkey: hotkeys.value.content,
    title: t("settings.shortcuts.contentTitle"),
  },
  {
    action: "paragraph" as const,
    description: t("settings.shortcuts.paragraphDescription"),
    hotkey: hotkeys.value.paragraph,
    title: t("settings.shortcuts.paragraphTitle"),
  },
]);

function selectLanguage(value: Locale) {
  void setLocale(value);
  isLanguageSelectOpen.value = false;
}

async function chooseAndMigrateDataDir() {
  if (isMigratingDataDir.value) {
    return;
  }

  try {
    const selectedDir = await chooseDataDir();

    if (!selectedDir) {
      return;
    }

    isMigratingDataDir.value = true;
    storageMessage.value = t("settings.storage.migrating");

    const nextSettings = await migrateDataDir(selectedDir);
    replaceSettings(nextSettings);
    storageMessage.value = t("settings.storage.migrated");
    emit("dataDirChanged");
  } catch (error) {
    storageMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isMigratingDataDir.value = false;
  }
}

async function revealCurrentDataDir() {
  try {
    await revealDataDir();
  } catch (error) {
    storageMessage.value = error instanceof Error ? error.message : String(error);
  }
}

async function beginHotkeyCapture(action: HotkeyAction) {
  capturingHotkey.value = action;
  hotkeyDraft.value = hotkeys.value[action];
  shortcutMessages[action] = t("settings.shortcuts.captureHelp");
  await setHotkeysEnabled(false);
  await nextTick();
  document.querySelector<HTMLInputElement>(`[data-hotkey-capture="${action}"]`)?.focus();
}

async function cancelHotkeyCapture() {
  if (!capturingHotkey.value) {
    return;
  }

  capturingHotkey.value = null;
  hotkeyDraft.value = "";
  await setHotkeysEnabled(true);
}

async function resetShortcut(action: HotkeyAction) {
  try {
    await resetHotkey(action);
    shortcutMessages[action] = t("settings.shortcuts.saved");
  } catch (error) {
    shortcutMessages[action] = error instanceof Error ? error.message : String(error);
  } finally {
    await cancelHotkeyCapture();
  }
}

async function saveCapturedHotkey(action: HotkeyAction, hotkey: string) {
  hotkeyDraft.value = hotkey;

  try {
    await setHotkey(action, hotkey);
    shortcutMessages[action] = t("settings.shortcuts.saved");
    await cancelHotkeyCapture();
  } catch (error) {
    shortcutMessages[action] = error instanceof Error ? error.message : String(error);
  }
}

function formatHotkeyParts(hotkey: string) {
  return hotkey.split("+").filter(Boolean);
}

function handleHotkeyCaptureKeydown(action: HotkeyAction, event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    void cancelHotkeyCapture();
    return;
  }

  if (event.key === "Backspace" || event.key === "Delete") {
    event.preventDefault();
    hotkeyDraft.value = "";
    return;
  }

  event.preventDefault();

  const normalized = normalizeHotkeyFromKeyboardEvent(event);
  if (!normalized) {
    shortcutMessages[action] = t("settings.shortcuts.invalid");
    return;
  }

  void saveCapturedHotkey(action, normalized);
}

function normalizeHotkeyFromKeyboardEvent(event: KeyboardEvent) {
  if (event.metaKey || event.shiftKey) {
    return null;
  }

  const keyToken = extractHotkeyKeyToken(event);
  if (!keyToken) {
    return null;
  }

  const validModifierCombo = (event.altKey && !event.ctrlKey) || (event.ctrlKey && event.altKey);
  if (!validModifierCombo) {
    return null;
  }

  const parts: string[] = [];
  if (event.ctrlKey) {
    parts.push("Ctrl");
  }
  if (event.altKey) {
    parts.push("Alt");
  }
  parts.push(keyToken);
  return parts.join("+");
}

function extractHotkeyKeyToken(event: KeyboardEvent) {
  const code = event.code ?? "";
  if (/^Key[A-Z]$/.test(code)) {
    return code.slice(3);
  }
  if (/^Digit[0-9]$/.test(code)) {
    return code.slice(5);
  }

  if (event.key === "Control" || event.key === "Alt" || event.key === "Shift") {
    return null;
  }
  if (/^[a-z0-9]$/i.test(event.key)) {
    return event.key.toUpperCase();
  }
  return null;
}

function handleSettingsKeydown(event: KeyboardEvent) {
  if (event.defaultPrevented || event.isComposing || event.key !== "Escape") {
    return;
  }

  event.preventDefault();

  if (capturingHotkey.value) {
    void cancelHotkeyCapture();
    return;
  }

  if (isLanguageSelectOpen.value) {
    isLanguageSelectOpen.value = false;
    return;
  }

  emit("back");
}

onMounted(() => {
  window.addEventListener("keydown", handleSettingsKeydown, true);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleSettingsKeydown, true);
  void setHotkeysEnabled(true);
});
</script>

<template>
  <main class="settings-shell">
    <div class="settings-toolbar" :aria-label="t('settings.actions')">
      <button type="button" class="back-button" @click="$emit('back')">
        <ArrowLeft aria-hidden="true" />
        <span>{{ t("common.back") }}</span>
      </button>

      <button type="button" class="icon-button" :aria-label="t('common.settings')" :title="t('common.settings')">
        <Settings aria-hidden="true" />
      </button>
    </div>

    <div class="settings-scroll">
      <div class="settings-layout">
        <aside class="settings-sidebar" :aria-label="t('settings.sidebar')">
          <h1>{{ t("common.settings") }}</h1>

          <nav>
            <button
              v-for="tab in tabs"
              :key="tab"
              type="button"
              :class="{ 'is-active': activeTab === tab }"
              @click="activeTab = tab"
            >
              {{ t(tabLabelKeys[tab]) }}
            </button>
          </nav>

          <div class="version-mark">v {{ appVersion }}</div>
        </aside>

        <section class="settings-content" :aria-labelledby="`${activeTab}-heading`">
          <header class="settings-heading">
            <h2 :id="`${activeTab}-heading`">{{ activeTabLabel }}</h2>
          </header>

          <div v-if="activeTab === 'general'" class="settings-panel">
            <label class="setting-row">
              <span class="setting-title">{{ t("settings.general.language") }}</span>
              <span class="setting-description">{{ t("settings.general.languageDescription") }}</span>
              <span class="select-field">
                <button type="button" class="select-trigger" @click="isLanguageSelectOpen = !isLanguageSelectOpen">
                  {{ selectedLanguageLabel }}
                  <ChevronDown aria-hidden="true" />
                </button>
                <span v-if="isLanguageSelectOpen" class="select-menu" role="listbox">
                  <button
                    v-for="option in languageOptions"
                    :key="option.value"
                    type="button"
                    :class="{ 'is-selected': locale === option.value }"
                    role="option"
                    :aria-selected="locale === option.value"
                    @click="selectLanguage(option.value)"
                  >
                    {{ option.label }}
                  </button>
                </span>
              </span>
            </label>

            <label class="setting-row">
              <span class="setting-title">{{ t("settings.storage.path") }}</span>
              <span class="setting-description">{{ t("settings.storage.pathDescription") }}</span>
              <span class="path-row">
                <input :value="dataDir" type="text" readonly />
                <button type="button" :disabled="isMigratingDataDir" @click="chooseAndMigrateDataDir">
                  {{ t("settings.storage.choose") }}
                </button>
                <button type="button" :disabled="isMigratingDataDir" @click="revealCurrentDataDir">
                  {{ t("settings.storage.reveal") }}
                </button>
              </span>
              <span v-if="storageMessage" class="setting-description">{{ storageMessage }}</span>
            </label>

            <section class="setting-row">
              <span class="setting-title">{{ t("settings.general.checkUpdates") }}</span>
              <div class="update-row update-row--inline">
                <button type="button">{{ t("settings.general.checkUpdates") }}</button>
                <span>{{ t("settings.general.latest") }}</span>
              </div>
            </section>
          </div>

          <div v-else-if="activeTab === 'shortcuts'" class="settings-panel">
            <section v-for="item in shortcutItems" :key="item.action" class="setting-row">
              <span class="setting-title">{{ item.title }}</span>
              <span class="setting-description">{{ item.description }}</span>
              <span class="shortcut-row">
                <span v-if="capturingHotkey !== item.action" class="shortcut-keys">
                  <kbd v-for="key in formatHotkeyParts(item.hotkey)" :key="key">{{ key }}</kbd>
                </span>
                <input
                  v-else
                  v-model="hotkeyDraft"
                  class="shortcut-capture"
                  type="text"
                  :data-hotkey-capture="item.action"
                  maxlength="16"
                  @blur="cancelHotkeyCapture"
                  @keydown="handleHotkeyCaptureKeydown(item.action, $event)"
                />
                <button type="button" @click="beginHotkeyCapture(item.action)">
                  {{ t("settings.shortcuts.change") }}
                </button>
                <button type="button" @click="resetShortcut(item.action)">
                  {{ t("settings.shortcuts.reset") }}
                </button>
              </span>
              <span v-if="shortcutMessages[item.action]" class="setting-description">
                {{ shortcutMessages[item.action] }}
              </span>
            </section>
          </div>

        </section>
      </div>
    </div>
  </main>
</template>

<style scoped src="./SettingsPage.scoped.css"></style>
