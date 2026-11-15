# SSG 插件设计草案

状态：设计提案，尚未实现。目标是将现有渲染选项、Typst 规则和 inspection 抽成内建插件，并为后续 HTML assembly 留出接口。

## 1. 基本边界

一个插件组织同一项能力的四部分：

1. **Typst 代码**：可选的作者 API，以及构建时自动应用的规则。
2. **Options**：Rust 中的强类型配置；解析、校验后转换为传给 Typst 的原生值。
3. **Inspection**：从编译结果查询得到的强类型数据。
4. **HTML assembly**：使用 inspection 和已解析配置，提出对最终页面、资源的修改。

四部分属于同一插件，但允许某部分为空。例如 MathML 模式可以不添加任何 Typst 规则，Math 插件的 assembly 也可以直接返回空贡献。

第一版采用随程序编译的 Rust 内建插件。不引入动态库、WASM、外部插件发现、运行时 CLI 注册或依赖解析器。这里的 SSG plugin 与 Typst 的 WASM plugin 不是同一种机制。

正文保持 bare Typst 兼容。标准公式、raw 和 heading 无需 import 即可使用；未来扩展语法由可选的 Typst 库提供，不成为文章参与构建的前提。

## 2. 生命周期

```text
CLI Options
  → 配置解析、资源读取与校验
  → 插件 Prepared 数据：Typst 模块 + inputs + 已解析配置
  → Rust 发现文章并生成 bundle 入口
  → World 加载虚拟模块，Typst 编译
  → 插件 inspection
  → 第一阶段：原始 HTML + 每页 JSON
  → 第二阶段：HTML assembly + 页面模板 + 静态资源
```

第一阶段保留 Typst 的实际导出表现，例如 SVG 公式、编译期代码颜色。第二阶段负责页面组装，例如导航、脚本和 CSS。插件接口允许 assembly 暂不实现；迁移现有代码时不需要同时实现客户端高亮。

同一次构建中，各阶段直接传递结构体。JSON 是第一阶段的输出边界，不是插件之间的内部通信格式。普通代码不构造 `serde_json::Value`。

## 3. 核心与插件的职责

| 核心保留 | 下放插件 |
| --- | --- |
| 文件发现、路由和输出路径校验 | 公式格式、代码高亮器、主题 |
| World、字体和 package 加载 | 特性相关的 inputs 与 Typst 规则 |
| bundle 编译与导出 | 特性相关的查询和 inspection 类型 |
| 页面身份、标题、作者原始 metadata | TOC、代码信息及未来扩展信息 |
| 插件调度、顺序和资源冲突检查 | 页面修改和所需静态资源 |
| JSON 输出格式与版本管理 | 自己的数据与配置版本 |

`#let meta` → `metadata(meta)` 的兼容桥接先保留在核心 `bundle.typ`。它是所有页面的基础约定，不必为了统一形式立刻变成插件。

## 4. 初始插件

### Math

- Options：`MathOptions { format: MathFormat }`，默认 MathML。
- Typst：迁入当前 `render.typ` 的公式规则；SVG 模式保留行内和块级公式的区别。
- Inspection：第一版只记录实际采用的格式，不预先设计尚无需求的公式源码、AST 或公式列表。
- Assembly：当前为空。以后如需额外数学样式或资源，再由该插件声明。

### Code

- Options：`CodeOptions { renderer: CodeRenderer, theme: Option<PathBuf> }`。
- Typst：Typst 模式设置 raw 主题；highlight.js 模式设置 `raw(theme: none)`。
- Inspection：迁入现有 `Code` 查询，得到原始文本、语言、block 标记；同时保留后续 assembly 所需的已解析配置。
- Assembly：后续在 highlight.js 模式中适配语言标记、声明脚本和主题 CSS；本轮迁移返回空贡献。

代码主题与 renderer 绑定：Typst 的 `.tmTheme` 与 highlight.js 的 CSS 是不同的输入。准备阶段生成 `ResolvedCodeOptions`，不用一个未经解释的字符串贯穿所有阶段。主题文件读取由 Code 插件负责，World 不识别主题含义。

### TOC

- Options：先使用空配置；以后有明确需求再加入最大深度等参数。
- Typst：可为空，普通 heading 已可查询。
- Inspection：迁入 `Heading` 查询，后续支持 `outlined` 等 Typst 语义。
- Assembly：以后根据标题目标插入目录，不通过 HTML 标签再次推导文档结构。

