# Typst SSG 实验

目前保留 Typst bundle 作为编译与导出后端。Rust 程序直接调用 Typst 0.15.1 的库，检查编译结果，再通过 `typst_bundle::export` 输出文件，不启动 Typst CLI。

## 使用

在仓库根目录执行（需要 Rust 1.92 或更新版本）：

```sh
cargo run -- --input content --output public --math mathml --code-renderer typst
cargo run -- --input content --output public --math svg --code-renderer highlight-js
cargo test
```

命令行参数统一由 Clap 的 `Options` 结构体解析；所有配置使用 `--参数`，`--input` 与 `--output` 默认为 `content`、`public`，不再支持位置参数或 `compiled` / `client` 旧别名。可用 `cargo run -- --help` 查看帮助。

渲染选项统一在 `Options` 中：

- `--math mathml|svg`：默认 `mathml`。`svg` 使用 Typst `html.frame` 渲染行内与块级公式。
- `--code-renderer typst|highlight-js`：默认 `typst`。`highlight-js` 目前仅关闭 Typst 高亮，保留 `data-lang` 和代码文本，不加载脚本或 CSS。
- `--code-theme PATH`：`typst` 模式读取 `.tmTheme` 文件，路径相对项目根目录，也可使用绝对路径；不指定时使用 Typst 默认主题。`highlight-js` 模式只保留该配置，暂不读取或应用主题。

Typst 主题控制语法颜色，不自动应用代码块的前景色和背景色。文章自己的局部规则仍可覆盖站点默认设置。显式 bundle 入口需使用 `src/render.typ` 中的 `render` show rule 才能应用这些选项。

`cargo run` 或 `mise run build` 默认构建完整站点，使用 `--report` 时额外生成 `public/ssg-report.json`；默认第一阶段构建输出 Typst 原始 HTML，以及每篇文章旁的 `index.json`。JSON 直接从结构体序列化，不构造 `serde_json::Value` 中间树。

例如 `content/cs/io/java-nio-1.typ` 输出：

```text
public/cs/io/java-nio-1/index.html
public/cs/io/java-nio-1/index.json
```

`index.html` 保留 Typst 的完整输出，不额外加入站点布局或样式。`index.json` 包含该页面的路径、类型、标题、metadata、TOC 和代码信息，可供后续阶段消费。

输入为目录时，递归发现其中的 `.typ` / `.typst`，排序后按相对路径生成 `目录/文件名/index.html`。图片等资源由 Typst 在编译时读取。符号链接不参与发现；两个源文件映射到同一路由时提前报错。入口源码仅存在内存中，无需磁盘上的入口文件。

也可以通过 `--input path/to/bundle.typ` 指定自定义 bundle 入口。

## 能读取的信息

每个 HTML 文档默认视为 `page`，检查结果使用 `Page`、`Metadata`、`Heading` 和 `Code` 结构体。`Metadata` 保留原生 Typst `Value`，允许自定义扩展携带任意 Typst 数据。第一阶段将原生值序列化到 JSON；函数等类型按 Typst 的默认行为输出为文本表示。序列化错误会在写文件之前报出。使用 `cargo run -- --input content --output public --report` 输出报告，其中包括：

- `path`、`title`：bundle 的输出路径和原生 `DocumentInfo` 标题。
- `metadata`：文档查询得到的所有 `metadata` 值，检查阶段保留原生值，输出 JSON 时进行序列化。入口把已有文章导出的 `meta` 字典转换为一个不可见的 `metadata` 元素，因此包含 `title`、`lang`、`year` 及以后新增的字段。
- `toc`：通过 Typst 的 `HeadingElem` 查询得到的层级、纯文本与现有跳转锚点。包含程序生成的标题，不依赖源码扫描。没有被引用的标题可能没有锚点，此时为 `null`。
- `code`：通过 introspector 查询 `RawElem`，提取原始文本、语言和 `block`（是否为块级代码），包含行内代码。即使 show rule 改变了 HTML 表现，仍可读取原始代码语义。

## 作者接口与边界

正文仍是标准 Typst，不要求引入基础库。现有文章可以保留 `#let meta = (...)`；不导出 `meta` 的文章也能由生成的入口包装成页面，元数据默认为空。`tests/fixtures/articles/bare.typ` 演示没有库依赖的正文及可查询的自定义 metadata。

默认将输入目录下每个 Typst 文件当作文章；共用基础库应放在文章目录之外。Rust 负责文件发现与路由，`src/bundle.typ` 负责文章包装，`src/render.typ` 统一应用 Typst 渲染设置。高亮模式由入口应用，作者自身的局部设置可以覆盖它。Rust 报告中的 `type: page` 是默认约定，尚未实现其他文章类型。

本阶段不实现基础库、导航 UI、客户端高亮脚本。未来基础库可以用标准 `metadata` 元素暴露可选扩展的数据，而不改变普通正文的写法。

`SiteWorld` 使用 typst-kit 提供的标准文件与 package 加载器、系统日期，以及内嵌与系统字体；package 使用默认目录，并按需从 Typst Universe 下载。仍未实现清理旧输出和原子替换，输出行为受 Typst 实验性 bundle/HTML API 的约束。

## 模块

- `main.rs`：串联输入准备、编译、提取与输出。
- `options.rs`：Clap 参数。
- `source.rs`：发现文章、检查路由冲突、生成内存入口。
- `world.rs`：Typst 环境和内存入口加载。
- `inspect.rs`：metadata、TOC 和代码的独立查询函数。
- `output.rs`：导出 bundle 和 JSON 报告。
