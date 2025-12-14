# 8. Scala 的层级

## Scala 继承层级

`Any` 类定义了以下的方法：

```scala
final def ==(that: Any): Boolean
final def !=(that: Any): Boolean
def equals(that: Any): Boolean
def hashCode: Int
def toString: String
```

其中 `==` `!=` 方法是 `final` 的，它们的取值取决于 `equals` 方法。因此，Scala 中可以使用 `==` 来比较 `Integer` `String` 和其他对象。

`Any` 有两个子类：`AnyVal` `AnyRef`。其中 `AnyVal` 有九个子类，包括 Java 的八种基本类型和 `Unit`。这些类都不能用 `new` 来创建，而必须使用字面量。实际上，这些类都是 `abstract final` 的，所以无法使用 `new`。`Unit` 则只有一个值，写作 `()`。

![Scala类继承层级](../class-hierarchy.png)

`AnyVal` 的子类被称为值类型，它们之间可以隐式地互相转换。之前提到过，它们还可以隐式地转换为对应的 `Rich` 类以支持 `until` `range` `max` 等更多操作。值类型在编译之后将会变成基本类型而不是他们对应的装箱类型，这样做能够带来一些性能提升。Scala 在这里做的事情和 Java 5 的自动装箱很相似。另外一个类 `AnyRef` 实际上就是 `java.lang.Object`。

在整个继承树的底端是 `scala.Nothing` 和 `scala.Null`。`Null` 类有一个实例，即 `null`，也就是空引用，而 `Nothing` 则没有值。这样做的目的是为类型推导提供方便。二者的主要区别是，`Null` 仅包括了 `AnyRef` 的子类（某种意义上的引用类型），`Nothing` 包括值类型。以及，`Null` 有一个实例。

```scala
{ x: Int => if (x == 0) "zero" else throw new Exception }  // Int => String
{ x: Int => if (x == 0) "zero" else null }                 // Int => String
{ x: Int => if (x == 0) "zero" else 1 }                    // Int => Any
```

## 自定义值类型

```scala
class Dollars(val amount: Int) extends AnyVal {
  override def toString() = "$" + amount
}
```

值类型可以让代码更加清晰，减少错误。考虑这样一段关于 `HTML` 的代码：

```scala
def title(text: String, anchor: String, style: String): String =
  s"<a id='$anchor'><h1 class='$style'>$text</h1></a>"
```

四个参数都是 `String`，有人称这种代码为 Stringly Typed。因为字符串之间没有区别，这段代码实际上和弱类型语言并没有本质区别，编译器也不能帮我们检查错误。如果使用短小的值类型，就能解决这种问题：

```scala
class Anchor(val value: String) extends AnyVal
class Style(val value: String) extends AnyVal
class Text(val value: String) extends AnyVal
class Html(val value: String) extends AnyVal

def title(text: Text, anchor: Anchor, style: Style): Html = new Html(
  s"<a id='${anchor.value}'><h1 class='${style.value}'>text.value</h1></a>"
)
```

## 相等性

因为 Scala 将 `==` 和 `equals` 统一起来，所以 `AnyRef` 定义了 `eq` 方法用于两个引用的直接比较。类似地，还有一个相反的方法名为 `ne`。有关相等性在 30 章还会有更多讨论。

```java
"abc" == new String("abc")       // false
"abc".equals(new String("abc"))  // true
```

```scala
new String("abc") == new String("abc")  // true
new String("abc") eq new String("abc")  // false
// warning: comparing a fresh object using `eq' will always yield false
//   new String("abc") eq new String("abc")
//                     ^
scala > "abc" eq "abc"  // true,  because of hash consing
```

这个原则的唯一例外是 Java 装箱类型。在 Java 中，能够互相转换的对象的 `equals` 结果仍然是 `false`。

```scala
java.lang.Integer.valueOf(1) equals java.lang.Long.valueOf(1)  // false
java.lang.Integer.valueOf(1) == java.lang.Long.valueOf(1)      // true
java.lang.Integer.valueOf(1) eq java.lang.Long.valueOf(1)      // false

java.lang.Integer.valueOf(1) equals new java.lang.Integer(1)   // true
java.lang.Integer.valueOf(1) == new java.lang.Integer(1)       // true
java.lang.Integer.valueOf(1) eq new java.lang.Integer(1)       // false
```

而对于 `Int` 来说，因为是值类型，不能进行 `eq` 比较。
