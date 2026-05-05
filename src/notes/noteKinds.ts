import type { MessageKey } from "../i18n/zh-CN";

export const noteKindDefinitions = [
  {
    labelKey: "kind.word",
    legacyAliases: ["词语"],
    value: "word",
  },
  {
    labelKey: "kind.sentence",
    legacyAliases: ["句子"],
    value: "sentence",
  },
  {
    labelKey: "kind.paragraph",
    legacyAliases: ["段落"],
    value: "paragraph",
  },
] as const satisfies ReadonlyArray<{
  labelKey: MessageKey;
  legacyAliases: readonly string[];
  value: string;
}>;

export type NoteKind = (typeof noteKindDefinitions)[number]["value"];

export function findNoteKindDefinition(kind: string) {
  return noteKindDefinitions.find(
    (definition) =>
      definition.value.toLowerCase() === kind.toLowerCase() ||
      definition.legacyAliases.some((alias) => alias.toLowerCase() === kind.toLowerCase()),
  );
}
