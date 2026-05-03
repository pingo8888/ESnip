<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { ArrowLeft, ChevronDown, Settings } from "lucide-vue-next";
import NoteCard from "../home/NoteCard.vue";
import type { Note, NoteKind } from "../home/noteTypes";

type SettingsTab = "general" | "cards" | "storage" | "shortcuts" | "about";

const appVersion = "0.1.0";

const tabs: Array<{ id: SettingsTab; label: string }> = [
  { id: "general", label: "全局" },
  { id: "cards", label: "卡片" },
  { id: "storage", label: "保存" },
  { id: "shortcuts", label: "快捷键" },
  { id: "about", label: "关于" },
];

const activeTab = ref<SettingsTab>("general");
const openSelect = ref<"language" | "theme" | null>(null);
const language = ref("zh-CN");
const theme = ref("paper-light");
const defaultKind = ref<NoteKind>("词语");
const cardWidth = ref(210);
const cardSpacing = ref(14);
const savePath = ref("~/Documents/ESnip");
const shortcutKeys = ["Alt", "Space"];

const emit = defineEmits<{
  back: [];
}>();

const activeTabLabel = computed(() => tabs.find((tab) => tab.id === activeTab.value)?.label ?? "设置");

const languageOptions = [
  { label: "中文（简体）", value: "zh-CN" },
  { label: "English", value: "en-US" },
];

const themeOptions = [{ label: "纸面 - 浅色", value: "paper-light" }];

const selectedLanguageLabel = computed(
  () => languageOptions.find((option) => option.value === language.value)?.label ?? "中文（简体）",
);

const selectedThemeLabel = computed(
  () => themeOptions.find((option) => option.value === theme.value)?.label ?? "纸面 - 浅色",
);

const cardWidthRangeStyle = computed(() => ({
  "--range-fill": `${((cardWidth.value - 180) / (320 - 180)) * 100}%`,
}));

const cardSpacingRangeStyle = computed(() => ({
  "--range-fill": `${((cardSpacing.value - 8) / (28 - 8)) * 100}%`,
}));

const previewNote = computed<Note>(() => ({
  id: "settings-preview",
  title: defaultKind.value === "词语" ? "palimpsest" : "把真正重要的句子留下来",
  excerpt:
    defaultKind.value === "段落"
      ? "一张卡片不需要在保存时就完成整理。它只需要把当时的判断、来源和触发记忆的文字放在一起。"
      : "A manuscript on which the original text has been effaced to make room for newer writing.",
  time: "刚刚",
  tags: ["预览"],
  kind: defaultKind.value,
  tone: "sage",
  createdAt: Date.now(),
  updatedAt: Date.now(),
}));

const previewNotes = computed<Note[]>(() =>
  Array.from({ length: 4 }, (_, index) => ({
    ...previewNote.value,
    id: `settings-preview-${index}`,
    title:
      index === 0
        ? previewNote.value.title
        : ["一条句子", "纸面边距", "摘录预览"][index - 1],
    excerpt:
      index === 0
        ? previewNote.value.excerpt
        : [
            "真正需要回看的文字，应该在列表里保持安静。",
            "卡片之间的距离决定了阅读时的停顿感。",
            "预览只反映当前设置，不写入真实数据。",
          ][index - 1],
    kind: (["词语", "句子", "段落", defaultKind.value] as NoteKind[])[index],
    tags: [["预览"], ["句子"], ["界面"], ["设置"]][index],
    tone: (["sage", "ochre", "clay", "ink"] as const)[index],
  })),
);

function toggleSelect(name: "language" | "theme") {
  openSelect.value = openSelect.value === name ? null : name;
}

function selectLanguage(value: string) {
  language.value = value;
  openSelect.value = null;
}

function selectTheme(value: string) {
  theme.value = value;
  openSelect.value = null;
}

function handleSettingsKeydown(event: KeyboardEvent) {
  if (event.defaultPrevented || event.isComposing || event.key !== "Escape") {
    return;
  }

  event.preventDefault();

  if (openSelect.value) {
    openSelect.value = null;
    return;
  }

  emit("back");
}

onMounted(() => {
  window.addEventListener("keydown", handleSettingsKeydown, true);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleSettingsKeydown, true);
});
</script>

