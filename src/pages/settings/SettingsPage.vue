<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { ArrowLeft, ChevronDown, Settings } from "lucide-vue-next";
import { useI18n } from "../../i18n";
import {
  chooseDataDir,
  migrateDataDir,
  releaseHotkeysDisabled,
  requestHotkeysDisabled,
  revealDataDir,
  type HotkeyAction,
  type SearchEngine,
} from "../../settings/appSettingsRepository";
import { useAppSettings } from "../../settings/useAppSettings";
import { formatHotkeyParts, normalizeHotkeyFromKeyboardEvent } from "../../settings/hotkeys";
import { useAppUpdater } from "../../updates/useAppUpdater";
import type { Locale, MessageKey } from "../../i18n/types";
import SettingsTagDeleteConfirm from "./SettingsTagDeleteConfirm.vue";
import { deleteTag, listAllTags, renameTag, type TagSummary } from "./tagSettingsRepository";

type SettingsTab = "general" | "tags" | "shortcuts";

const tabs: SettingsTab[] = ["general", "tags", "shortcuts"];
const tabLabelKeys: Record<SettingsTab, MessageKey> = {
  general: "settings.tabs.general",
  tags: "settings.tabs.tags",
  shortcuts: "settings.tabs.shortcuts",
};

const activeTab = ref<SettingsTab>("general");
const appVersion = ref("");
const isLanguageSelectOpen = ref(false);
const isSearchEngineSelectOpen = ref(false);
const isMigratingDataDir = ref(false);
const capturingHotkey = ref<HotkeyAction | null>(null);
const hotkeyDraft = ref("");
const storageMessage = ref("");
const tags = ref<TagSummary[]>([]);
const tagsMessage = ref("");
const isLoadingTags = ref(false);
const busyTag = ref("");
const editingTag = ref("");
const tagDraft = ref("");
const deleteTarget = ref<TagSummary | null>(null);
const shortcutMessages = reactive<Record<HotkeyAction, string>>({
  content: "",
  paragraph: "",
  save: "",
  title: "",
});
let globalHotkeysDisabledByCapture = false;
const { languageOptions, locale, selectedLanguageLabel, setLocale, t, translateError } = useI18n();
const { dataDir, hotkeys, replaceSettings, resetHotkey, searchEngine, setHotkey, setSearchEngine } = useAppSettings();
const { checkAndInstallUpdate, isBusy: isUpdateBusy, message: updateMessage } = useAppUpdater();

const emit = defineEmits<{
  back: [];
  dataDirChanged: [];
  tagsChanged: [];
}>();

const activeTabLabel = computed(() => t(tabLabelKeys[activeTab.value]));
const hasTags = computed(() => tags.value.length > 0);
const tagTotalText = computed(() => t("settings.tags.total", { count: tags.value.length }));
const searchEngineOptions = computed(() => [
  { label: "Google", value: "google" as const },
  { label: "Bing", value: "bing" as const },
  { label: "Baidu", value: "baidu" as const },
]);
const selectedSearchEngineLabel = computed(
  () => searchEngineOptions.value.find((option) => option.value === searchEngine.value)?.label ?? "Google",
);
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
  {
    action: "save" as const,
    description: t("settings.shortcuts.saveDescription"),
    hotkey: hotkeys.value.save,
    title: t("settings.shortcuts.saveTitle"),
  },
]);

function selectLanguage(value: Locale) {
  void setLocale(value);
  isLanguageSelectOpen.value = false;
}

function selectSearchEngine(value: SearchEngine) {
  void setSearchEngine(value);
  isSearchEngineSelectOpen.value = false;
}

function toggleLanguageSelect() {
  isLanguageSelectOpen.value = !isLanguageSelectOpen.value;
  if (isLanguageSelectOpen.value) {
    isSearchEngineSelectOpen.value = false;
  }
}

function toggleSearchEngineSelect() {
  isSearchEngineSelectOpen.value = !isSearchEngineSelectOpen.value;
  if (isSearchEngineSelectOpen.value) {
    isLanguageSelectOpen.value = false;
  }
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
    storageMessage.value = translateError(error);
  } finally {
    isMigratingDataDir.value = false;
  }
}

