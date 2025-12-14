# 22. 提取器 Extractor

## 简单的提取器

目前为止，我们见到的模式匹配都和 `case class` 一起出现，但这并不是必须的。实际上，`case` 关键字的作用只是提供 `unapply` 方法而已。下面是一段解析 Email 地址的代码：

```scala
object Email {
  def unapply(str: String): Option[(String, String)] = {
    val parts = str split "@"
    if (parts.length == 2) Some(parts(0), parts(1)) else None
  }
}

List("abc@efg", "abcdefg") flatMap { case Email(a, b) => Some(a + " AT " + b);  case _ => None }
```

这样，我们避开了 case class，并没有创建 Email 这样一个类，而是仍然使用 `String` 来表示，但同样完成了需要的功能。对于通常的 case class，在提供提取器的同时，也会把类本身的结构暴露给调用方。而使用提取器时，我们能够将背后的实现隐藏起来。这样，同时也达到了类似于 getter 的效果，我们可以任意修改后面的实现，只要提供对应的新版提取器即可，不用担心下游代码的兼容性。

当然，通常还会定义对应的 `apply`，并可以把对象直接声明为函数类型的子类：

```scala
object Email extends ((String, String) => String) {
  def apply(user: String, domain: String) = user + "@" + domain
  def unapply(str: String): Option[(String, String)] = ...
}
```

## 提取 0 个或 1 个元素的提取器

定义了 `unapply` 方法的对象就可以称作一个提取器 Extractor，无论有没有对应的 `apply` 方法。上面的例子返回多个变量，所以返回 `Option[Tuple]`。如果只返回一个，就无需使用元组。如果不提取任何值，就直接返回布尔值，不使用 `Option` 包装。这种情况通常很常见，尤其是当我们只是把提取器当做一个用来"检查"的工具。例如，一个检查全大写的提取器这样使用：

```scala
object UpperCase {
  def unapply(str: String): Boolean = str.toUpperCase == str
}
strList flatMap { case UpperCase() => Some("Y"); case _ => None}
```

最后，我们要考虑使用 `_*` 匹配剩余的多个元素的情况。显然之前的 `unapply` 已经满足不了我们的需求，Scala 在这里使用 `unapplySeq` 方法：

```scala
object Domain {
  def unapplySeq(whole: String): Option[Seq[String]] = Some(whole.split("\\.").reverse)
}
"abc.def.com" match { case Domain("com", "example", "www") => "example"; case Domain("com", _*) => "com"; case Domain(x, _*) => x }  // com
```

标准库中的 `List` `Array` 等提取器就是这样通过在伴生对象上实现的。

通常，使用 case class 能够带来略好的性能（因为 case class 的实现更加简单），并能够得到来自 `sealed` 关键字的编译错误的帮助，而提取器的灵活性更好。好在，由于调用方的代码并没有任何区别，如果仅仅用来进行模式匹配的话，二者可以无缝切换。

## 正则表达式

Scala 提供的正则表达式类位于 `scala.util.matching.Regex`，有多种方式来创建：

```scala
import scala.util.matching.Regex

new Regex("\\n[0-9]+")     // scala.util.matching.Regex = \n[0-9]+
new Regex("""\n[0-9]+""")  // scala.util.matching.Regex = \n[0-9]+
"[0-9]+".r                 // scala.util.matching.Regex = [0-9]+

"\\d+" findAllIn "ab123c123d"         // scala.util.matching.Regex.MatchIterator = <iterator>
"\\d+".r findFirstIn "ab123c123d"     // Option[String] = Some(123)
"[a-z]+".r findPrefixOf "ab123c123d"  // Option[String] = Some(ab)

val Decimal = """(-)?(\d+)(\.\d+)?""".r
val Decimal(a, b, c) = "-1.23"
// a: String = -
// b: String = 1
// c: String = .23
```

可以看到，我们可以直接使用正则表达式对象进行模式匹配，并通过正则表达式的分组来提取字符串对象，同样是通过 `unapplySeq` 方法实现的。
