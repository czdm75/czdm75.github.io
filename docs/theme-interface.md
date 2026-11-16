# 当前主题接口

本文描述当前仓库已经实现的主题接口，供修改模板、拆分组件或适配其他主题时查阅。它不是后续主题系统的设计提案，mdBook 主题使用本站 schema 和组件，并非原版 mdBook 模板的完整兼容层。

实现来源：[`assemble.rs`](../src/assemble.rs)、[`inspect.rs`](../src/inspect.rs)、[`theme.rs`](../src/theme.rs)、[`theme/default/`](../theme/default/) 和 [`theme/mdbook/`](../theme/mdbook/)。

## 1. 入口与数据流

```rust
pub fn assemble(
    files: BTreeMap<String, Vec<u8>>,
    pages: &[Page],
    options: &ThemeOptions,
) -> Result<BTreeMap<String, Vec<u8>>>
```

- `files`：Typst bundle 导出的文件集合，key 是站点相对输出路径，value 是文件字节。
- `pages`：inspection 得到的页面信息；每个 `Page.path` 必须在 `files` 中对应有效的 UTF-8 HTML。
- `options`：主题选择与导航组件开关，见下方 ThemeOptions。
- 返回值：替换为主题页面的 HTML、保留的 bundle 资源，以及新增主题资源和可选首页。

```text
Typst bundle
  ├─ export → files：完整 HTML + 资源
  └─ inspect → pages：标题、导航、标题目录等
                   ↓
               assemble
                   ├─ 构造全站 ArticleTree
                   ├─ 提取各页 head/body/lang，处理标题锚点
                   ├─ 构造 PageView，渲染 page.hbs
                   ├─ 添加主题 CSS/JS
                   └─ 缺少 index.html 时生成首页
```

`--mode site` 调用此入口；`--mode raw` 跳过主题加载。`--theme NAME` 选择主题目录，`--theme-dir PATH` 或 theme.directory 显式指定运行时主题包。

```ts
interface ThemeOptions {
  name: string;
  directory?: string;
  options: Record<string, unknown>;
}
```

该结构仅固定主题加载协议。展示选项由主题自己定义，用户通过 `[theme.options]` 和其子表配置，Rust 内部保存为 toml::Table；在 Handlebars 渲染边界由 Serde 转为 context，没有中间 JSON 文件。

引擎字段 mode、input、output、math、code-renderer、code-theme、report 位于 TOML 顶层。默认读取当前目录 typst-ssg.toml，也可通过 --config 指定。文件路径（包含 theme.directory）相对配置文件目录，CLI 路径相对当前目录。

主题选项按以下顺序合并：manifest defaults → 用户 theme.options → CLI 兼容别名 → 通用 --theme-option 覆盖。嵌套表递归合并，标量与数组整体替换。通用覆盖使用点分 KEY 和 TOML VALUE，例如 `--theme-option 'toc.mode="embedded"'`。兼容别名为 --site-tree → site-tree、--toc → toc.enabled、--toc-mode → toc.mode；Rust 只在 CLI 兼容层知道这些映射。

`Theme::load()` 合并并按主题声明的通用 option-schema 规则校验，读取模板与资源；`Theme::view(prefix, revision)` 提供动态 options 和资源 URL。没有 Rust TocMode 枚举、侧栏派生字段或 prepare_navigation 策略。主题选项不传入 Typst 编译。

## 2. 上游数据：Page

主题使用 Rust 强类型结构，不读取中间 JSON。下面用 TypeScript 风格表示经 Serde 序列化后的字段形状；这些定义是阅读说明，不是仓库已有的 TypeScript 类型或正式 JSON Schema。

