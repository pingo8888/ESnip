import { computed, readonly, ref } from "vue";
import { getAppSettings, updateAppChromeTitle, updateAppSettings } from "../settings/appSettingsRepository";
import { enUSMessages } from "./en-US";
import type { Locale, TranslateParams } from "./types";
import { supportedLocales } from "./types";
import { zhCNMessages, type MessageKey } from "./zh-CN";

const DEFAULT_LOCALE: Locale = "zh-CN";

const messages = {
  "en-US": enUSMessages,
  "zh-CN": zhCNMessages,
} satisfies Record<Locale, Record<MessageKey, string>>;

const currentLocale = ref<Locale>(DEFAULT_LOCALE);
let initialized = false;

export const languageOptions: Array<{ label: string; value: Locale }> = [
  { label: "中文（简体）", value: "zh-CN" },
  { label: "English", value: "en-US" },
];

export async function initI18n() {
  if (initialized) {
    return;
  }

  initialized = true;

  try {
    const settings = await getAppSettings();
    currentLocale.value = normalizeLocale(settings.locale);
    void syncAppChromeTitle();
  } catch (error) {
    console.error("Failed to load app locale", error);
    void syncAppChromeTitle();
  }
}

export async function setLocale(locale: Locale) {
  const nextLocale = normalizeLocale(locale);

  currentLocale.value = nextLocale;
  void syncAppChromeTitle();

  try {
    const settings = await updateAppSettings({ locale: nextLocale });
    currentLocale.value = normalizeLocale(settings.locale);
    void syncAppChromeTitle();
  } catch (error) {
    console.error("Failed to save app locale", error);
  }
}

export function useI18n() {
  return {
    formatRelativeTime,
    languageOptions,
    locale: readonly(currentLocale),
    selectedLanguageLabel,
    setLocale,
    t,
    translateNoteKind,
  };
}

export function t(key: MessageKey, params?: TranslateParams) {
  const template = messages[currentLocale.value][key] ?? zhCNMessages[key] ?? key;

  if (!params) {
    return template;
  }

  return template.replace(/\{(\w+)\}/g, (match, name) => String(params[name] ?? match));
}

export function translateNoteKind(kind: string) {
  if (kind === "词语") {
    return t("kind.word");
  }

  if (kind === "句子") {
    return t("kind.sentence");
  }

  if (kind === "段落") {
    return t("kind.paragraph");
  }

  return kind;
}

export function formatRelativeTime(timestamp: number) {
  const diffMs = Date.now() - timestamp;
  const minute = 60 * 1000;
  const hour = 60 * minute;
  const day = 24 * hour;

  if (diffMs < minute) {
    return t("time.justNow");
  }

  if (diffMs < hour) {
    return t("time.minutesAgo", { count: Math.floor(diffMs / minute) });
  }

  if (diffMs < day) {
    return t("time.hoursAgo", { count: Math.floor(diffMs / hour) });
  }

  if (diffMs < 2 * day) {
    return t("time.yesterday");
  }

  return t("time.daysAgo", { count: Math.floor(diffMs / day) });
}

function normalizeLocale(locale: string): Locale {
  return supportedLocales.includes(locale as Locale) ? (locale as Locale) : DEFAULT_LOCALE;
}

const selectedLanguageLabel = computed(
  () => languageOptions.find((option) => option.value === currentLocale.value)?.label ?? languageOptions[0].label,
);

async function syncAppChromeTitle() {
  try {
    await updateAppChromeTitle(t("app.name"), t("tray.show"), t("tray.quit"));
  } catch (error) {
    console.error("Failed to update app chrome title", error);
  }
}
