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
};

export const testNotes: Note[] = [
  {
    id: "dashboard-rhetoric",
    title: "关于仪表盘修辞",
    excerpt:
      "仪表盘是一句由数字写成的话。多数是冗长句，好的知道哪个从句是主语，哪个只是装饰。",
    time: "2 分钟前",
    tags: ["写作", "设计"],
    kind: "段落",
    tone: "sage",
  },
  {
    id: "longform-bookmarks",
    title: "书签：长文周记",
    excerpt:
      "Susan Sontag 谈摄影重读。Robin Sloan，Year of the Meteor。Dan Luu 关于 metrics 的笔记。",
    time: "14 分钟前",
    tags: ["阅读"],
    kind: "段落",
    tone: "ochre",
  },
  {
    id: "tomato-risotto",
    title: "番茄烩饭：偷懒版",
    excerpt:
      "米饭用黄油炒香。倒入半罐番茄，慢慢收至浓稠。最后加帕玛森、罗勒和黑胡椒。",
    time: "38 分钟前",
    tags: ["食谱", "厨房"],
    kind: "段落",
    tone: "clay",
  },
  {
    id: "note-tool-rhythm",
    title: "长摘录：关于笔记工具的节奏",
    excerpt:
      "一个摘录工具最容易做错的地方，是把每一次保存都变成一次整理。真正高频的动作应该足够轻，轻到用户不会在保存前先判断这段内容是否值得进入某个分类。分类、命名、补标签都可以之后发生。第一步只需要把东西稳妥地放进去，并且让它之后还能被找回来。界面上也应该承认这一点：输入入口要明显，卡片要能容纳未整理状态，搜索要比文件夹更靠前。",
    time: "11 天前",
    tags: ["产品", "笔记", "体验"],
    kind: "段落",
    tone: "ochre",
  },
  {
    id: "q3-open-questions",
    title: "Q3 计划：开放问题",
    excerpt:
      "搜索团队向谁汇报？编辑器重构是在外出前发版，还是之后？哪些问题不能再拖。",
    time: "1 小时前",
    tags: ["工作", "计划"],
    kind: "句子",
    tone: "ink",
  },
  {
    id: "river-walk",
    title: "散步记录：江边环线",
    excerpt:
      "河堤上的新树比预想中长得好。两只白鹭停在浅水边，风里有青草味。",
    time: "3 小时前",
    tags: ["日记"],
    kind: "段落",
    tone: "sage",
  },
  {
    id: "onboarding-flow",
    title: "草图：首次启动流程",
    excerpt:
      "三个页面。欢迎、导入、第一条摘录。跳过所有废话，让导入步骤承担欢迎页假装完成的工作。",
    time: "5 小时前",
    tags: ["产品", "体验"],
    kind: "段落",
    tone: "clay",
  },
  {
    id: "birthday-ideas",
    title: "生日礼物想法",
    excerpt:
      "陶艺课。韩国陶瓷展。那支她一直借走不还的钢笔。",
    time: "昨天",
    tags: ["家庭"],
    kind: "句子",
    tone: "ochre",
  },
  {
    id: "april-reading",
    title: "四月阅读记录",
    excerpt:
      "读完《The Dawn of Everything》。搁置一本注意力主题的书，重新开始读短篇。",
    time: "昨天",
    tags: ["阅读", "日记"],
    kind: "段落",
    tone: "ink",
  },
  {
    id: "card-density-boundary",
    title: "长灵感：卡片密度和阅读感之间的边界",
    excerpt:
      "如果卡片太像表格，用户会开始扫描而不是阅读；如果卡片太像文章，列表又会失去效率。现在这个界面更接近案头纸片，所以可以保留一点不齐整的高度差。关键不是每列完全对齐，而是让眼睛能快速找到标题、类型、时间和正文开头。短卡片应该像便签，长卡片应该像折起来的段落。两者混在一起时，页面才有真实笔记库的质感。",
    time: "12 天前",
    tags: ["灵感", "设计"],
    kind: "段落",
    tone: "ink",
  },
  {
    id: "empty-state-shorter",
    title: "把搜索结果的空状态写短一点",
    time: "昨天",
    tags: ["产品"],
    kind: "句子",
    tone: "sage",
  },
  {
    id: "uneven-card-height",
    title: "灵感：卡片不需要等高",
    excerpt: "让短摘录保持短。密度比整齐更重要。",
    time: "2 天前",
    tags: ["界面"],
    kind: "句子",
    tone: "ochre",
  },
  {
    id: "local-index-tokenization",
    title: "待查：本地索引分词",
    time: "2 天前",
    tags: ["技术"],
    kind: "词语",
    tone: "ink",
  },
  {
    id: "import-format-meeting",
    title: "会议记录：导入格式",
    excerpt:
      "Markdown 保留原文。网页剪藏只做最小清洗，引用来源放到尾部。第一版不做复杂模板。",
    time: "3 天前",
    tags: ["工作"],
    kind: "段落",
    tone: "clay",
  },
  {
    id: "one-sentence",
    title: "一句话",
    time: "3 天前",
    tags: ["摘录"],
    kind: "句子",
    tone: "sage",
  },
  {
    id: "color-note",
    title: "颜色备忘",
    excerpt: "纸色再暖一点，边线保持克制。",
    time: "4 天前",
    tags: ["设计"],
    kind: "词语",
    tone: "ochre",
  },
  {
    id: "toolbar-layering",
    title: "把添加按钮放进右上角第二行",
    excerpt: "窗口控制和应用操作不是同一层级。前者是壳，后者是内容工具。",
    time: "5 天前",
    tags: ["界面", "结构"],
    kind: "段落",
    tone: "ink",
  },
  {
    id: "import-error-copy",
    title: "导入失败时的说明文案",
    excerpt:
      "不要只说导入失败。用户需要知道失败的是文件读取、格式解析，还是写入本地数据。提示可以分三层：第一句说结果，第二句说原因，第三句给下一步。比如：未能导入 3 条摘录。第 2 行缺少标题字段。请检查 CSV 表头，或改用 Markdown 导入。这个文案比错误码更长，但它能减少用户猜测，也能减少之后设置页里不必要的说明。",
    time: "13 天前",
    tags: ["文案", "导入"],
    kind: "段落",
    tone: "clay",
  },
  {
    id: "paper-surface",
    title: "纸面",
    time: "5 天前",
    tags: ["词语"],
    kind: "词语",
    tone: "sage",
  },
  {
    id: "less-thinking",
    title: "好的工具应该先让人少想一步。",
    time: "6 天前",
    tags: ["摘录"],
    kind: "句子",
    tone: "clay",
  },
  {
    id: "import-entry",
    title: "导入入口",
    excerpt:
      "先接受纯文本和 Markdown。网页剪藏后置，别在第一版把解析做成核心风险。",
    time: "6 天前",
    tags: ["产品", "计划"],
    kind: "段落",
    tone: "ochre",
  },
  {
    id: "soft-hover",
    title: "卡片 hover 不要太强",
    excerpt: "一像按钮，阅读感就没了。",
    time: "1 周前",
    tags: ["设计"],
    kind: "句子",
    tone: "sage",
  },
  {
    id: "retrieval",
    title: "检索",
    time: "1 周前",
    tags: ["词语", "技术"],
    kind: "词语",
    tone: "ink",
  },
  {
    id: "rename-sentence",
    title: "如果一句话值得保存，它通常也值得被重新命名。",
    excerpt: "标题不是摘要，是再发现的入口。",
    time: "1 周前",
    tags: ["写作"],
    kind: "句子",
    tone: "clay",
  },
  {
    id: "long-sentence-test",
    title: "特别长的句子测试",
    excerpt:
      "当一个句子被保存下来，它常常不是因为信息量最大，而是因为它把一个模糊的判断压缩成了可以再次触发记忆的形状；所以卡片标题可以承担命名，正文可以保留原句，标签只负责把它放回一个大致的语境里，不需要在第一时间完成全部整理工作。",
    time: "2 周前",
    tags: ["测试", "句子"],
    kind: "句子",
    tone: "sage",
  },
  {
    id: "settings-groups",
    title: "设置页分组",
    excerpt:
      "外观、快捷键、数据、导入导出。不要把实验项放进第一层，避免用户误以为必须配置才能开始。",
    time: "8 天前",
    tags: ["设置", "产品"],
    kind: "段落",
    tone: "sage",
  },
  {
    id: "space-is-info",
    title: "空格也是信息",
    time: "8 天前",
    tags: ["设计"],
    kind: "句子",
    tone: "ochre",
  },
  {
    id: "compact-card",
    title: "短卡片需要更像便签，不要被模板撑高。",
    time: "9 天前",
    tags: ["界面"],
    kind: "句子",
    tone: "ink",
  },
  {
    id: "source-field",
    title: "来源",
    time: "9 天前",
    tags: ["字段"],
    kind: "词语",
    tone: "clay",
  },
  {
    id: "shortcut-draft",
    title: "快捷键草案",
    excerpt:
      "Enter 搜索。Ctrl+N 新建。Esc 关闭浮层。方向键只在搜索结果中接管，不要影响正文选择。",
    time: "10 天前",
    tags: ["快捷键"],
    kind: "段落",
    tone: "sage",
  },
  {
    id: "untitled-excerpt-1",
    excerpt:
      "没有标题时，正文应该自然上移。用户刚保存一段摘录时，可能还没决定它叫什么。",
    time: "刚刚",
    tags: ["无标题", "测试"],
    kind: "段落",
    tone: "ochre",
  },
  {
    id: "untitled-excerpt-2",
    excerpt: "只保存一句话，不一定要马上命名。",
    time: "刚刚",
    tags: ["测试"],
    kind: "句子",
    tone: "sage",
  },
  {
    id: "untitled-empty-1",
    time: "刚刚",
    tags: ["空卡"],
    kind: "句子",
    tone: "ink",
  },
  {
    id: "untitled-empty-2",
    time: "刚刚",
    tags: [],
    kind: "词语",
    tone: "clay",
  },
];