```ts
interface Page {
  path: string;
  type: "page";
  title: string | null;
  navigation?: Navigation;
  metadata: unknown[]; // 任意原生 Typst Value，不作为模板上下文透传
  toc: Heading[];
  code: { language: string | null; block: boolean; text: string }[];
}

interface Navigation {
  groups: { key: string; title: string; collapsed: boolean }[];
  title: string | null;
  order: number; // Rust usize，导航排序值
}

interface Heading {
  level: number;
  text: string;
  anchor: string | null;
}
```

assemble 直接消费 `path`、`title`、`navigation`、`toc`；`type`、`metadata`、`code` 不直接提供给当前模板。`navigation` 缺失时在 Page 序列化结果中省略；它与下文模板中固定存在但可为 `null` 的字段不同。

## 3. 页面根上下文：PageView

每次 `registry.render("page", &view)` 使用一个完整的 `PageView`。这是当前主题最主要的数据边界，结构定义在 assemble 内部。

```ts
interface PageView {
  title: string;
  lang: string;
  prefix: string;
  revision: string;
  head_html: string;
  has_title: boolean;
  body_html: string;
  is_index: boolean;
  show_title: boolean;
  article_count: number;
  topic_count: number;
  tree: TreeView;
  toc: TocItem[];
  theme: ThemeView;
}
```

| 字段 | 文章页取值 / 含义 | 自动生成首页取值 |
| --- | --- | --- |
| `title` | `Page.title`，缺失则使用完整 `Page.path` | `文章与笔记` |
| `lang` | 从原 HTML 的 html 标签读取，缺失则为 `zh` | `zh` |
| `prefix` | 按输出路径中 `/` 的数量重复 `../` | 空字符串 |
| `revision` | 所选主题资源按输出路径排序后串联路径与文件字节的 FNV-1a 64 位哈希，以十六进制表示 | 同文章页 |
| `head_html` | 原 head 内部 HTML，不含 head 标签 | 空字符串 |
| `has_title` | 原 head 是否包含 title 起始标签 | `false` |
| `body_html` | 原 body 内部 HTML，可能补写标题 ID | 空字符串 |
| `is_index` | `false` | `true` |
| `show_title` | `Page.toc` 首项不是一级标题时为 `true`，空目录也为 `true` | `false` |
| `article_count` | `0`，当前不在文章页展示 | `pages.len()` |
| `topic_count` | `0`，当前不在文章页展示 | 根文章树的直接分组数，不含根文章或深层分组 |
| `tree` | 全站文章树，标识当前页，链接相对当前页 | 同一棵树，无当前页，链接相对站点根 |
| `toc` | 引擎提供的本页嵌套标题目录，即使主题关闭展示仍然提供 | 空数组 |
| `theme` | 组件开关和相对当前页的资源 URL | 同结构，资源 URL 相对站点根 |

`is_index` 表示 assemble 自动生成的首页模式。bundle 自己提供 `index.html` 时，它仍按普通文章页组装，不启用此模式。

### ThemeView

该结构定义于 src/theme.rs，由 Theme::view() 生成：

```ts
interface ThemeView {
  options: Record<string, unknown>;
  stylesheets: string[];
  scripts: string[];
}
```

options 是所选主题默认值和覆盖项合并后的动态数据。仓库两套主题定义：

```toml
[defaults]
site-tree = true
[defaults.toc]
enabled = true
mode = "split"
```

这些字段不是引擎协议；其他主题可以定义完全不同的选项。两套仓库主题在 page.hbs 中使用 eq/and/or/not 选择面板，tree.hbs 使用 current/current_ancestor 决定目录位置和初始展开。内嵌目录引用同一份页面 toc，无须 Rust 修改树结构。

资源数组已包含当前页 prefix 和 ?v=revision；模板依序输出 link 与 defer script，不再次拼接路径。所有动态选项遵守普通 HTML 转义，配置字符串不作为 HBS 表达式执行。

## 4. 全站文章树：TreeView / TreeItem