## 5. Options 与 World

CLI 继续由 Clap 静态声明，用 `#[command(flatten)]` 组合插件自己的 `Args` 结构体，保留现有参数名：

```rust
struct Options {
    input: PathBuf,
    output: PathBuf,
    report: bool,
    math: MathOptions,
    code: CodeOptions,
}
```

上述代码为结构示意，省略 Clap 属性。`--input`、`--output` 属于核心，`--math` 属于 Math，`--code-renderer` 和 `--code-theme` 属于 Code。

准备阶段将 Options 转成插件自己的 resolved config，读取并记录所需文件。传给 Typst 的值使用命名空间：

```text
sys.inputs.plugins.math.format
sys.inputs.plugins.code.renderer
sys.inputs.plugins.code.theme
```

这些仍是 Typst 的 Dict、Bytes 等原生值，不经过 JSON。主题等二进制资源也不塞入源码字符串。

World 接收已聚合的 inputs 和虚拟源文件，不再接收整个 CLI Options，也不包含 Math/Code 分支。插件模块放入受核心管理的虚拟路径，例如 `/.ssg/plugins/code/lib.typ`，由 World 的内存文件表加载；核心检查模块路径与插件 ID 冲突。

## 6. Typst 模块约定

插件模块可以提供统一的构建入口：

```typst
#let apply(body, options: (:)) = {
  // 插件自己的 set / show rules
  body
}
```

核心生成 Typst import 和调用，将插件按注册顺序包装文章。模块作用域隔离局部名称，配置通过 `options` 显式传入；插件不读取其他插件的私有 inputs。

作者 API 与自动包装入口分开。作者只有使用自定义扩展时才需要 import 基础库。插件开启与否控制是否应用自动规则，不能仅仅通过“不生成作者 import”来表达禁用。

第一版使用明确的静态顺序：Math → Code → TOC。包装的嵌套顺序必须在生成器中固定并测试，不能依赖 map 的遍历或拼接顺序。文章内部更局部的规则仍允许覆盖插件默认值；CLI 配置表达站点默认策略，不保证作者无法覆盖。

## 7. Rust 接口与类型组织

建议先用关联类型表达插件边界。以下为接口草图，不是当前可编译 API：

```rust
trait Plugin {
    type Options;
    type Inspection: Serialize;

    fn id(&self) -> &'static str;
    fn options(&self) -> &Self::Options;
    fn prepare(&self, ctx: &PrepareContext) -> Result<PreparedPlugin>;
    fn inspect(&self, ctx: &InspectContext) -> Result<Self::Inspection>;
    fn assemble(
        &self,
        ctx: &AssemblyContext,
        inspection: &Self::Inspection,
    ) -> Result<AssemblyContribution>;
}
```

`prepare` 返回拥有数据所有权的配置、模块和资源；`inspect` 返回结构体；`assemble` 返回贡献。上下文仅借用已有数据。尽量采用 map、filter_map、collect 和返回值组合，不要求插件修改共享 Vec、JSON 对象或 HTML 树。

上下文只提供本阶段需要的信息：

- Prepare：项目根目录和资源读取能力。
- Inspect：当前文档 introspector、bundle introspector、页面身份，以及该插件自己的 prepared 数据。
- Assembly：页面身份、实际 HTML 节点、该插件自己的已解析配置及资源 URL 解析能力；不重新开放编译环境。

接口草图中的 prepared 数据暂使用公共结构承载模块、inputs 和资源；如果后续各插件需要不同的 prepared 类型，再将其提升为关联类型。

关联类型意味着不能直接把这些插件装进未经类型擦除的 `Vec<Box<dyn Plugin>>`。第一版明确采用静态 registry 和强类型聚合结构：

```rust
struct PageInspection {
    core: PageCore,
    plugins: BuiltinInspection,
}

struct BuiltinInspection {
    math: Option<MathInspection>,
    code: Option<CodeInspection>,
    toc: Option<TocInspection>,
}
```

关闭的插件对应 `None`，不执行自己的规则、查询或 assembly。新增内建插件需要更新 registry 和聚合结构，这是第一版接受的编译期成本。以后确有第三方注册需求，再在 registry 边界增加类型擦除和序列化适配，不提前将插件结果退化为 JSON map。

