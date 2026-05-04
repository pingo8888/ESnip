<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { ArrowLeft, ChevronDown, Settings } from "lucide-vue-next";
import { useI18n } from "../../i18n";
import type { Locale, MessageKey } from "../../i18n/types";
import NoteCard from "../home/NoteCard.vue";
import type { Note, NoteKind } from "../home/noteTypes";

type SettingsTab = "general" | "cards" | "storage" | "shortcuts" | "about";
type SettingsSelect = "language" | "theme";

const appVersion = "0.1.0";

const tabs: SettingsTab[] = ["general", "cards", "storage", "shortcuts", "about"];
const tabLabelKeys: Record<SettingsTab, MessageKey> = {
  about: "settings.tabs.about",
  cards: "settings.tabs.cards",
  general: "settings.tabs.general",
  shortcuts: "settings.tabs.shortcuts",
  storage: "settings.tabs.storage",
};

const activeTab = ref<SettingsTab>("general");
const openSelect = ref<SettingsSelect | null>(null);
const theme = ref("paper-light");
const defaultKind = ref<NoteKind>("词语");
const cardWidth = ref(210);
const cardSpacing = ref(14);
const savePath = ref("~/Documents/ESnip");
const shortcutKeysTitle = ["Alt", "W"];
const shortcutKeysContent = ["Alt", "S"];
const { languageOptions, locale, selectedLanguageLabel, setLocale, t, translateNoteKind } = useI18n();

const emit = defineEmits<{
  back: [];
}>();

const activeTabLabel = computed(() => t(tabLabelKeys[activeTab.value]));

const themeOptions = computed(() => [{ label: t("settings.theme.paperLight"), value: "paper-light" }]);

const selectedThemeLabel = computed(
  () => themeOptions.value.find((option) => option.value === theme.value)?.label ?? t("settings.theme.paperLight"),
);

const cardWidthRangeStyle = computed(() => ({
  "--range-fill": `${((cardWidth.value - 180) / (320 - 180)) * 100}%`,
}));

const cardSpacingRangeStyle = computed(() => ({
  "--range-fill": `${((cardSpacing.value - 8) / (28 - 8)) * 100}%`,
}));

const previewNote = computed<Note>(() => ({
  id: "settings-preview",
  title: defaultKind.value === "词语" ? t("settings.preview.wordTitle") : t("settings.preview.title"),
  excerpt: defaultKind.value === "段落" ? t("settings.preview.paragraphBody") : t("settings.preview.body"),
  time: t("time.justNow"),
  tags: [t("settings.preview.tagPreview")],
  kind: defaultKind.value,
  tone: "sage",
  createdAt: Date.now(),
  updatedAt: Date.now(),
}));

const previewNotes = computed<Note[]>(() =>
  Array.from({ length: 4 }, (_, index) => ({
    ...previewNote.value,
    id: `settings-preview-${index}`,
    title:
      index === 0
        ? previewNote.value.title
        : [t("settings.preview.lineTitle"), t("settings.preview.spacingTitle"), t("settings.preview.previewTitle")][
            index - 1
          ],
    excerpt:
      index === 0
        ? previewNote.value.excerpt
        : [
            t("settings.preview.lineBody"),
            t("settings.preview.spacingBody"),
            t("settings.preview.previewBody"),
          ][index - 1],
    kind: (["词语", "句子", "段落", defaultKind.value] as NoteKind[])[index],
    tags: [
      [t("settings.preview.tagPreview")],
      [t("settings.preview.tagSentence")],
      [t("settings.preview.tagInterface")],
      [t("settings.preview.tagSettings")],
    ][index],
    tone: (["sage", "ochre", "clay", "ink"] as const)[index],
  })),
);

function toggleSelect(name: SettingsSelect) {
  openSelect.value = openSelect.value === name ? null : name;
}

function selectLanguage(value: Locale) {
  void setLocale(value);
  openSelect.value = null;
}

function selectTheme(value: string) {
  theme.value = value;
  openSelect.value = null;
}

function handleSettingsKeydown(event: KeyboardEvent) {
  if (event.defaultPrevented || event.isComposing || event.key !== "Escape") {
    return;
  }

  event.preventDefault();

  if (openSelect.value) {
    openSelect.value = null;
    return;
  }

  emit("back");
}

