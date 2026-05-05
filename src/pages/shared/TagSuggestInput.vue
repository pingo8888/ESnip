<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { useI18n } from "../../i18n";
import { listTags } from "../home/notesRepository";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    prioritySuggestions?: string[];
    tagPrefixes?: string[];
    type?: "search" | "text";
  }>(),
  {
    placeholder: "",
    prioritySuggestions: () => [],
    tagPrefixes: () => ["#"],
    type: "text",
  },
);

const emit = defineEmits<{
  "suggestion-open-change": [isOpen: boolean];
  "update:modelValue": [value: string];
}>();

const inputEl = ref<HTMLInputElement | null>(null);
const listEl = ref<HTMLElement | null>(null);
const suggestions = ref<string[]>([]);
const highlightedIndex = ref(0);
const activeToken = ref<{ start: number; end: number; query: string; trigger: string } | null>(null);
const { t } = useI18n();
let requestSerial = 0;
let suggestionTimer: ReturnType<typeof setTimeout> | undefined;

const isOpen = computed(() => activeToken.value !== null && suggestions.value.length > 0);

function handleInput(event: Event) {
  const input = event.target as HTMLInputElement;

  emit("update:modelValue", input.value);
  scheduleSuggestions(input.value, input.selectionStart ?? input.value.length);
}

function handleClick() {
  updateSuggestionsFromInput();
}

function handleKeyup(event: KeyboardEvent) {
  if (["ArrowDown", "ArrowUp", "Enter", "Escape"].includes(event.key)) {
    return;
  }

  updateSuggestionsFromInput();
}

function updateSuggestionsFromInput() {
  const input = inputEl.value;

  if (!input) {
    return;
  }

  scheduleSuggestions(input.value, input.selectionStart ?? input.value.length);
}

function handleKeydown(event: KeyboardEvent) {
  if (!isOpen.value) {
    return;
  }

  if (event.key === "ArrowDown") {
    event.preventDefault();
    highlightedIndex.value = (highlightedIndex.value + 1) % suggestions.value.length;
    return;
  }

  if (event.key === "ArrowUp") {
    event.preventDefault();
    highlightedIndex.value = (highlightedIndex.value - 1 + suggestions.value.length) % suggestions.value.length;
    return;
  }

  if (event.key === "Enter") {
    event.preventDefault();
    event.stopPropagation();
    selectTag(suggestions.value[highlightedIndex.value]);
    return;
  }

  if (event.key === "Escape") {
    event.preventDefault();
    event.stopPropagation();
    closeSuggestions();
  }
}

function scheduleSuggestions(value: string, cursor: number) {
  const token = findActiveTagToken(value, cursor);
  activeToken.value = token;

  if (suggestionTimer) {
    clearTimeout(suggestionTimer);
  }

  if (!token) {
    closeSuggestions();
    return;
  }

  suggestionTimer = setTimeout(() => {
    void loadSuggestions(token.query);
  }, 120);
}

async function loadSuggestions(prefix: string) {
  const requestId = ++requestSerial;

  try {
    const tags = await listTags(prefix);

    if (requestId !== requestSerial) {
      return;
    }

    suggestions.value = buildSuggestions(prefix, tags);
    highlightedIndex.value = Math.min(highlightedIndex.value, Math.max(suggestions.value.length - 1, 0));
  } catch (error) {
    console.error("Failed to load tag suggestions", error);
    if (requestId === requestSerial) {
      closeSuggestions();
    }
  }
}

function filterExistingTags(tags: string[]) {
  const existingTags = parseExistingTags();

  return tags.filter((tag) => !existingTags.some((existingTag) => existingTag.toLowerCase() === tag.toLowerCase()));
}

function buildSuggestions(prefix: string, tags: string[]) {
  const normalizedPrefix = prefix.toLowerCase();
  const prioritySuggestions = props.prioritySuggestions.filter((suggestion) =>
    suggestion.toLowerCase().startsWith(normalizedPrefix),
  );
  const mergedSuggestions = [...prioritySuggestions, ...tags];
  const uniqueSuggestions: string[] = [];

  for (const suggestion of mergedSuggestions) {
    if (!uniqueSuggestions.some((item) => item.toLowerCase() === suggestion.toLowerCase())) {
      uniqueSuggestions.push(suggestion);
    }
  }

  return filterExistingTags(uniqueSuggestions);
}

