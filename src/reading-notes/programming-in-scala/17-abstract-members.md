# 17. 抽象成员

## 抽象成员的种类

一个包括各种抽象成员的例子：

```scala
trait Abstract {
  type T
  def transform(x: T): T
  val initial: T
  var current: T
}
class Concrete extends Abstract {
  type T = String
  def transform(x: String) = x + x
  val initial = "hi"
  var current = initial
}
```

这里的抽象成员包括类型成员、函数、可变变量和不可变变量。

首先，定义抽象类型的目的是形成一个别名（alias）。这样做的目的通常是隐藏一个复杂而含义不明显的类型，也可以使用 `<:` 或 `>:`。然后，在后面的类定义中就可以使用这个别名。

之前我们已经知道，`def` `val` `var` 对调用者来说并没有本质的区别。但当继承这个定义的时候，`val` 的下游只能是 `val`，所以每一次重复调用它，返回值都应该是相同的。`def` 则无法做出这样的保证。也就是说，可以用 `val` 来定义抽象的 `def`，但不能反过来。

## 抽象 val 的初始化时机

考虑之前出现过的有理数类：

```scala
trait RationalTrait { val numerArg: Int; val denomArg: Int }
class Rational (val numerArg: Int, val denomArg: Int) extends RationalTrait

new RationalTrait { val numerArg = 1; val denomArg = 2 }  // $anon$1@1c00d406
new Rational(1, 2)  // Rational@67cd84f9
```

这两种方式看起来似乎是一样的。但实际上，二者的参数初始化时间存在区别。使用类的情况下，`1` 和 `2` 是作为参数被传入，（在非传名参数的情况下）是先求值，再传入。而在使用 `new trait` 的情况下，则会先初始化 `RationalTrait`，再传入这两个值。如果我们加上一个能够检测这种情况的条件：

```scala
trait RationalTrait { val numerArg: Int; val denomArg: Int; require(numerArg > 0) }

new RationalTrait { val numerArg = 1; val denomArg = 2 }
// java.lang.IllegalArgumentException: requirement failed
//   at scala.Predef$.require(Predef.scala:268)
//   at RationalTrait.$init$(<console>:11)
//   ... 29 elided

new { val numerArg = 1; val denomArg = 2 } with RationalTrait  // $anon$1@2ba0b7cf
```

在进行 `require` 判断时，这两个变量的值还是默认值 0，所以抛出了异常。在 Trait 初始化完成之后，才会被赋上值。如果采用混入的方式，那么这两个变量会仙贝初始化，然后才会调用父类的构造方法，不会发生这个问题。

当然，一种更优雅的方式是将 `require` 语句放在 `lazy val` 的初始化中去，这在逻辑上也更合理。

```scala
trait LazyRationalTrait {
  val numerArg: Int
  val denomArg: Int
  lazy val = numerArg / g
  lazy val = denomArg / g
  private lazy val g = {
    require(denomArg != 0)
    gcd(numberArg, denomArg)
  }
  private def gcd(a: Int, b: Int): Int = if (b == 0) a else gcd(b, a % b)
}
```

显然，如果 `lazy val` 的初始化涉及到副作用，初始化时间的情况将会变得相当复杂。所以，这个特性和函数式数据结构结合得更加紧密。

## 抽象类型的作用

考虑这样的情况：

```scala
class A
class B extends A
abstract class C { def consume(x: A) }

class D extends C { override def consume(x: B) = print(x) }
// <console>:14: error: class D needs to be abstract, since method consume in class C of type (x: A)Unit is not defined
// (Note that A does not match B: class B is a subclass of class A, but method parameter types must match exactly.)
//        class D extends C { override def consume(x: B) = print(x) }
//              ^
// <console>:14: error: method consume overrides nothing.
// Note: the super classes of class D contain the following, non final members named consume:
// def consume(x: A): Unit
//        class D extends C { override def consume(x: B) = print(x) }
```

我们发现，`consume(x: B)` 无法重写 `consume(x: A)`，因为它们接收不同类型的参数。那么如果我们希望限制 `D` 类型中 `consume` 方法能够接收的参数类型呢？留下一个废弃的 `consume(x: A)` 显然不合适，允许重写更不合理。如果我们使用抽象类型：

```scala
abstract class C {
  type T <: A  // a type that is subclass of A
  def consume(x: T)
}
class D extends C {
  type T = B
  override def consume(x: B) = print(x)
}

(new D) consume (new B)  // $line46.$read$$iw$$iw$B@43687885

(new D) consume (new A)
// <console>:15: error: type mismatch;
//  found   : A
//  required: B
//        (new D) consume (new A)
//                         ^
```

这样，我们就限制了子类中参数的类型。

此外，这样定义的类型限制是**路径依赖**的。体现为，如果我们将一个 `D` 对象放在 `C` 引用中，那么抛出的 `type mismatch` 将会是：

```scala
val d: C = new D  // d: C = D@1eb3b8c0

d.consume(new A)
// <console>:15: error: type mismatch;
//  found   : A
//  required: d.T
//        d.consume(new A)
//                  ^
(new { val d: C = new D }).d.consume(new A)
// <console>:16: error: type mismatch;
//  found   : A
//  required: _1.T where val _1: C
//        (new { val d: C = new D }).d.consume(new A)
//                                             ^
```

可以看到，因为引用是 `C` 类型的，我们无法直接知道所需的 `T` 是哪一个实际类型，但是我们知道这个 `T` 是 `d` 对象中的，也就是说这个类型依赖于其（对象引用的）路径。具体来说，依赖于这些对象所属的类，有点类似于 Java 的内部类。不过，对于内部类的情况，Scala 使用 `#` 作为连接符。

## 改良类型 refinement type

抽象类型甚至让 Scala 具有了一定程度上类似于 duck type（没错）的能力。例如，在上面的例子中，如果我们想要一个类型，能够包括所有接收 `B` 类型的 `C`（例如，所有食草的动物），那么就有：

```scala
val cThatTakesBs: List[C {type T = B}] = ...
```

这样我们就无需为一系列" `T` 为 `B` 的对象"定义一个麻烦的容易忘记的新 trait 了，也能够更加肆无忌惮地使用 `new` `object` `with` 等语法来创建匿名类了。与 duck typing 相比，Scala 的这种能力需要我们在父类中提前做好设计（又一次，类型驱动设计），但相比于 Python 的按方法名判断，这种方式显然要安全得多。鸭子类型和改良类型都是实现"结构子类型"的一种方式，即由对象的结构决定其类型，而非传统的反过来的"名义子类型"，只有显式继承的才算是子类。

## 枚举 Enumeration

由于路径依赖类型的存在，Scala 不需要像很多语言一样让编译器额外处理枚举类型（再一次体现了 Scala 简单的基本语法延伸出复杂的用法和功能）。只需要继承一个类即可：

```scala
object Color extends Enumeration {
  val Red = value
  val Green = value
}
object Color extends Enumration {
  val Red, Green = Value
}
import Color._
```

`Enumeration` 类的核心部分大致上是：

```scala
abstract class Enumeration (initial: Int) extends Serializable {
  protected final def Value: Value = Value(nextId)
  protected final def Value(i: Int): Value = Value(i, nextNameOrNull)
  protected final def Value(name: String): Value = Value(nextId, name)
  protected final def Value(i: Int, name: String): Value = new Val(i, name)


  abstract class Value extends Ordered[Value] with Serializable {}
  protected class Val(i: Int, name: String) extends Value with Serializable {}
}
```

所以，不同的 `object extends Enumeration` 里的 `Value` 类，由于路径依赖不能兼容，使得不同枚举类的元素之间不能互相兼容。
