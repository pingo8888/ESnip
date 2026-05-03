<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ArrowLeft, Settings } from "lucide-vue-next";
import NoteCard from "../home/NoteCard.vue";
import type { Note, NoteKind, NoteTone } from "../home/notes.fixture";

const props = withDefaults(
  defineProps<{
    initialNote?: Note | null;
    mode?: "create" | "edit";
  }>(),
  {
    initialNote: null,
    mode: "create",
  },
);

const noteKinds: NoteKind[] = ["词语", "句子", "段落"];
const noteTones: NoteTone[] = ["sage", "ochre", "clay", "ink"];

const kind = ref<NoteKind>("词语");
const tone = ref<NoteTone>("sage");
const title = ref("");
const excerpt = ref("");
const tagsInput = ref("");

const emit = defineEmits<{
  cancel: [];
  openSettings: [];
  save: [note: Note];
}>();

const pageTitle = computed(() => (props.mode === "edit" ? "编辑卡片" : "新建卡片"));

const parsedTags = computed(() =>
  tagsInput.value
    .split(/[,，]/)
    .map((tag) => tag.trim())
    .filter(Boolean),
);

const previewNote = computed<Note>(() => ({
  id: props.initialNote?.id ?? "preview",
  title: title.value.trim() || undefined,
  excerpt: excerpt.value.trim() || undefined,
  time: "刚刚",
  tags: parsedTags.value,
  kind: kind.value,
  tone: tone.value,
}));

function saveCard() {
  emit("save", {
    ...previewNote.value,
    id: props.mode === "edit" && props.initialNote ? props.initialNote.id : `note-${Date.now()}`,
  });
}

watch(
  () => props.initialNote,
  (note) => {
    kind.value = note?.kind ?? "词语";
    tone.value = note?.tone ?? "sage";
    title.value = note?.title ?? "";
    excerpt.value = note?.excerpt ?? "";
    tagsInput.value = note?.tags.join(", ") ?? "";
  },
  { immediate: true },
);
</script>

<template>
  <main class="new-card-shell">
    <div class="new-card-toolbar" aria-label="新建卡片操作">
      <button type="button" class="back-button" @click="$emit('cancel')">
        <ArrowLeft aria-hidden="true" />
        <span>返回</span>
      </button>

      <button type="button" class="icon-button" aria-label="设置" title="设置" @click="$emit('openSettings')">
        <Settings aria-hidden="true" />
      </button>
    </div>

    <div class="new-card-scroll">
      <form class="new-card-form" @submit.prevent="saveCard">
        <header class="form-heading">
          <h1>{{ pageTitle }}</h1>
          <div aria-hidden="true"></div>
        </header>

        <section class="field-group" aria-labelledby="kind-label">
          <div id="kind-label" class="field-label">类型</div>
          <div class="kind-options">
            <button
              v-for="item in noteKinds"
              :key="item"
              type="button"
              :class="{ 'is-active': kind === item }"
              @click="kind = item"
            >
              {{ item }}
            </button>
          </div>
          <p class="field-help">选择摘录的形态，之后可以继续调整。</p>
        </section>

        <label class="field-group">
          <span class="field-label">标题</span>
          <input v-model="title" type="text" placeholder="可以留空，也可以写一个短标题..." />
        </label>

        <label class="field-group">
          <span class="field-label">内容</span>
          <textarea v-model="excerpt" rows="5" placeholder="写下定义、原句、段落，或只是一个尚未展开的想法。"></textarea>
        </label>

        <div class="split-row">
          <label class="field-group">
            <span class="field-label">标签</span>
            <input v-model="tagsInput" type="text" placeholder="逗号分隔" />
          </label>

          <section class="field-group accent-group" aria-labelledby="tone-label">
            <div id="tone-label" class="field-label">色调</div>
            <div class="tone-options">
              <button
                v-for="item in noteTones"
                :key="item"
                type="button"
                :class="[`tone-option--${item}`, { 'is-active': tone === item }]"
                :aria-label="`选择 ${item} 色调`"
                @click="tone = item"
              ></button>
            </div>
          </section>
        </div>

        <section class="field-group preview-group" aria-labelledby="preview-label">
          <div id="preview-label" class="field-label">预览</div>

          <NoteCard :note="previewNote" />
        </section>

        <div class="form-actions">
          <button type="button" class="secondary-action" @click="$emit('cancel')">取消</button>
          <button type="submit" class="primary-action">保存卡片</button>
        </div>
      </form>
    </div>
  </main>
</template>

<style scoped src="./NewCardPage.scoped.css"></style>