async function revealCurrentDataDir() {
  try {
    await revealDataDir();
  } catch (error) {
    storageMessage.value = translateError(error);
  }
}

async function loadTags() {
  if (isLoadingTags.value) {
    return;
  }

  try {
    isLoadingTags.value = true;
    tagsMessage.value = "";
    tags.value = await listAllTags();
  } catch (error) {
    tagsMessage.value = translateError(error);
  } finally {
    isLoadingTags.value = false;
  }
}

function startEditingTag(tag: TagSummary) {
  editingTag.value = tag.label;
  tagDraft.value = tag.label;
  tagsMessage.value = "";
}

function cancelEditingTag() {
  editingTag.value = "";
  tagDraft.value = "";
}

async function saveTagEdit(oldTag: string) {
  const newTag = tagDraft.value.trim().replace(/^#+/, "").trim();

  if (!newTag) {
    tagsMessage.value = t("settings.tags.emptyError");
    return;
  }

  if (/\s/.test(newTag)) {
    tagsMessage.value = t("settings.tags.whitespaceError");
    return;
  }

  try {
    busyTag.value = oldTag;
    tags.value = await renameTag(oldTag, newTag);
    tagsMessage.value = t("settings.tags.renameSaved");
    cancelEditingTag();
    emit("tagsChanged");
  } catch (error) {
    tagsMessage.value = translateError(error);
  } finally {
    busyTag.value = "";
  }
}

function requestDeleteTag(tag: TagSummary) {
  deleteTarget.value = tag;
  tagsMessage.value = "";
}

function cancelDeleteTag() {
  deleteTarget.value = null;
}

async function confirmDeleteTag() {
  if (!deleteTarget.value) {
    return;
  }

  const tag = deleteTarget.value.label;

  try {
    busyTag.value = tag;
    tags.value = await deleteTag(tag);
    tagsMessage.value = t("settings.tags.deleteSaved");
    cancelDeleteTag();
    emit("tagsChanged");
  } catch (error) {
    tagsMessage.value = translateError(error);
  } finally {
    busyTag.value = "";
  }
}

async function beginHotkeyCapture(action: HotkeyAction) {
  capturingHotkey.value = action;
  hotkeyDraft.value = hotkeys.value[action];
  shortcutMessages[action] = t("settings.shortcuts.captureHelp");
  await disableGlobalHotkeysForCapture();
  await nextTick();
  document.querySelector<HTMLInputElement>(`[data-hotkey-capture="${action}"]`)?.focus();
}

async function cancelHotkeyCapture() {
  if (!capturingHotkey.value) {
    return;
  }

  capturingHotkey.value = null;
  hotkeyDraft.value = "";
  await releaseGlobalHotkeysForCapture();
}

async function disableGlobalHotkeysForCapture() {
  if (globalHotkeysDisabledByCapture) {
    return;
  }

  try {
    await requestHotkeysDisabled();
    globalHotkeysDisabledByCapture = true;
  } catch (error) {
    console.error("Failed to disable global hotkeys for shortcut capture", error);
  }
}

async function releaseGlobalHotkeysForCapture() {
  if (!globalHotkeysDisabledByCapture) {
    return;
  }

  try {
    await releaseHotkeysDisabled();
    globalHotkeysDisabledByCapture = false;
  } catch (error) {
    console.error("Failed to restore global hotkeys after shortcut capture", error);
  }
}

async function resetShortcut(action: HotkeyAction) {
  try {
    await resetHotkey(action);
    shortcutMessages[action] = t("settings.shortcuts.saved");
  } catch (error) {
    shortcutMessages[action] = translateError(error);
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
    shortcutMessages[action] = translateError(error);
  }
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

  if (isSearchEngineSelectOpen.value) {
    isSearchEngineSelectOpen.value = false;
    return;
  }

  if (deleteTarget.value) {
    cancelDeleteTag();
    return;
  }

  if (editingTag.value) {
    cancelEditingTag();
    return;
  }

  emit("back");
}

async function loadAppVersion() {
  try {
    appVersion.value = await getVersion();
  } catch (error) {
    console.warn("Failed to load app version", error);
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleSettingsKeydown, true);
  void loadAppVersion();
  void loadTags();
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleSettingsKeydown, true);
  void releaseGlobalHotkeysForCapture();
});

