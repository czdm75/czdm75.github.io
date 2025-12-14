# 2. 基础类型、类和对象

## 单例对象

对于单例对象，`scalac` 会编译成一个名为 `ObjectName$` 的 class 文件。如果需要进行大量的编译而不希望每一次调用 `scalac` 命令都要重新寻找 classpath 等，可以使用 `fsc` 命令来编译。在调用 `fsc` 之后，会拉起一个守护进程，再次调用 `fsc` 就会将源文件发送到这个守护进程的端口上。最后，使用 `fsc -shutdown` 停止守护进程。

除了使用 `main` 函数，Scala 还提供了一个用于创建 App 的 Trait ：

```scala
object A extends App {
  args foreach println
}
```

## 基础类型

```scala
0x00FF  // Int = 255
0xCAFEBABE  // Int = -889275714
0xCAFEBABEL  // Long = 3405691582
1.23e-2  // Double = 0.0123
1e2  // Double = 100.0
'\u0041'  // Char = A
```

## 字符串

Raw 字符串

```scala
"""AB
  BA"""
// "AB\n  BA"

"""ab
  |ba""".stripMargin
// "AB\nBA"
```

可以看到，RAW 字符串里保留了空格和换行符，如果在定义中使用管道符并就能通过 `stripMargin` 避免将缩进用的字符串包括在内。

## 字符串插值器（Interpolater）

Scala 定义了三个字符串插值器。`f` 允许使用 C 风格的格式化，`raw` 中的转义符不会生效，`s` 是通常的插值器。由于 `f` 被实现为了宏，它可以在编译期进行类型检查，`printf` 则不能。

```scala
print(f"hello, $name, ${age + 0.5}%7.2f")
print(s"abd $name")
print(raw"\t\n")
```

也可以自己定义插值器。

```scala
import java.time.LocalDate
implicit class DateInterpolator(val sc: StringContext) extends AnyVal {
  def date(args: Any*): LocalDate = LocalDate.of(
    args(0).toString.toInt,
    args(1).toString.toInt,
    args(2).toString.toInt)
}

val y = 2018
val m = 4
val d = 5

date"$y, $m, $d"  // java.time.LocalDate = 2018-04-05
```

## 符号 Symbol

Symbol 对象的作用和 Java 中的 Interned String 类似，通过一个单引号来声明。对于一般的字符串，频繁使用字面量可能造成大量字符串对象的创建，给系统的性能带来压力，解决办法是将其放入常量池，和 JVM 对数字的处理一样。

对于 Symbol 来说，当符号创建时，实际上调用了 `Symbol.apply(name: String)` 在常量池中建立了一个对象，之后再次使用时引用的将会时同一个对象。这样，在比较时避免了字符串的重复构造和遍历，而是直接比较地址即可。

```scala
'abc  // Symbol = 'abc
'abc.name  // String = abc
```

## 操作符

对于超过一个参数的方法，可以将参数用括号包起来使用中缀表达：

```scala
object A {
  def op(i: Int, j:Int): Unit = print(i + j)
}
A op (1, 2)
```

Scala 只允许四个前缀操作符：`+ - ! ~`，使用 `unary_` 来定义：

```scala
class Num(val a: Int) {
  def unary_-(): Num = new Num(-a)
}
-(new Num(3)).a
```

后缀操作符的形式是没有参数的方法。通常，当方法有副作用时保留括号来调用，而在方法没有副作用时不适用括号，使其看起来就像对变量成员的访问一样：

```scala
"ABC".reverse
```

Scala 的 `==` 方法调用了 `equals` 方法，但是 null 安全的。`eq` 和 `ne` 方法比较引用。因此，Scala 程序惯用 `==` 来比较对象，编写 `equals` 就更加重要。

因为使用函数作为操作符，操作符的有限集比较复杂。除了正常可以理解的，`*` 高于 `+` 这类规则外，对于自己定义的操作符，Scala 使用第一个字符来判断，这样做的目的是，例如我们自己定义了 `**()` 和 `++()` 方法，那么 `**()` 的优先级更高，这符合我们的心理预期。

另外任何赋值操作符的优先级与正常的赋值操作符 `=` 相同。也就是说，`+=` `-=` 这类以等号结尾的操作符优先级与赋值操作符相等，无论首字母是什么。因此，在 Scala 类的操作符的定义中，遵循公共的常规约定十分重要。

对于这些特殊字符，Scala 会将其转换为一定的字符串，以和 Java 兼容。例如 `:->` 需要在 Java 中使用 `$colon$minus$greater` 来访问。

对于基本类型的包装类，Scala 实际上将复杂的操作定义在了它们对应的富包装类中，即 `RichInt` `StringOps` 等类，并提供隐式转换。

## Application 特质

最后简单介绍一下 App 特质。这个特质可以这样使用：

```scala
object MyApp extends App {
  for (arg <- args)
  println(arg)
}
```

这样编写之后，这个程序就可以正常地被编译和运行。只需要在命令行 `scala MyApp` 即可，还可以正常地使用命令行参数，方便创建简单的程序。这样做的原理是，这个对象继承了 `App` 这个 trait，这些代码会被父类的 `main` 函数中被调用，这里不做过多解释。
