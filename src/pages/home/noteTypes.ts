export type NoteKind = "词语" | "句子" | "段落";

export type NoteTone = "sage" | "ochre" | "clay" | "ink";

export type Note = {
  id: string;
  title?: string;
  excerpt?: string;
  time: string;
  tags: string[];
  kind: NoteKind;
  tone: NoteTone;
  createdAt: number;
  updatedAt: number;
};

export type NoteInput = {
  title?: string;
  excerpt?: string;
  tags: string[];
  kind: NoteKind;
  tone: NoteTone;
};

export type NoteUpdateInput = NoteInput & {
  id: string;
};

export type NotesCursor = {
  updatedAt: number;
  id: string;
};