watch(activeTab, (tab) => {
  if (tab === "tags") {
    void loadTags();
  }
});
</script>

<template>
  <main class="page-shell">
    <div class="settings-toolbar page-toolbar" :aria-label="t('settings.actions')">
      <button type="button" class="page-back-button" @click="$emit('back')">
        <ArrowLeft aria-hidden="true" />
        <span>{{ t("common.back") }}</span>
      </button>

      <button type="button" class="page-icon-button" :aria-label="t('common.settings')" :title="t('common.settings')">
        <Settings aria-hidden="true" />
      </button>
    </div>

    <div class="settings-scroll page-scroll thin-scrollbar">
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
                <button type="button" class="select-trigger" @click="toggleLanguageSelect">
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
              <span class="setting-title">{{ t("settings.general.searchEngine") }}</span>
              <span class="setting-description">{{ t("settings.general.searchEngineDescription") }}</span>
              <span class="select-field">
                <button type="button" class="select-trigger" @click="toggleSearchEngineSelect">
                  {{ selectedSearchEngineLabel }}
                  <ChevronDown aria-hidden="true" />
                </button>
                <span v-if="isSearchEngineSelectOpen" class="select-menu" role="listbox">
                  <button
                    v-for="option in searchEngineOptions"
                    :key="option.value"
                    type="button"
                    :class="{ 'is-selected': searchEngine === option.value }"
                    role="option"
                    :aria-selected="searchEngine === option.value"
                    @click="selectSearchEngine(option.value)"
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
                <button type="button" :disabled="isUpdateBusy" @click="checkAndInstallUpdate">
                  {{ t("settings.general.checkUpdates") }}
                </button>
                <span>{{ updateMessage || t("settings.general.latest") }}</span>
              </div>
            </section>
          </div>

          <div v-else-if="activeTab === 'tags'" class="settings-panel">
            <section class="setting-row">
              <span class="setting-title">{{ t("settings.tags.title") }}</span>
              <span class="setting-description">{{ tagTotalText }}</span>

              <span v-if="isLoadingTags" class="setting-description">{{ t("settings.tags.loading") }}</span>
              <span v-else-if="!hasTags" class="setting-description">{{ t("settings.tags.empty") }}</span>

              <div v-else class="tag-manager-list">
                <div v-for="tag in tags" :key="tag.label" class="tag-manager-item">
                  <template v-if="editingTag === tag.label">
                    <span class="tag-edit-field">
                      <span aria-hidden="true">#</span>
                      <input
                        v-model="tagDraft"
                        type="text"
                        maxlength="80"
                        @keydown.enter.prevent="saveTagEdit(tag.label)"
                        @keydown.esc.prevent="cancelEditingTag"
                      />
                    </span>

                    <span class="tag-count">{{ t("settings.tags.cardCount", { count: tag.count }) }}</span>

                    <span class="tag-actions">
                      <button type="button" :disabled="busyTag === tag.label" @click="saveTagEdit(tag.label)">
                        {{ t("common.save") }}
                      </button>
                      <button type="button" :disabled="busyTag === tag.label" @click="cancelEditingTag">
                        {{ t("common.cancel") }}
                      </button>
                    </span>
                  </template>

                  <template v-else>
                    <span class="tag-name">#{{ tag.label }}</span>
                    <span class="tag-count">{{ t("settings.tags.cardCount", { count: tag.count }) }}</span>

                    <span class="tag-actions">
                      <button type="button" :disabled="Boolean(busyTag)" @click="startEditingTag(tag)">
                        {{ t("common.edit") }}
                      </button>
                      <button type="button" :disabled="Boolean(busyTag)" @click="requestDeleteTag(tag)">
                        {{ t("common.delete") }}
                      </button>
                    </span>
                  </template>
                </div>
              </div>

              <span v-if="tagsMessage" class="setting-description">{{ tagsMessage }}</span>
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

    <SettingsTagDeleteConfirm
      v-if="deleteTarget"
      :tag="deleteTarget"
      @cancel="cancelDeleteTag"
      @confirm="confirmDeleteTag"
    />
  </main>
</template>

<style scoped src="./SettingsPage.scoped.css"></style>
