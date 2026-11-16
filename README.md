# Typst SSG 实验

目前保留 Typst bundle 作为编译与导出后端。Rust 程序直接调用 Typst 0.15.1 的库，检查编译结果，再通过 `typst_bundle::export` 输出文件，不启动 Typst CLI。

## 使用

在仓库根目录执行（需要 Rust 1.92 或更新版本）：

```sh
cargo run -- --input content --output public --math mathml --code-renderer typst
cargo run -- --input content --output public --math svg --code-renderer highlight-js
cargo test
```

Clap 的 `Cli` 解析显式命令行覆盖项，再与 TOML 和内置默认值合并为 `Options`。`--input` 与 `--output` 的内置默认值为 `content`、`public`；可用 `cargo run -- --help` 查看帮助。

引擎配置位于 TOML 顶层，主题选择位于 `[theme]`，动态展示配置位于 `[theme.options]`；以下 CLI 参数可覆盖对应配置：

- `--theme NAME`：默认 `default`，按目录名选择主题；仓库提供 default、mdbook，新增名称不需要修改 Rust。raw 模式忽略主题加载。
- `--theme-dir PATH`：显式指定包含 `theme.toml` 的主题目录，对应 `[theme].directory`，优先于按名称查找。
- `--theme-option KEY=VALUE`：通用覆盖项，可重复；KEY 是相对 theme.options 的点分路径，VALUE 使用 TOML 语法，例如 `--theme-option 'toc.mode="embedded"'`。主题新增选项不需要修改 Rust。
- `--site-tree true|false`：兼容别名，覆盖 `theme.options.site-tree`；仓库主题默认 true，控制全站文章树。
- `--toc true|false`：兼容别名，覆盖 `theme.options.toc.enabled`；仓库主题默认 true，两种布局都可关闭。
- `--toc-mode MODE`：兼容别名，覆盖 `theme.options.toc.mode`。仓库主题允许 embedded/split，默认 split。`embedded` 将本页目录嵌入左侧文章树的当前文章下，并展开其祖先分组；`split` 将全站文章树放在左侧、本页目录放在右侧。两套主题都支持。`--site-tree=false` 与 `embedded` 搭配时，左侧只显示本页目录；`--toc=false` 时模式不影响布局。
- `--math mathml|svg`：默认 `mathml`。`svg` 使用 Typst `html.frame` 渲染行内与块级公式。
- `--code-renderer typst|highlight-js`：默认 `typst`。`highlight-js` 目前仅关闭 Typst 高亮，保留 `data-lang` 和代码文本，不加载脚本或 CSS。
- `--code-theme PATH`：`typst` 模式读取 `.tmTheme` 文件，路径相对项目根目录，也可使用绝对路径；不指定时使用 Typst 默认主题。`highlight-js` 模式只保留该配置，暂不读取或应用主题。

Typst 主题控制语法颜色，不自动应用代码块的前景色和背景色。文章自己的局部规则仍可覆盖站点默认设置。显式 bundle 入口需使用 `src/render.typ` 中的 `render` show rule 才能应用这些选项。

`cargo run` 或 `mise run build` 默认构建完整站点。编译导出的 HTML、资源和强类型 `Page` 数据直接在内存中进入 assemble，默认只输出最终站点，不写中间 HTML 或每页 JSON。`--report` 可额外输出 `ssg-report.json`，用于检查原始语义数据。

仓库主题位于 `theme/default/` 和 `theme/mdbook/`。默认主题保留系统明暗配色，通过 Google Fonts 加载 Noto Serif 与 Noto Serif SC，无法联网时回退到本机字体。mdBook 主题从上游固定版本提取配色变量和顶栏样式，适配本站 tree、toc 与首页组件，提供 Auto、Light、Rust、Coal、Navy、Ayu 配色。它使用本机字体，不加载 mdBook fonts、editor、代码高亮、搜索或正文增强脚本。来源版本及许可证见 [mdBook NOTICE](theme/mdbook/NOTICE.md)。

两套主题共享 `theme/shared/content.css` 中的正文与首页基础排版，保留 Typst head、正文资源和编译期代码颜色；外围 CSS 独立维护。桌面侧栏可折叠，移动端初始收起；文件夹的手动展开/折叠状态在当前标签页内跨页面和刷新保留；未提供自定义 bundle 根页面时自动生成首页。目录和正文标题锚点始终由引擎准备；主题关闭 toc 只影响展示，不改变正文锚点。

```sh
# mdBook 外壳，保留左右导航
cargo run -- --theme mdbook --output public-mdbook
# mdBook 式内嵌目录
cargo run -- --theme mdbook --toc-mode embedded --output public-mdbook
# 只保留文章树
cargo run -- --theme mdbook --toc=false --output public-mdbook
# 专注阅读，不生成左右导航组件
cargo run -- --theme mdbook --site-tree=false --toc=false --output public-mdbook
```