```ts
interface TreeView {
  is_folder: true;
  collapsed: boolean;
  current_ancestor: boolean;
  name: string;
  path: string;
  entries: TreeItem[];
  key: string;
}

type TreeItem = TreeView | ArticleLink;

interface ArticleLink {
  is_folder: false;
  title: string;
  url: string;
  path: string;
  current: boolean;
}
```

Rust 中 `TreeItem` 是 `#[serde(untagged)]` 枚举。序列化后没有 `Folder` / `Page` 外层包装；模板通过 `is_folder` 分支访问不同字段。分组没有 title、url、current；文章没有 entries、collapsed、current_ancestor、name、key。

| 字段 / 行为 | 约定 |
| --- | --- |
| 根节点 | `is_folder=true`、`collapsed=false`、`name=""`、`path=""` |
| 分组 `name` | 当前分组显示名称 |
| 分组 `path` | 从根到当前分组的显示名称，以 ` / ` 拼接；是展示文本，不是路由或稳定 ID |
| 分组 `key` | 内部导航分组 key 的祖先链，以长度前缀编码；跨页面和相对 URL 保持一致，不使用可重复的显示名称；根为空字符串 |
| 分组 `collapsed` | 初始折叠状态，来自导航分组；无导航配置时为 `false` |
| 文章 `title` | 优先导航标题覆盖，其次页面标题，最后使用去掉 `/index.html` 后路径的最后一段 |
| 文章 `url` | `prefix + escape_url(Page.path)`，已是相对当前页面的 URL，模板不再拼 prefix |
| 文章 `path` | 原输出路径去掉结尾 `/index.html`，用于首页辅助展示，未做 URL 编码 |
| 文章 `current` | `Page.path` 与当前文章输出路径相等 |
| 分组 `current_ancestor` | 当前文章是否位于该分组后代中；与主题展示选项无关，原 collapsed 保留 |

导航构造规则：

- 先按 `navigation.order` 排序，无导航数据的页面排序值为 `usize::MAX`。
- 有导航数据时，按 `groups` 建立层级；同一父节点下按分组 `key` 合并，分组名称和折叠状态取首次创建时的值。原始 key 组合成 TreeView.key 后提供给模板。
- 无导航数据的页面根据输出路径推导分组，去掉末尾的文章目录和文件名。
- 所有页面都没有导航数据时，递归采用分组优先、同类按路径/分组 key 字典序排序；否则保留构树时的插入顺序，允许文章和分组交错。
- 当前模型中分组自身没有页面链接；提供 current_ancestor，但不提供章节编号、前后章等字段。

## 5. 本页目录：TocItem

```ts
interface TocItem {
  text: string;
  url: string | null;
  children: TocItem[];
}
```

- 从 `Page.toc` 按语义标题级别递归嵌套，跳级时不创建虚拟节点。模板上下文不再保留 `level`。
- `url` 是 `#` 加经过 URL 编码的实际目标 ID。它是本页片段链接，不拼 `prefix`。
- `url=null` 时显示纯文本；`children` 始终存在，叶节点为空数组。
- 这是文章内部标题目录，与全站文章树是两个不同组件。

引擎始终构造目录并准备稳定锚点；`theme.options.toc.enabled=false` 仅阻止主题输出目录组件，不改变正文或目录数据。目标 ID 由 `heading_targets()` 确认：HTML 标题数量与 inspection 标题数量一致时，按出现顺序匹配，优先使用已有 ID；必要时补写不冲突的 `ssg-heading-N`。数量不一致时，仅保留确实存在于正文中的编译器锚点，其他目标为空，不猜测对应关系。这些补写不会回写到原始 `Page.toc` 或报告。

## 6. 模板组件与上下文

所有模板运行时从 manifest 声明的 template-dirs 递归读取，每次 assemble 注册一次，开启 Handlebars 严格模式。没有自定义 helper；使用内建 if、unless、each、eq、and、or、not 和 partial。仓库两套主题共使用七个模板名称，page 各自覆盖，其余六个 partial 共用；自定义主题只强制要求 page，其余组件名称不在 Rust 中注册。