<template>
  <main class="settings-shell">
    <div class="settings-toolbar" aria-label="设置操作">
      <button type="button" class="back-button" @click="$emit('back')">
        <ArrowLeft aria-hidden="true" />
        <span>返回</span>
      </button>

      <button type="button" class="icon-button" aria-label="设置" title="设置">
        <Settings aria-hidden="true" />
      </button>
    </div>

    <div class="settings-scroll">
      <div class="settings-layout">
        <aside class="settings-sidebar" aria-label="设置分类">
          <h1>设置</h1>

          <nav>
            <button
              v-for="tab in tabs"
              :key="tab.id"
              type="button"
              :class="{ 'is-active': activeTab === tab.id }"
              @click="activeTab = tab.id"
            >
              {{ tab.label }}
            </button>
          </nav>

          <div class="version-mark">v {{ appVersion }}</div>
        </aside>

        <section class="settings-content" :aria-labelledby="`${activeTab}-heading`">
          <header class="settings-heading">
            <h2 :id="`${activeTab}-heading`">{{ activeTabLabel }}</h2>
          </header>

          <div v-if="activeTab === 'general'" class="settings-panel">
            <label class="setting-row">
              <span class="setting-title">语言</span>
              <span class="setting-description">菜单、标签和提示文本使用的界面语言。</span>
              <span class="select-field">
                <button type="button" class="select-trigger" @click="toggleSelect('language')">
                  {{ selectedLanguageLabel }}
                  <ChevronDown aria-hidden="true" />
                </button>
                <span v-if="openSelect === 'language'" class="select-menu" role="listbox">
                  <button
                    v-for="option in languageOptions"
                    :key="option.value"
                    type="button"
                    :class="{ 'is-selected': language === option.value }"
                    role="option"
                    :aria-selected="language === option.value"
                    @click="selectLanguage(option.value)"
                  >
                    {{ option.label }}
                  </button>
                </span>
              </span>
            </label>

            <label class="setting-row">
              <span class="setting-title">主题</span>
              <span class="setting-description">纸面是温暖的浅色，墨色保持安静。</span>
              <span class="select-field">
                <button type="button" class="select-trigger" @click="toggleSelect('theme')">
                  {{ selectedThemeLabel }}
                  <ChevronDown aria-hidden="true" />
                </button>
                <span v-if="openSelect === 'theme'" class="select-menu" role="listbox">
                  <button
                    v-for="option in themeOptions"
                    :key="option.value"
                    type="button"
                    :class="{ 'is-selected': theme === option.value }"
                    role="option"
                    :aria-selected="theme === option.value"
                    @click="selectTheme(option.value)"
                  >
                    {{ option.label }}
                  </button>
                </span>
              </span>
            </label>
          </div>

          <div v-else-if="activeTab === 'cards'" class="settings-panel">
            <section class="setting-row">
              <span class="setting-title">默认卡片类型</span>
              <span class="setting-description">开始新建卡片时默认选中的类型。</span>
              <div class="segmented-control" aria-label="默认卡片类型">
                <button
                  v-for="kind in ['词语', '句子', '段落']"
                  :key="kind"
                  type="button"
                  :class="{ 'is-active': defaultKind === kind }"
                  @click="defaultKind = kind as NoteKind"
                >
                  {{ kind }}
                </button>
              </div>
            </section>

            <label class="setting-row range-row">
              <span class="setting-title">卡片宽度</span>
              <span class="setting-description">设置页预览中每张卡片的宽度。</span>
              <span class="range-control">
                <input
                  v-model.number="cardWidth"
                  type="range"
                  min="180"
                  max="320"
                  step="10"
                  :style="cardWidthRangeStyle"
                />
                <output>{{ cardWidth }}px</output>
              </span>
            </label>

            <label class="setting-row range-row">
              <span class="setting-title">卡片间距</span>
              <span class="setting-description">设置页预览中卡片之间的呼吸感。</span>
              <span class="range-control">
                <input
                  v-model.number="cardSpacing"
                  type="range"
                  min="8"
                  max="28"
                  step="2"
                  :style="cardSpacingRangeStyle"
                />
                <output>{{ cardSpacing }}px</output>
              </span>
            </label>

            <section class="setting-row">
              <span class="setting-title setting-title--caps">预览</span>
              <div
                class="card-preview-strip"
                :style="{ gap: `${cardSpacing}px`, '--preview-card-width': `${cardWidth}px` }"
              >
                <div
                  v-for="note in previewNotes"
                  :key="note.id"
                  class="card-preview-frame"
                >
                  <NoteCard :note="note" />
                </div>
              </div>
            </section>
          </div>

          <div v-else-if="activeTab === 'storage'" class="settings-panel">
            <label class="setting-row">
              <span class="setting-title">保存路径</span>
              <span class="setting-description">摘录库在磁盘上的位置。第一版仅展示，不执行真实选择。</span>
              <span class="path-row">
                <input v-model="savePath" type="text" />
                <button type="button">选择...</button>
                <button type="button">在文件夹中显示</button>
              </span>
            </label>

            <section class="setting-row">
              <span class="setting-title">文件夹内容</span>
              <span class="setting-description">上次同步：今天，09:14。</span>
              <p class="storage-stat">14 张卡片 · 142 KB</p>
              <p class="storage-stat storage-stat--muted">5 个词语 · 5 个句子 · 3 个段落 · 1 个草稿</p>
            </section>
          </div>

          <div v-else-if="activeTab === 'shortcuts'" class="settings-panel">
            <section class="setting-row">
              <span class="setting-title">桌面取词快捷键</span>
              <span class="setting-description">
                在桌面任意位置唤起一个小取词窗口，粘贴或输入词语后在摘录库里查找。
              </span>
              <span class="shortcut-row">
                <kbd v-for="key in shortcutKeys" :key="key">{{ key }}</kbd>
                <button type="button">更改...</button>
                <button type="button">重置</button>
              </span>
            </section>
          </div>

          <div v-else class="settings-panel about-panel">
            <section class="about-hero">
              <div class="app-mark" aria-hidden="true">简</div>
              <div>
                <h3>ESnip · 简摘</h3>
                <p>版本 {{ appVersion }} · 桌面版 · 本地优先</p>
              </div>
            </section>

            <p class="about-copy">
              一个安静保存词语、句子和段落的地方。适合慢阅读、长期记忆，以及那些值得回头看的文字。
            </p>

            <section class="feature-grid" aria-label="软件功能">
              <div>
                <h4>三种卡片类型</h4>
                <p>词语、句子和段落，各自保留不同形状的内容。</p>
              </div>
              <div>
                <h4>即时搜索</h4>
                <p>搜索标题、正文、标签和类型，帮助你重新找到摘录。</p>
              </div>
              <div>
                <h4>桌面取词</h4>
                <p>未来可以通过快捷键唤起轻量取词窗口。</p>
              </div>
              <div>
                <h4>本地优先</h4>
                <p>数据架构预留本地存储和全文检索扩展空间。</p>
              </div>
            </section>

            <div class="update-row">
              <button type="button">检查更新</button>
              <span>当前已是最新版本。</span>
            </div>
          </div>
        </section>
      </div>
    </div>
  </main>
</template>

<style scoped src="./SettingsPage.scoped.css"></style>