切换主题时建议使用新的输出目录，构建不会清理旧主题文件。

例如 `content/cs/io/java-nio-1.typ` 输出 `public/cs/io/java-nio-1/index.html`。主题资源输出到 `public/_theme/<theme>/` 和 `public/_theme/shared/`，链接相对当前页面，可部署到子路径。Typst 的 head 样式和正文资源引用均保留。普通 HTML 标题会补齐不冲突的目录 ID；自定义 show rule 导致标题数量不一致时，仅使用能确认存在的原始锚点，其余目录项显示为文本。报告仍记录原始编译数据，不包含 assemble 补充的 ID。

输入为目录时，递归发现其中的 `.typ` / `.typst`，排序后按相对路径生成 `目录/文件名/index.html`。图片等资源由 Typst 在编译时读取。符号链接不参与发现；两个源文件映射到同一路由时提前报错。入口源码仅存在内存中，无需磁盘上的入口文件。

也可以通过 `--input path/to/bundle.typ` 指定自定义 bundle 入口。

## 配置文件

默认读取当前目录的 [`typst-ssg.toml`](typst-ssg.toml)。仓库提供的配置可直接编辑；文件不存在时直接使用内置默认值。通过 `--config PATH` 可以指定其他配置，显式指定的文件缺失或解析失败会报错。

例如使用 mdBook 内嵌目录：

```toml
# 引擎选项
mode = "site"
input = "content"
output = "public-mdbook"
math = "mathml"
code-renderer = "typst"
report = false
# code-theme = "syntax.tmTheme"

# 主题选项
[theme]
name = "mdbook"
[theme.options]
site-tree = true
[theme.options.toc]
enabled = true
mode = "embedded"
```

引擎配置优先级为 **显式 CLI 参数 > TOML > 内置默认值**。动态主题选项优先级为 **通用 CLI 覆盖 > CLI 兼容别名 > 用户 theme.options > 主题 manifest defaults**。嵌套表递归合并，标量和数组整体替换；例如只改 toc.enabled 不会丢失主题默认的 toc.mode。

引擎字段和 theme 外层字段拒绝未知名称；options 内部允许任意 TOML 数据，不在 Rust 中注册字段。主题可通过 manifest 的 option-schema 声明类型及允许值，在加载时校验。字符串按普通 Handlebars 规则转义，不会作为表达式再次执行。

```sh
cargo run
cargo run -- --config site.toml
cargo run -- --config site.toml --theme-option 'toc.mode="split"' --report=false
```

- TOML 中的 `input`、`output`、`code-theme`、`theme.directory` 相对配置文件所在目录解析，绝对路径保持不变；省略的 input/output 使用同一目录下的默认路径。
- CLI 路径相对当前工作目录解析；`--config` 自身也是如此。
- Typst 项目根目录仍为当前工作目录，输入必须在该根目录内；配置文件不会隐式更改工作目录。
- `--report` 等价于 `--report=true`，`--report=false` 可关闭 TOML 中启用的报告。
- raw 模式解析引擎配置和通用 CLI 值，但不加载主题默认值或校验主题规则。
- 迁移旧配置：原 theme 下的 site-tree、toc、toc-mode 现在分别放在 theme.options.site-tree、theme.options.toc.enabled、theme.options.toc.mode。原 CLI 参数继续支持。

## 自定义主题

主题在每次构建时从磁盘读取，修改 HBS、CSS、JS 后直接再次运行，无需重新编译 Rust。默认按顺序查找当前项目的 `theme/NAME`、可执行文件旁的 `theme/NAME`，最后使用编译时源码仓库中的 `theme/NAME` 作为开发回退。显式指定 directory 时只读取该目录，缺失或损坏不会退回其他主题。

```toml
[theme]
name = "custom"
directory = "themes/custom"
[theme.options.toolbar]
enabled = false
```

在主题目录添加 `theme.toml`，例如：

```toml
template-dirs = ["templates"]
stylesheets = ["_theme/custom/main.css"]
scripts = ["_theme/custom/main.js"]

[assets]
"_theme/custom/main.css" = "main.css"
"_theme/custom/main.js" = "main.js"
"_theme/custom/logo.png" = "logo.png"

[defaults.toolbar]
enabled = true
title = "My site"

[option-schema."toolbar.enabled"]
type = "boolean"
```

模板直接消费合并后的 context，例如：

```hbs
{{#if theme.options.toolbar.enabled}}
  <header>{{theme.options.toolbar.title}}</header>
{{/if}}
```

`eq`、`and`、`or`、`not` 是当前 Handlebars 库的内建 helper。目录嵌入、左右栏布局和默认展开策略均由 HBS 表达式实现；Rust 始终提供正文、文章树、current_ancestor 和 toc，不复制目录到文章节点，也不计算主题布局字段。