| 模板 / 组件 | 收到的上下文 | 直接读取字段 | 调用组件 / 作用 |
| --- | --- | --- | --- |
| `page.hbs` | `PageView` | `lang`、`head_html`、`has_title`、`title`、`prefix`、`is_index`、`theme` | 输出完整 HTML、顶栏、可选双侧栏、文章容器；调用 tree、article/index、toc |
| `article.hbs` | 当前 `PageView` | `show_title`、`title`、`body_html` | 可选地补一个页面 h1，再插入正文 |
| `index.hbs` | 当前 `PageView` | `article_count`、`topic_count`、`tree` | 首页文案和统计；调用 index-tree |
| `tree.hbs` | TreeView + options/outline 参数 | entries；分组的 name、collapsed、current_ancestor、key；文章的 title、url、current；options/outline | HBS 决定展开分组及在当前文章下调用 embedded-toc |
| `toc.hbs` | `{ items: TocItem[] }` | `items`；每项的 `text`、`url`、`children` | 递归本页目录 |
| `embedded-toc.hbs` | `{ items: TocItem[] }` | `items` | 非空时输出 `.on-this-page` 导航并调用 toc；放在当前文章下或独立左栏中 |
| `index-tree.hbs` | `TreeView` | `entries`；分组的 `path`；文章的 `title`、`url`、`path` | 完整展示首页分组文章，不受 collapsed 影响 |

实际调用关系及上下文切换：

```hbs
{{! page → tree：显式传入 tree 对象 }}
{{> tree tree options=theme.options outline=toc}}
{{! page → article/index：继承 PageView }}
{{#if is_index}}{{> index}}{{else}}{{> article}}{{/if}}
{{! page → toc：通过 hash 参数提供 items }}
{{> toc items=toc}}
{{! index → index-tree：显式传入 tree 对象 }}
{{> index-tree tree}}
{{! each 分组分支内：当前上下文已经是子分组 }}
{{> tree options=../options outline=../outline}}
{{> index-tree}}
{{! toc 的 each 内：用 children 替换下一层 items }}
{{> toc items=children}}
```

站点名、首页介绍、面板标签等文案目前写在模板中，没有对应配置字段。严格模式下访问某种节点独有字段时，必须处于正确的 `is_folder` 分支。

## 7. HTML、转义与路径边界

- 普通字段使用 `{{field}}`，经过 Handlebars HTML 转义。
- 只有 `head_html` 和 `body_html` 使用 `{{{field}}}`，不做 HTML 转义；它们来自编译器输出，正文可附加 assemble 生成的标题 ID。
- 页面拆分基于 Typst 导出格式的字符串查找，不是通用 DOM 解析；缺少必要 head/body 标签会报错。
- 仅从原 html 标签提取 `lang`；原 html/body 标签及其他属性不会自动透传。
- `head_html` 非空时原样插入，否则补 charset 和 viewport；`has_title=false` 时另补 title。没有全面的 head 去重与合并逻辑。
- Typst 原正文的资源引用保留，不重写路径。head 先插入，主题 CSS 后加载。
- `escape_url()` 保留 ASCII 字母数字和 `-._~/`，其余按 UTF-8 字节百分号编码；它不等同于 HTML 转义。

## 8. 运行时主题包与资源

`Theme::load(options)` 首先使用显式 directory；未指定时依次查找当前项目 `theme/<name>`、可执行文件旁的 `theme/<name>`、编译时源码仓库的 `theme/<name>`（开发回退）。同名本地目录一旦找到就采用，其 manifest 出错不会静默回退。没有资源内嵌回退；分发时须携带主题及共享依赖。

每个主题目录中的 `theme.toml` 使用以下 schema：

