# 12. 模式匹配

## 样例类 case class

```scala
abstract class Expr
case class Number(num: Double) extends Expr
case class UnOp(operator: String, arg: Expr) extends Expr
case class BinOp(operator: String, left: Expr, right: Expr) extends Expr
```

Scala 为一个 case class 提供了包括：

- 一个 `apply` 工厂方法，等同于 `def apply(num: Double) = new Number(num)`
- 一系列字段。等同于 `val num: Double`
- 正确实现的 `toString` `equals` `hashCode` 方法
- 一个 `copy` 方法，这个方法可以接收参数以产生部分不同的新对象。

最重要的是，样例类可以进行模式匹配。

## 模式

- 通配模式 `case _` 匹配任何对象，用于缺省捕获。
- 常量模式 `case 1` 仅匹配自己，也就是 `equals` 返回真值的对象。包括数字、字符串、单例对象、`val` 值等都可以。
- 变量模式 `case e` 变量模式也匹配任何对象，但这个变量名在后续的表达式中是有意义的，可以进行进一步处理。在区分常量模式和变量模式时，Scala 简单地使用首字母来判断。如果**首字母是大写，就认为常量**。必要时可以选择转义。
- 构造器模式 `case BinOp("+", e, Number(0))` 构造器可以进行深度匹配，例如这里嵌套的 `Number` 对象。
- 序列模式 `case List(0, a, _)`
- 元组模式 `case (0, a, _)`
- 类型模式 `case m: Map[_, _]` 在 Scala 中推荐使用类型匹配而非 `isInstanceOf[String]` `asInstanceOf[String]` 来判断类型。
- 变量绑定 `case BinOp("-", v @ Number(1), _)` `v` 可以作为变量使用。这样可以在变量模式的基础上进行匹配。

常量和变量的匹配顺序规则如下：大写开头作为常量，小写开头作为变量，加转义则变回常量，这是考虑常量的值是作用域中某个变量的情况。大写开头的变量则不被支持。

```scala
import math.Pi
val pi = 3.14
def f(n: Double) = n match {
  case Pi => Pi
  case `pi` => pi
  case pi => pi
}

List(Pi, 3.14, 3) map f  // List(3.141592653589793, 3.14, 3.0)
```

另一个问题是类型擦除。由于类型擦除，Scala 也没有办法准确地推断出一个泛型容器内部的类型。因此，一个匹配 `Map` 的模式匹配将能够接受所有的 `Map` 类型。

```scala
val f = (n: Any) => n match {case m: Map[Int, Int] => true; case _ => false}
// warning: non-variable type argument Int in type pattern scala.collection.immutable.Map[Int,Int] (the underlying of Map[Int,Int]) is unchecked since it is eliminated by erasure
//  (n: Any) => n match {case m: Map[Int, Int] => true; case _ => false}
//                               ^

f(Map(1 ->2))   // true
f(Map(1 ->""))  // true
```

## 模式守卫与模式重叠

在模式匹配中，模式需要是线性的，一个变量模式只能出现一次。如果我们要判断两个位置的值相等，就需要这样做：

```scala
def same(s: Any) = s match { case (x, x) => true; case _ => false }
// error: x is already defined as value x
//  def same(s: Any) = s match { case (x, x) => true; case _ => false }
//                                        ^

def same(s: Any) = s match { case (x, y) if x == y => true; case _ => false }
```

这种方式当然也能添加其他的条件。

模式重叠（Pattern Overlaps）指的是，在模式匹配中，排在上面的模式所覆盖的范围应该小于下面的，否则下面的模式就是 Unreachable Code。

## 封闭类

是否在模式匹配的最后使用 `case _ =>` 是一个选择。如果使用这样的语句，那么错误就有可能被隐藏起来难以发现。如果不使用，则会抛出 `MatchError`。但如果使用封闭类（sealed class），Scala 就能够判断出这个类的所有情况都已经被覆盖了。

```scala
sealed abstract class A
class B extends A
class C extends A

def f(a: A) = a match { case a: B => true }
// warning: match may not be exhaustive.
// It would fail on the following input: C()
//  def f(a: A) = a match { case a: B => true }
//                ^

def f(a: A) = a match { case a: B => true; case a: C => false }  // no warning
```

当然，也可以选择使用 `@unchecked` 注解，但这样做通常并不合适。

```scala
def f(a: A) = (a: @unchecked) match { case a: B => true }
```

Sealed class 的最典型例子就是 `Option`。这个类只有两个子类，`Some` 和 `None`。Scala 的 `Map` 就使用了这个类：

```scala
val m = Map("a" -> 1, "b" -> 2)
m("c")
// java.util.NoSuchElementException: key not found: c
//   at scala.collection.immutable.Map$Map2.apply(Map.scala:135)
//   ... 28 elided

m.get("c")  // None
m.get("a")  // Some(1)
```

处理 `Option` 的常用方法也包括 `map` `flatMap` 和模式匹配。

## 模式的更多应用

```scala
val myTuple = (123, "abc")
val (number, string) = myTuple
// number: Int = 123
// string: String = abc
```

类似地，所有的 case class 都可以用类似的方法来解析。

花括号内的部分实际上就是一个函数字面量，或者说一个 lambda 表达式。

```scala
val withDefault: Option[Int] => Int = {
  case Some(x) => x
  case None => 0
}
// withDefault: Option[Int] => Int = <function1>
```

我们把 `withDefault` 定义为了一个接收 `Option[Int]`，返回 `Int` 的 lambda 表达式，这种语法在 Akka 中非常常用。

如果一个模式匹配的最后没有 `case _` 或 `case v`，那么当遇到未覆盖的值时会抛出 `MatchError`，这样的模式匹配属于一个**偏函数**。就是说，它不能完全处理整个定义域。

```scala
val second: Function1[List[Int], Int] = { case x :: y :: _ => y }
// warning: match may not be exhaustive.
// It would fail on the following inputs: List(_), Nil
//  val second: Function1[List[Int], Int] = { case x :: y :: _ => y }
//                                          ^

val second: PartialFunction[List[Int], Int] = { case x :: y :: _ => y }
List(Nil, List(1, 2, 3)) map second.isDefinedAt  // List(false, true)
```

在将一个 lambda 表达式隐式转换成一个 `PartialFunction` 对象时，这个类被这样定义：

```scala
new PartialFunction[List[Int], Int] {
  def apply(xs: List[Int]) = xs match { case x :: y :: _ => y}
  def isDefinedAt(xs: List[Int]) = xs match {
    case x :: y :: _ => true
    case _ => false
  }
}
```

相对于普通的 `Function` 类，`PartialFunction` 额外定义了一系列可以用于量多个偏函数连接起来的方法。如果不能匹配，两种函数都会抛出 `MatchError`。

for 表达式里也会出现模式匹配。通常 for 表达式都能够完美匹配，因为容器中只能保存一种对象。一个例外是 `Option`，这种情况下 `None` 会被抛弃。

```scala
for (Some(a) <- List(Some(1), None, Some(3))) print(a)  // 13
for ((a, b) <- List((1, 2), (3, 4))) println(a + " " + b)  // 1 2\n3 4
```