- 目录中必须提供 `templates/page.hbs`，消费现有 PageView。其他 partial 的名称由主题自己决定。
- `template-dirs` 相对 manifest 解析，递归注册 `.hbs`；子目录以 `/` 组成模板名。后加载目录中的同名模板覆盖前面的模板，符号链接不参与扫描。
- stylesheets/scripts 是按加载顺序排列的输出路径，必须在 assets 中声明。assets 的 key 是 `_theme/` 下的输出路径，value 是相对 manifest 的源文件路径，资源按二进制读取和复制。
- 可以像 [mdBook manifest](theme/mdbook/theme.toml) 一样引用共享模板和资源；运行时加载器不包含主题名称、partial 或资源注册列表。
- 独立分发可执行文件时需一起分发主题文件及其共享依赖；当前不把主题资源内嵌到二进制。

## 能读取的信息

每个 HTML 文档默认视为 `page`，检查结果使用 `Page`、`Metadata`、`Heading` 和 `Code` 结构体。`Metadata` 保留原生 Typst `Value`，允许自定义扩展携带任意 Typst 数据。可选报告将原生值序列化到 JSON；函数等类型按 Typst 的默认行为输出为文本表示。序列化错误会在写文件之前报出。使用 `cargo run -- --input content --output public --report` 输出报告，其中包括：

- `path`、`title`：bundle 的输出路径与文章标题。优先使用原生 `DocumentInfo` 标题；未指定时，若首个标题是全文唯一的 H1，则使用其纯文本（去除首尾空白），否则保持未命名。
- `navigation`：使用 summary 时额外记录导航分组、可选标题覆盖和顺序；未使用 summary 时省略。内部导航 metadata 不混入文章的 `metadata` 列表。
- `metadata`：文档查询得到的所有 `metadata` 值，检查阶段保留原生值，输出 JSON 时进行序列化。入口把已有文章导出的 `meta` 字典转换为一个不可见的 `metadata` 元素，因此包含 `title`、`lang`、`year` 及以后新增的字段。
- `toc`：通过 Typst 的 `HeadingElem` 查询得到的层级、纯文本与现有跳转锚点。包含程序生成的标题，不依赖源码扫描。没有被引用的标题可能没有锚点，此时为 `null`。
- `code`：通过 introspector 查询 `RawElem`，提取原始文本、语言和 `block`（是否为块级代码），包含行内代码。即使 show rule 改变了 HTML 表现，仍可读取原始代码语义。

## 作者接口与边界

正文仍是标准 Typst，不要求引入基础库。现有文章可以保留 `#let meta = (...)`；不导出 `meta` 的文章也能由生成的入口包装成页面，元数据默认为空。`tests/fixtures/articles/bare.typ` 演示没有库依赖的正文及可查询的自定义 metadata。

未提供 `summary.typ` 时，将输入目录下每个 Typst 文件当作文章（输入目录根部的 `summary.typ` 保留为导航配置）；共用基础库应放在文章目录之外。Rust 负责文件发现与路由，`src/bundle.typ` 负责文章包装，`src/render.typ` 统一应用 Typst 渲染设置。高亮模式由入口应用，作者自身的局部设置可以覆盖它。Rust 报告中的 `type: page` 是默认约定，尚未实现其他文章类型。

本阶段不实现基础库、客户端高亮脚本、章节折叠或反向链接。未来基础库可以用标准 `metadata` 元素暴露可选扩展的数据，而不改变普通正文的写法。

`SiteWorld` 使用 typst-kit 提供的标准文件与 package 加载器、系统日期，以及内嵌与系统字体；package 使用默认目录，并按需从 Typst Universe 下载。仍未实现清理旧输出和原子替换，输出行为受 Typst 实验性 bundle/HTML API 的约束。

## 模块

- `main.rs`：串联输入准备、编译、提取、导出、assemble 与写出。
- `options.rs`：引擎配置、TOML 加载、路径解析及 CLI 覆盖合并。
- `source.rs`：发现文章、检查路由冲突、生成内存入口；`summary.typ` 展开可选导航清单。
- `world.rs`：Typst 环境和内存入口加载。
- `inspect.rs`：metadata、TOC 和代码的独立查询函数。
- `assemble.rs`：页面组装、目录数据与锚点、导航数据和首页。
- `theme.rs`：动态主题配置合并和通用类型校验、运行时 manifest 加载、模板发现、资源读取及展示 context。
- `output.rs`：提供 bundle 导出和文件写出函数，写出最终站点和可选报告；assemble 由 main 独立调用。

## 调试输出

`cargo run -- --mode raw --output public-raw` 跳过主题 assemble，输出 Typst 原始 HTML、资源及每页旁的 `index.json` metadata，不生成主题资源或首页。默认 `--mode site` 输出组装后的站点。两种模式均支持 `--report`。建议使用不同输出目录；构建不会清理旧文件。