```ts
interface ThemeManifest {
  "template-dirs": string[];
  stylesheets?: string[];
  scripts?: string[];
  assets?: Record<string, string>; // 输出路径 → manifest 相对源文件路径
  defaults?: Record<string, unknown>;
  "option-schema"?: Record<string, { type: string; enum?: unknown[] }>;
}
```

- defaults 为主题自己的默认选项表，可包含任意嵌套 TOML 值；省略时为空。option-schema 可选，key 为点分选项路径，type 支持 string、boolean、integer、float、array、table、datetime，enum 可选；只校验声明的路径，未提供的可选路径跳过。父路径存在但不是表时会报错。这是本项目的轻量规则格式，不是 JSON Schema。
- template-dirs 必填，按声明顺序递归扫描 `.hbs`；子目录模板名使用 `/`，后加载目录覆盖同名模板，不跟随扫描项中的符号链接。必须提供 page 模板。
- stylesheets/scripts 可省略，默认空数组；保持声明顺序，引用的输出路径必须在 assets 中。
- assets 可省略，默认空表；输出路径必须位于 `_theme/` 下，不能包含上级跳转、绝对路径或反斜杠。源文件相对 manifest 解析，允许通过 `../` 共享组件资源。
- 运行时资源为 `Asset { path: String, bytes: Vec<u8> }`，支持文本和二进制。资源表按输出路径排序，哈希路径与文件字节产生 revision；模板修改不影响资源 revision。
- 所有主题文件每次构建只读取一次。下次运行直接读取修改后的模板和资源，不需要重新编译。

当前仓库 manifest 为 [`default/theme.toml`](../theme/default/theme.toml) 和 [`mdbook/theme.toml`](../theme/mdbook/theme.toml)，加载的资源为：

| 主题 | 按加载顺序输出的资源 |
| --- | --- |
| default | `_theme/default/main.css`、`_theme/shared/content.css`、`_theme/shared/embedded-toc.css`；`_theme/default/main.js` |
| mdbook | `_theme/mdbook/variables.css`、`chrome.css`、`_theme/shared/content.css`、`_theme/mdbook/main.css`、`_theme/shared/embedded-toc.css`；`_theme/mdbook/panels.js`、`main.js` |

mdBook 的 `panels.js` 与 default 的 `main.js` 复用同一源文件 `theme/default/main.js`。HBS 模板不复制到输出目录。所有所选资源共用 revision，模板字节不参与该哈希。资源路径与 bundle 文件冲突时直接报错，不覆盖。

- `theme/shared/embedded-toc.css`：两套主题共用的内嵌标题目录样式。
- `theme/shared/content.css`：从原默认主题提取的正文和首页排版；正文元素规则限定在 article 内。两套外壳各自设置配色变量、字体继承和布局。
- `theme/default/main.css`：默认主题配色、布局和导航样式。default 的 page 仍引用 Google Fonts Noto Serif / Noto Serif SC。
- `theme/mdbook/variables.css`：固定版本的上游完整配色变量文件。
- `theme/mdbook/chrome.css`：从上游提取的顶栏和配色菜单规则，不引入 general.css 的正文规则。
- `theme/mdbook/main.css`：将上游配色变量映射为正文共用变量，适配本站的双侧栏和阅读布局，使用本机字体。
- `theme/mdbook/main.js`：本地配色交互，不加载上游 book.js、fonts、editor、highlight.js 或搜索脚本。

主要共用变量包括 `--text-color`、`--background-color`、`--slug-color`、`--target-color`、`--hover-color-link`、`--border-color`、`--reading-width`。mdBook 来源 commit、改动范围和 MPL-2.0 许可证见 [`NOTICE.md`](../theme/mdbook/NOTICE.md)。

## 9. DOM / JS 组件契约

共用面板脚本不读取 PageView JSON，而是依赖 page 模板输出的 DOM。更换模板时必须同时维护这些约定。