## 8. Inspection 与 JSON 边界

当前 `Metadata` 继续保留 Typst 原生值，用于作者自定义字段；插件自身的 inspection 使用自己的 Rust 类型。

每页 JSON 后续可以按插件 ID 分组，并加上版本：

```text
schema_version
core: path / type / title / metadata
plugins:
  math: version / config / data
  code: version / config / data
  toc:  version / config / data
```

配置快照记录实际采用的 renderer、格式和资源引用，避免第二阶段重新使用 CLI 默认值，导致与第一阶段产物不一致。不要仅记录主题的本机绝对路径：后续需要独立 assembly 时，应输出对应资源或构建资源清单，让产物自包含。

迁移初期继续保持现有 `index.json` 字段，等第二阶段真正消费插件分组后再升级格式。

同进程流程不要求 inspection 能从 JSON 恢复。若支持独立执行第二阶段，则各插件另外定义可反序列化、版本化的磁盘结构；Typst 原生 Value 的文本回退是有损的，不能承诺函数等值会被还原。第二阶段所需数据必须有明确的持久化表示。

## 9. HTML assembly 的约束

Assembly 处理导出前的 HTML DOM，或在独立第二阶段解析第一阶段 HTML；这两个入口最终使用同一种页面修改模型。

插件返回 `AssemblyContribution`，可以包含：

- 页面 head 资源引用和脚本初始化要求。
- 静态资源及其逻辑身份。
- 对已知目标的节点修改，例如给 code 添加语言 class、插入 TOC。

核心负责去重、资源 URL、输出路径校验和应用修改。相同身份的同内容资源只输出一次；内容不同或两个插件争用同一节点的排他修改时明确报错。第一版不提供任意字符串替换 HTML 的 hook。

必须先解决语义元素与 HTML 目标的对应关系：Typst location 只适用于当前编译结果，JSON 中的标题文本和代码内容也不足以唯一定位节点。需要修改节点时，在第一阶段导出前分配稳定的 target ID，并记录到 inspection；保留已有 ID，检查生成 ID 冲突。当前没有链接的标题可能没有 anchor，这个问题不能留给 TOC 插件猜测。

给语义目标分配 ID 是第一阶段对导出结果的一项明确增强，不能在实现后继续声称 HTML 字节完全未修改；当前尚未实现，第一阶段仍保持现状。

作者自定义 show rule 可能使 raw 不再输出为 code。Code 插件应只修改确认存在且符合约定的代码节点，不能根据语义记录把作者自定义 HTML 强行还原。标题同理：可通过目标标记定位，但不能假设 HeadingElem 必然对应 h1–h6。

## 10. 渐进迁移

1. **拆配置**：将 MathOptions、CodeOptions 放到各自模块，顶层 Options flatten；CLI 名称、默认值和 HTML 输出保持不变。
2. **拆 Typst**：将 render.typ 拆为插件 apply 模块；准备阶段聚合 inputs，World 改为只接收基础环境与 prepared 数据。
3. **拆 inspection**：迁移 Code 与 TOC 的类型和查询；引入静态 registry，保留现有 JSON 格式。
4. **补目标映射**：为需要 assembly 的元素提供稳定 target ID，覆盖重复标题、重复代码和自定义 show rule。
5. **实现 assembly**：先以 Code 的 highlight.js 分支验证资源、主题和节点修改；Math 的 assembly 保持为空。
6. **稳定磁盘协议**：有独立第二阶段需求时，再升级 JSON 版本和插件分组、实现插件磁盘类型的读取。

每一步只引入当时需要的接口，不要求为了完成配置拆分先构建通用插件平台。

## 11. 验收重点

- 无作者 import 的文章仍能构建；启用插件不要求修改正文。
- 默认输出与迁移前一致，MathML/SVG、Typst 主题和无色代码都有覆盖。
- 插件关闭后不执行对应查询，缺省能力有明确的 bare Typst 行为。
- inputs、虚拟模块、资源输出冲突能明确报错；插件顺序固定。
- inspection 保持强类型，序列化只发生在输出边界。
- 后续 assembly 能区分重复目标，尊重已有锚点和作者自定义渲染。
- 仅声明同进程支持时，不暗示第一阶段 JSON 已足以跨进程恢复全部 Typst 值。