## 站点导航

所有页面共享左侧文章树，默认按输入目录的相对目录分组，目录使用文件夹名，文章使用 `meta.title`，缺失时尝试首个且唯一的 H1，再回退到文件名（不含扩展名），当前文章高亮。右侧为本页 TOC。侧栏内部按钮可分别折叠内容，收起后保留展开按钮，桌面偏好保存在浏览器本地。1100px 及以下默认收起，点击按钮打开侧边面板，同一时间只展开一侧，Escape 可关闭。目录分支使用原生 details，可独立折叠。无自定义根页面时，自动生成带文章数量、主题分组和路径的首页。

### 自定义 summary.typ

在输入目录根部添加 `summary.typ` 并导出 `summary` 数组，即可指定要编译的文章、导航分组、显示标题与顺序。该文件通过 Typst 正常求值，可以使用变量和表达式；它自身不会生成页面。未添加该文件时，保持自动发现所有文章的行为。

例如 `content/summary.typ`：

```typst
#let summary = (
  (title: "编程语言", children: (
    (path: "pl/monad-scala-perspective.typ", title: "从 Scala 理解 Monad"),
    "pl/lambda-calculus-y-combinator.typ",
    "pl/scala-currying-partials.typ",
  )),
  (path: "notes", title: "读书笔记", children: (
    (path: "notes/ddia", title: "数据密集型应用"),
    "notes/intro-algo",
  )),
  "math",
)
```

- 页面用源文件路径表示：字符串形式使用文章 `meta.title`；字典形式可以通过 `title` 覆盖导航标题。未定义文章标题时尝试首个且唯一的 H1，仍无标题则使用文件名。
- 分组支持 `collapsed: true`，控制侧栏中该文件夹的初始折叠状态；默认 `false`（展开），可点击展开或折叠。手动分组和自动展开的目录均支持，选项只作用于该分组，不向内部子目录继承；首页列表仍完整展示。例如 `(path: "notes/ddia", title: "设计数据密集型应用", collapsed: true)`。当前 `content/summary.typ` 将所有最末层文件夹设为默认折叠，上层分类保持展开。
- 分组用 `children` 数组表示，可任意嵌套和重组文章。分组标题优先使用 `title`，否则使用 `path` 最后一段的文件夹名。`path` 不限制其 children，也不改变 children 的路径解析。
- **所有路径都相对输入目录**，使用 `/` 分隔，文章路径包含 `.typ` 或 `.typst` 扩展名。文章和分组按照数组中的顺序显示，允许页面与分组交错排列，首页和侧栏使用相同顺序。
- 目录项不写 `children` 时，自动递归选择该目录下所有文章，按相对路径的字典序排列并保留内部子目录。目录标题可覆盖，内部目录默认用文件夹名，文章默认用 `meta.title`。目录路径末尾可带 `/`；`"."` 可展开整个输入目录。
- 有 summary 时只编译清单直接列出或由目录展开选中的文章；未选中的文章不参与编译。缺失或没有文章的路径、重复选中同一篇文章会报错。
- 导航重组和标题覆盖不改变文章 URL、文章内容或文档标题。`--input path/to/bundle.typ` 显式入口继续由该 bundle 自己决定输出，不自动加载 summary。

例如只想把 `notes` 显示为「读书笔记」，并保留其中的自动排序，可以写 `#let summary = ((path: "notes", title: "读书笔记"),)`。注意这会只选择 `notes` 下的文章；其他目录需另外列出。输出目录不会清理旧文件，删掉清单中的文章后应清理旧输出再发布。

## 页面模板

当前主题的数据 schema、组件上下文、资源与 DOM/JS 约定见 [主题接口文档](docs/theme-interface.md)。

仓库两套主题通过各自 manifest 选择 `page.hbs` 页面骨架并复用其余 partial。`theme/default/templates/` 包含 `page.hbs`（公共骨架）、`article.hbs`（正文包装）、`index.hbs`（首页）、`tree.hbs`（递归文章树）、`toc.hbs`（递归目录）、`index-tree.hbs`（首页分组列表）。Rust 负责准备 PageView、TreeView 和 TocItem 等展示结构；模板不直接读取 Typst 原生 metadata。Handlebars 在渲染边界通过 Serde 转换数据，不写中间 JSON 文件。

模板每次构建从磁盘读取并注册一次，开启严格模式。按 manifest 的 template-dirs 顺序覆盖同名模板，修改 HBS 不需要重新编译。普通字段使用双花括号自动转义，仅 Typst 导出的 `head_html` / `body_html` 使用三花括号保留 HTML。URL 编码和资源版本号由 Rust 计算。

TOC 使用 `text-wrap: balance` 均衡多行文本，项内行距小于项间距；旧浏览器会回退到普通换行。
