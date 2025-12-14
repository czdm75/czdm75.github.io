# 7. 组合与继承

在这一部分的例子中，我们将最终实现这样一个布局库：

```scala
val column1 = elem("hello") above elem("***")
val column2 = elem("***") above elem("world")
column1 beside column2
// hello ***
//  *** world
```

## 抽象类

```scala
abstract class Element {
  def contents: Array[String]
  def height: Int = contents.length
  def width: Int = if (height == 0) 0 else contents(0).length
}
```

首先我们定义了一个元素的抽象类。这里的三个方法都没有参数括号：这样，调用方也必须不加括号才能访问。

Scala 世界的通常做法是，对于那些不改变对象本身的方法调用，不加括号，使其看起来更像是一个成员变量。这样做的目的是，使得这个方法看起来很像一个成员变量。我们在解释 Java 为什么要使用 getter 时举了许多次的例子：

```java
class A { public int length; }         // version 1.0
class A { public int length( ... ); }  // version 2.0
```

这个问题在这里直接得到了解决，因为一个名为 `length` 的变量和一个名为 `length` 的方法对调用方来说没有任何区别：

```scala
class A { var length }             // version 1.0
class A { def length: Int = ... }  // version 2.0
```

从代码风格来讲，虽然所有的空括号都可以被省略，对于那些**有副作用**的方法，例如执行 IO、修改变量，总之涉及到 mutable 对象的方法，最好还是加上括号。典型的例子如 `println()`。

将字段和方法统一对待的这种方式被称为 Scala 的统一访问原则（the Uniform Access Principle）。

## 扩展一个类

```scala
class ArrayElement(conts: Array[String]) extends Element {
  def contents: Array[String] = conts
}
```

由于上面我们提到的统一访问原则，你甚至可以用字段来 override 一个方法，这两种结构位于同一个命名空间内。出于同样的原因，这样的代码在 Java 里可行，在 Scala 里则不行：

```java
class A { int i = 0; public int i( return 0; ); }
```

```scala
class A { val i = 0; def i = 1 }  // won't compile
```

Java 有四个命名空间：字段，方法，类型和包。而 Scala 只有两个：值，包括字段，方法，包和单例对象，以及类型命名空间，包括类和特质（Trait）。

作为一个有经验的程序员，当我们意识到上面的变量名 `conts` 是在试图表达和 `contents` 一样的内容而在躲避变量名冲突时，就应该考虑是不是该重构一下这个片段了。这时适合使用参数化字段。而且，参数化字段和普通的字段一样可以使用 `override` `protected` `private` 来修饰：

```scala
class Cat { val dangerous = false }
class Tiger ( override val dangerous = true,
              private var age: Int
) extends Cat
```

需要调用父类的构造方法时：

```scala
class ArrayElement(val contents: Array[String]) extends Element

class LineElement(s: String) extends ArrayElement(Array(s)) {
  override def width = s.length
  override def height = 1
}
```

Scala 在重写方法时强制使用 `override` 修饰符。考虑这样的情况：你想在你的类库里添加一个新的方法，但用户在他们的代码里已经继承了这个类并重写了相同名字的方法，在 Java 中这种情况是相当麻烦的。这种我们不愿见到的重载被称为脆基类。不过如果你和下游调用者的代码都是使用 Scala 编写的，由于缺少 `override` 修饰符，下游代码编译时将会报错。虽然这样的解决方式仍然不怎么优雅，至少比 Java 的情况要好。

最后，和 Java 一样，可以给方法或类打上 `final` 修饰符以防止继承。

滥用继承是一个常见的问题，毕竟，脆基类的问题只会出现在继承而不会出现在组合中。在使用继承的时候，最好确认：首先，二者必须是一个 is-a 的关系。其次，考虑用户是否真的想将子类当做一个父类对象来使用。比如，上面的 `LineElement` 继承 `ArrayElement` 就比较奇怪。让它直接继承 `Element` 可能是更好的选择。

## 完善类库：实现方法，定义工厂

简单起见，我们先假设参数和被调用的 `Element` 宽或高相同。那么，方法可以这样实现：

```scala
abstract class Element {
  def above(that: Element): Element =
    new ArrayElement(this.contents ++ that.contents)
  def beside(that: Element): Element =
    new ArrayElement (
      for (
        (line1, line2) <- this.contents zip that.contents
      ) yield line1 + line2
    )
  override def toString = contents mkString "\n"
}
```

实现了方法之后，我们希望使用工厂来对调用者暴露接口，而不是直接把继承层级告诉用户。毕竟，上面而我们已经修改了一次 `LineElement` 的层级了。同时，我们的 `above` 这些方法也可以转而调用工厂方法：

```scala
object Element {
  def elem(contents: Array[String]): Element = new ArrayElement(contents)
  def elem(line: String): Element = new LineElement(line)
}


import Elements.elem
abstract class Element {
  def above(that: Element): Element = elem(this.contents ++ that.contents)
  def beside(that: Element): Element =
    elem(
      for ((line1, line2) <- this.contents zip that.contents
      ) yield line1 + line2
    )
}
```

此外，我们还发现 `ArrayElement` 本身也完全不需要暴露给用户了。所以，可以将它们全部转移成私有的：

```scala
object Element {
  private class ArrayElement(
    val contents: Array[String]) extends Element
  private class LineElement { ... }
  def elem(contents: Array[String]): Element = new ArrayElement(contents)
  def elem(line: String): Element = new LineElement(line)
}
```
