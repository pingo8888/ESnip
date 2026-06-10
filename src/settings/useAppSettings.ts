import { computed, ref } from "vue";
import type { Locale } from "../i18n/types";
import {
  getAppSettings,
  updateAppSettings,
  type AppSettings,
  type AppTheme,
  type HotkeyAction,
  type HotkeySettings,
  type SearchEngine,
} from "./appSettingsRepository";

const DEFAULT_HOTKEYS: HotkeySettings = {
  content: "Alt+S",
  paragraph: "Alt+P",
  save: "Alt+Enter",
  title: "Alt+W",
};

const DEFAULT_SETTINGS: AppSettings = {
  cleanBracketedContentOnCapture: false,
  dataDir: "",
  hotkeys: { ...DEFAULT_HOTKEYS },
  locale: detectBrowserLocale(),
  searchEngine: "google",
  theme: "light",
};

const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS });
let initialized = false;

export async function initAppSettings() {
  if (initialized) {
    return settings.value;
  }

  initialized = true;

  try {
    settings.value = normalizeSettings(await getAppSettings());
    applyAppTheme(settings.value.theme);
  } catch (error) {
    console.error("Failed to load app settings", error);
  }

  return settings.value;
}

export function useAppSettings() {
  return {
    cleanBracketedContentOnCapture: computed(() => settings.value.cleanBracketedContentOnCapture),
    dataDir: computed(() => settings.value.dataDir),
    hotkeys: computed(() => settings.value.hotkeys),
    locale: computed(() => settings.value.locale),
    searchEngine: computed(() => settings.value.searchEngine),
    theme: computed(() => settings.value.theme),
    replaceSettings,
    resetHotkey,
    setCleanBracketedContentOnCapture,
    setHotkey,
    setLocale,
    setSearchEngine,
    setTheme,
  };
}

export async function setCleanBracketedContentOnCapture(cleanBracketedContentOnCapture: boolean) {
  await saveSettings({ cleanBracketedContentOnCapture });
}

export async function setTheme(theme: AppTheme) {
  await saveSettings({ theme });
}

export async function setLocale(locale: Locale) {
  await saveSettings({ locale });
}

export async function setSearchEngine(searchEngine: SearchEngine) {
  await saveSettings({ searchEngine });
}

export async function setHotkey(action: HotkeyAction, hotkey: string) {
  await saveSettings({
    hotkeys: {
      ...settings.value.hotkeys,
      [action]: hotkey,
    },
  });
}

export async function resetHotkey(action: HotkeyAction) {
  await setHotkey(action, DEFAULT_HOTKEYS[action]);
}

async function saveSettings(patch: Partial<AppSettings>) {
  const nextSettings = normalizeSettings({
    ...settings.value,
    ...patch,
  });

  settings.value = nextSettings;
  applyAppTheme(nextSettings.theme);

  try {
    settings.value = normalizeSettings(await updateAppSettings(nextSettings));
    applyAppTheme(settings.value.theme);
  } catch (error) {
    console.error("Failed to save app settings", error);
    settings.value = normalizeSettings(await getAppSettings());
    applyAppTheme(settings.value.theme);
    throw error;
  }
}

function replaceSettings(nextSettings: AppSettings) {
  settings.value = normalizeSettings(nextSettings);
  applyAppTheme(settings.value.theme);
}

function normalizeSettings(value: AppSettings): AppSettings {
  return {
    cleanBracketedContentOnCapture: value.cleanBracketedContentOnCapture === true,
    dataDir: value.dataDir ?? "",
    hotkeys: normalizeHotkeys(value.hotkeys),
    locale: value.locale === "en-US" ? "en-US" : "zh-CN",
    searchEngine: normalizeSearchEngine(value.searchEngine),
    theme: normalizeTheme(value.theme),
  };
}

function applyAppTheme(theme: AppTheme) {
  if (typeof document === "undefined") {
    return;
  }

  document.documentElement.dataset.theme = theme;
}

function detectBrowserLocale(): Locale {
  if (typeof navigator === "undefined") {
    return "zh-CN";
  }

  const languages = [navigator.language, ...navigator.languages].filter(Boolean);
  return languages.some((language) => language.toLowerCase().startsWith("zh")) ? "zh-CN" : "en-US";
}

function normalizeHotkeys(hotkeys: Partial<HotkeySettings> | undefined): HotkeySettings {
  return {
    content: hotkeys?.content || DEFAULT_HOTKEYS.content,
    paragraph: hotkeys?.paragraph || DEFAULT_HOTKEYS.paragraph,
    save: hotkeys?.save || DEFAULT_HOTKEYS.save,
    title: hotkeys?.title || DEFAULT_HOTKEYS.title,
  };
}

function normalizeSearchEngine(searchEngine: SearchEngine | undefined): SearchEngine {
  return searchEngine === "bing" || searchEngine === "baidu" ? searchEngine : "google";
}

function normalizeTheme(theme: AppTheme | undefined): AppTheme {
  return theme === "dark" ? "dark" : "light";
}

applyAppTheme(DEFAULT_SETTINGS.theme);
