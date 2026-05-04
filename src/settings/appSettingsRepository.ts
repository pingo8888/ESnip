import { invoke } from "@tauri-apps/api/core";
import type { Locale } from "../i18n/types";

export type AppSettings = {
  locale: Locale;
};

export async function getAppSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_app_settings");
}

export async function updateAppSettings(settings: AppSettings): Promise<AppSettings> {
  return invoke<AppSettings>("update_app_settings", { settings });
}
