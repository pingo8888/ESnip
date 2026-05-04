export const supportedLocales = ["zh-CN", "en-US"] as const;

export type Locale = (typeof supportedLocales)[number];

export type TranslateParams = Record<string, number | string>;

export type { MessageKey } from "./zh-CN";
