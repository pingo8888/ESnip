import { invoke } from "@tauri-apps/api/core";
import type { Locale } from "../i18n/types";

export type AppSettings = {
  dataDir: string;
  hotkeys: HotkeySettings;
  locale: Locale;
  searchEngine: SearchEngine;
};

export type HotkeyAction = "title" | "content" | "paragraph" | "save";

export type HotkeySettings = Record<HotkeyAction, string>;

export type SearchEngine = "google" | "bing" | "baidu";

export async function getAppSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_app_settings");
}

export async function updateAppSettings(settings: AppSettings): Promise<AppSettings> {
  return invoke<AppSettings>("update_app_settings", { settings });
}

export async function requestHotkeysDisabled(): Promise<void> {
  await invoke("request_hotkeys_disabled");
}

export async function releaseHotkeysDisabled(): Promise<void> {
  await invoke("release_hotkeys_disabled");
}

export async function chooseDataDir(): Promise<string | null> {
  return invoke<string | null>("choose_data_dir");
}

export async function migrateDataDir(targetDir: string): Promise<AppSettings> {
  return invoke<AppSettings>("migrate_data_dir", { targetDir });
}

export async function revealDataDir(): Promise<void> {
  await invoke("reveal_data_dir");
}

export async function updateAppChromeTitle(title: string, showLabel: string, quitLabel: string): Promise<void> {
  await invoke("update_app_chrome_title", { quitLabel, showLabel, title });
}