function parseExistingTags() {
  const value = props.modelValue;
  const excludedStart = activeToken.value?.start ?? -1;
  const excludedEnd = activeToken.value?.end ?? -1;
  const tags = new Set<string>();
  const tagPattern = /#([^\s,，#]+)/g;
  let match: RegExpExecArray | null;

  while ((match = tagPattern.exec(value))) {
    if (match.index >= excludedStart && match.index < excludedEnd) {
      continue;
    }

    tags.add(match[1].trim());
  }

  return [...tags];
}

function selectTag(tag: string) {
  if (!activeToken.value) {
    return;
  }

  const value = props.modelValue;
  const before = value.slice(0, activeToken.value.start);
  const after = value.slice(activeToken.value.end).replace(/^\s*/, "");
  const nextValue = `${before}${activeToken.value.trigger}${tag} ${after}`;
  const nextCursor = before.length + activeToken.value.trigger.length + tag.length + 1;

  emit("update:modelValue", nextValue);
  closeSuggestions();

  void nextTick(() => {
    inputEl.value?.focus();
    inputEl.value?.setSelectionRange(nextCursor, nextCursor);
  });
}

function closeSuggestions() {
  activeToken.value = null;
  suggestions.value = [];
  highlightedIndex.value = 0;
}

function scrollHighlightedIntoView() {
  void nextTick(() => {
    listEl.value?.querySelector(".is-active")?.scrollIntoView({ block: "nearest" });
  });
}

watch(highlightedIndex, () => {
  scrollHighlightedIntoView();
});

function findActiveTagToken(value: string, cursor: number) {
  const beforeCursor = value.slice(0, cursor);
  const tokenStart = Math.max(beforeCursor.lastIndexOf(" "), beforeCursor.lastIndexOf(","), beforeCursor.lastIndexOf("，")) + 1;
  const token = beforeCursor.slice(tokenStart);

  const trigger = [...props.tagPrefixes].sort((a, b) => b.length - a.length).find((prefix) => token.startsWith(prefix));

  if (!trigger) {
    return null;
  }

  const tokenEndMatch = value.slice(cursor).match(/[\s,，]/);
  const tokenEnd = tokenEndMatch ? cursor + tokenEndMatch.index! : value.length;

  return {
    end: tokenEnd,
    query: token.slice(trigger.length),
    start: tokenStart,
    trigger,
  };
}

watch(
  () => props.modelValue,
  (value) => {
    if (!value) {
      closeSuggestions();
    }
  },
);

watch(isOpen, (value) => {
  emit("suggestion-open-change", value);
});
</script>

<template>
  <div class="tag-suggest-input">
    <input
      ref="inputEl"
      :placeholder="placeholder"
      :type="type"
      :value="modelValue"
      @click="handleClick"
      @input="handleInput"
      @keydown="handleKeydown"
      @keyup="handleKeyup"
    />

    <div v-if="isOpen" class="tag-suggestions" role="listbox">
      <div ref="listEl" class="tag-suggestions-list" tabindex="-1">
        <button
          v-for="(tag, index) in suggestions"
          :key="tag"
          type="button"
          tabindex="-1"
          :class="{ 'is-active': highlightedIndex === index }"
          role="option"
          @mousedown.prevent="selectTag(tag)"
        >
          <span>{{ activeToken?.trigger ?? "#" }}{{ tag }}</span>
        </button>
      </div>

      <div class="tag-suggestions-help" aria-hidden="true">
        <span>{{ t("tagSuggestions.select") }}</span>
        <span>{{ t("tagSuggestions.confirm") }}</span>
        <span>{{ t("tagSuggestions.close") }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped src="./TagSuggestInput.scoped.css"></style>
