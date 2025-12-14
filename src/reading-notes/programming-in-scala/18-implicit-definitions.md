# 18. 隐式定义

## 隐式转换

在使用隐式转换时，编译器会首先尝试编译。如果类型不能匹配，编译器会在作用域中寻找合适的隐式转换。查找隐式转换的范围是，作用域中所有的"单个标识符"，也就是直接定义在作用域里，而不是某个对象里，再加上源类型与目标类型的伴生对象里。例如，当前作用域下一个变量 `someVar.String2Int` 不会被搜索，但 `String` 类和 `Int` 类对应的伴生对象中的方法都会被搜索。此外，Scala 的隐式转换只能进行一次，不会发生难以控制的链式多次转换的情况。也不会覆盖显式的定义，只要能通过类型检查，就不会调用隐式转换。

例如，`Predef` 中定义了数字之间互相转换的函数。再一次，Scala 通过一个通用的语言特性解决了一种"特殊情况"。（虽然 Scala 编译器仍然进行了特殊情况的处理，生成了效率更高的字节码）。

隐式定义会出现在三个地方：转换到预期的类型，选择接收端和隐式参数。分别对应这样的方式：

```scala
import scala.language.implicitConversions
class A
class B { def run = println("B.run") }
implicit def a2b(x: A): B = new B
implicit val y = new B
def fun(x: B) = println("obj B in fun")
def func(x: A)(implicit y: B) = y.run

fun(new A)   // converting to a expected type
// obj B in fun
(new A).run  // converting the receiver
// B.run
func(new A)  // implicit parameters
// B.run
```

隐式转换适合用来创建 DSL。例如，用来创建 Map 的语法就是使用隐式转换制作的：

```scala
Map(1 -> "one", 2 -> "two")

package scala
object Predef {
  implicit final class ArrowAssoc[A](private val self: A) extends Anyval {
    @inine def -> [B](y: B)Tuple2[A, B] = Tuple2(self, y)
  }
}

// or, seperated class and function
object Predef {
  final class ArrowAssoc[A](private val self: A) extends Anyval {
    @inine def -> [B](y: B)Tuple2[A, B] = Tuple2(self, y)
  }
  implicit def any2ArrowAssoc[A](x: A): ArrowAssoc[A] = new ArrowAssoc
}
```

其中上面一种方式称作隐式类，相当于一个类定义和一个以其构造方法为形式的隐式转换函数。显然其构造方法必须是单参数的。Scala 还限制隐式类必须存在于另一个对象、类或特质里，这样就一定程度上限制了隐式类的滥用。

## 隐式参数

隐式参数常被用在提供一个多次用到的通用值的情形。例如定义一个命令提示符：

```scala
class PreferedPrompt(val preference: String)
def printWithPrompt(str: String)(implicit prompt: PreferedPrompt) =
  println(prompt.preference + " " + str)
implicit val prompt = new PreferedPrompt(">")

printWithPrompt("run")(prompt)
printWithPrompt("run")
```

常见的方式是将默认值放在一个对象中，再 `import obj._`。

同时，我们在这里专门为隐式参数定义了一个类，而不是使用 String。这是为了避免不必要的额外匹配的风险，因为隐式转换**只寻找类型**而不判断变量名。隐式参数在 Scala 中最常见的场景是用于排序，排序函数的第二个参数列表里通常是一个 `Ordering[T]` 对象。显然，这里的功能只需要一个 `(T, T) => Boolean` 类型的参数就能完成，但这样的类型太过泛化，风险比较高。

考虑一个常见的排序函数的递归实现：

```scala
def sort[T](l: List[T])(implicit ordering: Ordering[T]) = {
  ...
  sort(...)
  if (ordering.gt(...))
  ...
}
```

和我们的直觉相符，Scala 会直接把外层的函数接收的 `ordering` 作为隐式参数继续使用，也可以直接去调用这个参数。那么，有没有可能不去显式地写 `ordering` 这个变量名呢？（毕竟它是隐式的）

```scala
def sort[T](l: List[T])(implicit ordering: Ordering[T]) = {
  ...
  if (implicitly[Ordering[T]].gt(...))
  ...
}
```

`implicitly` 是 Scala 定义的一个用于查找一定类型的隐式参数的函数。现在我们发现，有了这种查找方式，我们实际上已经不再需要 `ordering` 这个变量名了。所以，进一步地，我们可以直接去修改类型参数：

```scala
def sort[T: Ordering](l: List[T]) = ...
```

这样定义意味着要求类型参数 `T` 必须有相应的 `ordering`。一个很好的性质是，通过这种方式，我们并不需要修改 `T` 类型。例如，一个外部库里有一个类型 A，当我们对 A 排序时，不需要修改 A 使其实现 `Comparable`，而只需要提供一个 `Ordering[A]`。

另外一个需要解决的问题是隐式定义的冲突。在 Scala 2.8 之后，采用了和方法重载类似的方式：优先选择"更具体"的那一个。如果同样具体，就需要手动指定。在静态类型语言中，这并不是什么大问题：

```scala
implicit val v: AnyVal = 1
implicit val i: Int = 2

implicitly[Int]     // 2
implicitly[AnyVal]  // 2, which is the more specific one

implicit val ii: Int = 3
implicitly[Int]  // ambiguous implicit value 2 and 3 with same weight
// <console>:15: error: ambiguous implicit values:
//  both value i of type => Int
//  and value ii of type => Int
//  match expected type Int
//        implicitly[Int]
//                  ^
```

可以使用 `-Xprint:typer` 参数来查看发生的隐式转换。

## 隐式转换的优先级

并不是所有隐式转换都会显式发生冲突。例如，`String` 有两个隐式转换，一个转换成 `WrappedString`，其方法返回的仍然是 `WrappedString`。另一个是 `StringOps`，其返回值仍然是 `String`，第二个的优先级更高。这样，如果我们需要一个 `Seq`，会得到 `WrappedString`，否则仍然得到一个 `String`。这样的原因是 `StringOps` 的转换位于 `Predef`，`WrappedString` 的转换位于 `scala.LowPriorityImplicits`。Scala 选择隐式转换的原则是：

- 更具体类型的隐式转换优先级更高。
- 如果 `A extends B`，那么 `B` 中的优先级更高。

考虑我们定义一个类继承另一个类，那么子类中的隐式很可能更加是我们想要的。

```scala
implicit val a: String = "a"
implicit val b: CharSequence = "b"
def f(implicit s: String) = s

f  // a
```

```scala
class A { implicit val a: String = "a" }
object B extends A { implicit val b: String = "b" }
def f(implicit s: String) = s
import B._

f  // b
```