onMounted(() => {
  window.addEventListener("keydown", handleSettingsKeydown, true);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleSettingsKeydown, true);
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
                <button type="button" class="select-trigger" @click="toggleSelect('language')">
                  {{ selectedLanguageLabel }}
                  <ChevronDown aria-hidden="true" />
                </button>
                <span v-if="openSelect === 'language'" class="select-menu" role="listbox">
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
              <span class="setting-title">{{ t("settings.general.theme") }}</span>
              <span class="setting-description">{{ t("settings.general.themeDescription") }}</span>
              <span class="select-field">
                <button type="button" class="select-trigger" @click="toggleSelect('theme')">
                  {{ selectedThemeLabel }}
                  <ChevronDown aria-hidden="true" />
                </button>
                <span v-if="openSelect === 'theme'" class="select-menu" role="listbox">
                  <button
                    v-for="option in themeOptions"
                    :key="option.value"
                    type="button"
                    :class="{ 'is-selected': theme === option.value }"
                    role="option"
                    :aria-selected="theme === option.value"
                    @click="selectTheme(option.value)"
                  >
                    {{ option.label }}
                  </button>
                </span>
              </span>
            </label>
          </div>

          <div v-else-if="activeTab === 'cards'" class="settings-panel">
            <section class="setting-row">
              <span class="setting-title">{{ t("settings.cards.defaultKind") }}</span>
              <span class="setting-description">{{ t("settings.cards.defaultKindDescription") }}</span>
              <div class="segmented-control" :aria-label="t('settings.cards.defaultKind')">
                <button
                  v-for="kind in ['词语', '句子', '段落']"
                  :key="kind"
                  type="button"
                  :class="{ 'is-active': defaultKind === kind }"
                  @click="defaultKind = kind as NoteKind"
                >
                  {{ translateNoteKind(kind) }}
                </button>
              </div>
            </section>

            <label class="setting-row range-row">
              <span class="setting-title">{{ t("settings.cards.cardWidth") }}</span>
              <span class="setting-description">{{ t("settings.cards.cardWidthDescription") }}</span>
              <span class="range-control">
                <input
                  v-model.number="cardWidth"
                  type="range"
                  min="180"
                  max="320"
                  step="10"
                  :style="cardWidthRangeStyle"
                />
                <output>{{ cardWidth }}px</output>
              </span>
            </label>

            <label class="setting-row range-row">
              <span class="setting-title">{{ t("settings.cards.cardSpacing") }}</span>
              <span class="setting-description">{{ t("settings.cards.cardSpacingDescription") }}</span>
              <span class="range-control">
                <input
                  v-model.number="cardSpacing"
                  type="range"
                  min="8"
                  max="28"
                  step="2"
                  :style="cardSpacingRangeStyle"
                />
                <output>{{ cardSpacing }}px</output>
              </span>
            </label>

            <section class="setting-row">
              <span class="setting-title setting-title--caps">{{ t("settings.cards.preview") }}</span>
              <div
                class="card-preview-strip"
                :style="{ gap: `${cardSpacing}px`, '--preview-card-width': `${cardWidth}px` }"
              >
                <div
                  v-for="note in previewNotes"
                  :key="note.id"
                  class="card-preview-frame"
                >
                  <NoteCard :note="note" />
                </div>
              </div>
            </section>
          </div>

          <div v-else-if="activeTab === 'storage'" class="settings-panel">
            <label class="setting-row">
              <span class="setting-title">{{ t("settings.storage.path") }}</span>
              <span class="setting-description">{{ t("settings.storage.pathDescription") }}</span>
              <span class="path-row">
                <input v-model="savePath" type="text" />
                <button type="button">{{ t("settings.storage.choose") }}</button>
                <button type="button">{{ t("settings.storage.reveal") }}</button>
              </span>
            </label>

            <section class="setting-row">
              <span class="setting-title">{{ t("settings.storage.contents") }}</span>
              <span class="setting-description">{{ t("settings.storage.contentsDescription") }}</span>
              <p class="storage-stat">{{ t("settings.storage.contentsStat") }}</p>
              <p class="storage-stat storage-stat--muted">{{ t("settings.storage.contentsMuted") }}</p>
            </section>
          </div>

          <div v-else-if="activeTab === 'shortcuts'" class="settings-panel">
            <section class="setting-row">
              <span class="setting-title">{{ t("settings.shortcuts.titleTitle") }}</span>
              <span class="setting-description">
                {{ t("settings.shortcuts.titleDescription") }}
              </span>
              <span class="shortcut-row">
                <kbd v-for="key in shortcutKeysTitle" :key="key">{{ key }}</kbd>
                <button type="button">{{ t("settings.shortcuts.change") }}</button>
                <button type="button">{{ t("settings.shortcuts.reset") }}</button>
              </span>
            </section>

            <section class="setting-row">
              <span class="setting-title">{{ t("settings.shortcuts.contentTitle") }}</span>
              <span class="setting-description">
                {{ t("settings.shortcuts.contentDescription") }}
              </span>
              <span class="shortcut-row">
                <kbd v-for="key in shortcutKeysContent" :key="key">{{ key }}</kbd>
                <button type="button">{{ t("settings.shortcuts.change") }}</button>
                <button type="button">{{ t("settings.shortcuts.reset") }}</button>
              </span>
            </section>
          </div>

          <div v-else class="settings-panel about-panel">
            <section class="about-hero">
              <div class="app-mark" aria-hidden="true">简</div>
              <div>
                <h3>ESnip · 简摘</h3>
                <p>{{ t("settings.about.meta", { version: appVersion }) }}</p>
              </div>
            </section>

            <p class="about-copy">
              {{ t("settings.about.copy") }}
            </p>

            <section class="feature-grid" :aria-label="t('settings.about.features')">
              <div>
                <h4>{{ t("settings.about.featureCards") }}</h4>
                <p>{{ t("settings.about.featureCardsCopy") }}</p>
              </div>
              <div>
                <h4>{{ t("settings.about.featureSearch") }}</h4>
                <p>{{ t("settings.about.featureSearchCopy") }}</p>
              </div>
              <div>
                <h4>{{ t("settings.about.featureQuickCapture") }}</h4>
                <p>{{ t("settings.about.featureQuickCaptureCopy") }}</p>
              </div>
              <div>
                <h4>{{ t("settings.about.featureLocal") }}</h4>
                <p>{{ t("settings.about.featureLocalCopy") }}</p>
              </div>
            </section>

            <div class="update-row">
              <button type="button">{{ t("settings.about.checkUpdates") }}</button>
              <span>{{ t("settings.about.latest") }}</span>
            </div>
          </div>
        </section>
      </div>
    </div>
  </main>
</template>

<style scoped src="./SettingsPage.scoped.css"></style>