| DOM / 状态 | 用途 |
| --- | --- |
| `#grid-wrapper`、其直接 article 子元素 | 三列布局与正文阅读区域 |
| `#tree-panel` / `#toc` | 文章树 / 本页目录面板 |
| `#tree-content` / `#toc-content` | 可折叠面板内容 |
| `[data-toggle="tree"]` / `[data-toggle="toc"]` | 面板按钮；JS 读写 `aria-expanded` |
| 按钮 `aria-controls` | 指向对应内容节点 |
| body 的 `tree-collapsed` / `toc-collapsed` | CSS 控制运行时面板折叠和布局宽度 |
| body 的 `no-site-tree` / `no-toc` | 对应左/右面板未生成，不保留布局列；由 page.hbs 根据动态 options 决定 |
| body 的 `toc-embedded` / `toc-split` | 所选目录模式 |
| `.on-this-page[data-outline]` | 内嵌目录，位于当前文章链接后或独立左栏 |
| `#toc[data-outline]` | 独立右侧目录 |
| `.panel-toggle`、`.panel-toggle-icon`、`.panel-label` | 默认主题的面板按钮及折叠后外观；mdBook 按钮位于顶栏 |
| `.article-tree`、`.tree-folder[data-tree-key]`、details/summary | 文章树层级、分组身份和原生展开行为 |
| `#tree-panel[data-tree-root]` | 当前页到站点首页的相对 URL，用于隔离不同站点的文件夹状态 |
| `aria-current="page"` | 标识并突出当前文章 |
| `.toc-label` / `.bullet` | 本页目录链接；bullet 当前由 CSS 隐藏 |
| `.article-content`、`.page-heading`、`.index-page` 等 | 正文包装、补充标题和首页样式 |

交互规则：

- JS 和 CSS 使用 `max-width: 1100px` 作为紧凑布局断点。
- 桌面面板默认展开，按 localStorage 的 `panel-tree` / `panel-toc` 恢复 `open` / `closed` 偏好。
- 紧凑布局初始收起，展开一个面板时关闭另一个；此时的切换不保存桌面偏好。
- 紧凑布局点击本页目录链接会关闭其所在的左/右面板；按 Escape 关闭两侧面板。
- 分组 details 的初始 open 由 `collapsed` 决定；embedded 模式下强制展开当前文章的祖先分组以显示目录，其他分组保留原状态。在当前标签页内，用户手动展开/折叠的分组优先恢复；未操作过的分组继续使用默认状态。
- 文件夹状态按站点根 URL 存储在 sessionStorage 的 `site-tree-folders:v1:<root-url>` 中，使用分组稳定 key 区分同名节点；页面跳转与刷新后恢复。存储不可用或内容无效时继续使用默认状态。链接点击及 pagehide 同步保存，避免异步 toggle 事件晚于页面跳转。
- JS 根据实际存在的按钮选择启用的面板；关闭任意或全部组件都可运行。折叠内容设置 inert，按钮同步更新 aria-expanded。
- default 折叠后保留窄侧轨和展开按钮；mdBook 折叠后隐藏整列，展开按钮保留在顶栏。
- mdBook 额外使用 `.theme-picker`、`[data-theme]` 和 html 上的 `light/rust/coal/navy/ayu` class。配色偏好存储在 `mdbook-theme`；Auto 删除偏好并跟随系统。head 中提前应用有效配色，避免首屏闪烁；defer 脚本处理后续切换。

## 10. 当前接口边界

当前可复用的数据边界是 `PageView`、`TreeView/ArticleLink`、`TocItem`；模板负责具体 HTML，CSS/JS 共同依赖上述 DOM 契约。它们目前都是内部实现，没有 schema 版本号或外部主题兼容承诺。

当前已提供内建主题资源清单和资源 URL 数组，并支持运行时主题目录和模板覆盖，但没有 mdBook resource/fa helper 兼容层、独立 head/header/footer 扩展插槽、前后章、章节编号或搜索索引。mdBook 外壳消费本站的 PageView，而非复制 mdBook renderer 上下文。
