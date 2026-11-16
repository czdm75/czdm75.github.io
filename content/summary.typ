// 路径均相对 content；数组决定导航顺序，未覆盖的文章标题使用 meta.title。
// 最后一层文件夹默认折叠，保留上层分类展开。
#let summary = (
  (path: "cs", title: "计算机系统", children: (
    (path: "cs/io", title: "I/O 与网络", collapsed: true, children: (
      "cs/io/linux-io-multiplex.typ",
      "cs/io/java-nio-1.typ",
      "cs/io/java-nio-2.typ",
    )),
  )),
  (path: "pl", title: "编程语言", collapsed: true, children: (
    "pl/lambda-calculus-y-combinator.typ",
    "pl/scala-currying-partials.typ",
    "pl/monad-scala-perspective.typ",
  )),
  (path: "distributed", title: "分布式系统与大数据", collapsed: true, children: (
    "distributed/hadoop-basic.typ",
    "distributed/spark-rdd.typ",
    "distributed/spark-sql.typ",
  )),
  (path: "math", title: "数学与统计", collapsed: true),
  (path: "notes", title: "读书笔记", children: (
    // 数字文件名的书籍章节按路径自动排序。
    (path: "notes/intro-algo", title: "算法导论", collapsed: true),
    (path: "notes/core-java-impatient", title: "写给大忙人看的 Java 核心技术", collapsed: true),
    (path: "notes/in-depth-jvm", title: "深入理解 Java 虚拟机", collapsed: true, children: (
      "notes/in-depth-jvm/memory-region.typ",
      "notes/in-depth-jvm/gc.typ",
      "notes/in-depth-jvm/memory-model.typ",
      "notes/in-depth-jvm/synchronization.typ",
      "notes/in-depth-jvm/threadlocal-reference.typ",
    )),
    (path: "notes/programming-scala", title: "Scala 编程", collapsed: true),
    (path: "notes/ddia", title: "设计数据密集型应用", collapsed: true),
  )),
  (path: "etc", title: "杂谈", collapsed: true, children: (
    "etc/tiktok-in-my-memory.typ",
  )),
)
