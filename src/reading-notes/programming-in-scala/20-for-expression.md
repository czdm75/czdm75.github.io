# 20. for 表达式

for 表达式会被翻译成对应的 `map` `flatmap` `filter` `foreach` 的序列。也可以只实现其中的一部分：

- 实现 `map`，允许单个生成器的 for 表达式
- 实现 `map` 和 `flatMap`，可以允许多个生成器的 for 表达式
- 实现 `foreach`，可以允许 for 循环
- 实现 `filter`，可以允许 for 表达式中的 `if` 条件

其中，通过将 `flatMap` 翻译为 for 表达式，实现了对 Monad 的访问的比较好的语法糖。

以一系列在一个两层嵌套的列表 `val ll = List(List(1), List(2))` 上的操作为例，对于这样的 for 表达式：

```scala
for { a <- ll } yield a.toArray
for { a <- ll; b <- a if b % 2 == 0 } println(b)
```

等同于：

```scala
ll map { _.toArray }
ll flatMap { _ filter (_ % 2 == 0) } foreach println
```

比较特殊的情况是，for 表达式中生成器的左边不是一个简单的变量名而是一个模式。为了避免抛出 `MatchError`，需要先对是否符合模式进行一次判断，因为在 for 表达式中，不能匹配的元素将会被直接抛弃。

```scala
val ll = List(List(1, 2), List(2))
for { List(a, b) <- ll } yield a + b
ll map { case List(a, b) => a + b }  // MatchError
ll filter { case List(_, _) => true; case _ => false } map { case List(a, b) => a + b }  // Works as for expression
```

现在我们终于可以解释 for 表达式强大的适配能力和"自动选择合适的结果集合类型"的能力了。实际上，这些工作都是由 `map` `flatMap` `filter` 这些方法完成的。
