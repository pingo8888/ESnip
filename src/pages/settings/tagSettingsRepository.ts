import { invoke } from "@tauri-apps/api/core";

export type TagSummary = {
  label: string;
  count: number;
};

export async function listAllTags(): Promise<TagSummary[]> {
  return invoke<TagSummary[]>("list_tags", { limit: null, prefix: "" });
}

export async function renameTag(oldTag: string, newTag: string): Promise<TagSummary[]> {
  return invoke<TagSummary[]>("rename_tag", { newTag, oldTag });
}

export async function deleteTag(tag: string): Promise<TagSummary[]> {
  return invoke<TagSummary[]>("delete_tag", { tag });
}
