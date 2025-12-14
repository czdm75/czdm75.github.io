# 13. 列表

## 列表的形式

我们熟悉的列表语法：`List(1, 2, 3)` 是一个语法糖，它等价于 `1 :: 2 :: 3 :: Nil`，别忘了 `::` 是右结合的。`Nil` 的类型是 `List[Nothing]`。因为 `List` 是 Immutable 的，它也被实现为协变的。这样，由于 `Nothing` 是所有类型的子类，`Nil` 也是所有 `List` 的子类，因此 `Nil` 可以作为任何 `List` 的空列表表示。

我们已经知道在列表的前面增加元素是高效的，而在 `ArrayBuffer` 的后面增加元素是高效的。`ArrayBuffer` 能够向后扩容，而 `List` 的默认实现是一个链表。不严谨地说，`List(1, 2 ,3)` 这个对象的 `head` 属性是 1，而 `tail` 属性是 `List(2, 3)`，这样做比复制整个列表节省内存和时间。更重要的是，实现为链表有利于进行模式匹配。

或者从另一个角度来考虑，列表的所有操作都可以被归纳为三种：`head` `tail` `isEmpty`。对于 `List(1, 2, 3)(1)`，大体上相当于 `(1 :: 2 :: 3 :: Nil).tail.head`。Scala 列表的这些特性和 Haskell 非常相似。

## 列表与模式匹配

在模式匹配中，列表模式也可以使用 `::` 来表达，上面已经出现了这样的形式。不过利用 `::` 能够匹配得更加自由：

```scala
val a :: b :: c = List(1, 2, 3, 4)
// a: Int = 1
// b: Int = 2
// c: List[Int] = List(3, 4)

val a :: b :: c  = List(1, 2, 3)
// a: Int = 1
// b: Int = 2
// c: List[Int] = List(3)

val List(a, b, _) = List(1, 2, 3)
// a: Int = 1
// b: Int = 2

val List(a, b, _) = List(1, 2, 3, 4)
// scala.MatchError: List(1, 2, 3, 4) (of class scala.collection.immutable.$colon$colon)
//   ... 28 elided
```

相对于 `List(a, b, c)` 的形式，使用 `::` 能够正确地处理不同长度的列表。这两种方式适用于不同的情景。

有意思的是，这两种形式实际上都不符合我们之前对模式的定义。实际上，`List(a, b)` 是一个由开发库定义的 extractor 模式的实例（详细说明出现于书 24 章）。而另一种形式 `a :: b` 当出现在模式匹配中时，不再是调用的 `c.::` 方法，而是 `scala.::` 这个类，`a :: b` 等价于 `::(a, b)`，其中 `::` 是一个 Case Class。也就是说，这里的模式部分实际上是使用 `a` 和 `b` 作为两个参数生成的 `::` 对象模式。

```scala
final case class :: [+A](override val head: A, private[scala] var next: List[A @uncheckedVariance]) extends List[A] {
  override def isEmpty: Boolean = false
  override def headOption: Some[A] = Some(head)
  override def tail: List[A] = next
}
```

用模式匹配来实现关于列表的功能就是一个非常类似于 Haskell 的过程了。我们来尝试使用递归和模式匹配来实现 `:::` 的功能。为了避免和原有的方法冲突，我们将其命名为 `+++:`：

```scala
implicit final class AppendList[T](private val self: List[T]) extends AnyVal {
  def +++:(other: List[T]): List[T] = {
    other match {
      case Nil => self
      case head :: tail => head :: tail +++: self
    }
  }
}

List(1, 2) +++: List(3, 4)  // List(1, 2, 3, 4)
```

可以先不去考虑这里的隐式转换。模式匹配的逻辑并不复杂：如果是一个空列表，那么只需要返回原来的列表就可以了。如果是一个有内容的列表，那么就变成其 `head` 与一个递归的 append 列表的连接。这里的主要部分在于列表的递归思想。当然，实际的代码要比这样效率更高些。

由于列表的这种实现方式，取得元素的 `head` 和取得剩余列表的 `tail` 是 O(1) 的操作，而取得元素的 `last` 和取得前面一部分列表的 `init` 是 O(n) 的操作。

然后，我们来尝试实现一个归并排序：

```scala
def msort[T](less: (T, T) => Boolean)(xs: List[T]): List[T] = {
  def merge(xs: List[T], ys: List[T]): List[T] = {
    (xs, ys) match {
      case (Nil, _) => ys
      case (_, Nil) => xs
      case (x :: xsl, y :: ysl) =>
        if (less(x, y)) x :: merge(xsl, ys)
        else y :: merge(xs, ysl)
    }
  }
  val n = xs.length / 2
  if (n == 0) xs
  else {
    val (ys, zs) = xs splitAt n
    merge(msort(less)(ys), msort(less)(zs))
  }
}

msort((x: Int, y: Int) => x < y)(List(1, 2, 3))
val intSort = msort((x: Int, y: Int) => x < y) _
```

这里也能看到柯里化的手法，通过柯里化让一个泛型函数变成了一个固定参数类型的函数，然后接受下一个参数来执行。

## List 相关的高阶方法

高阶方法接受或返回另一个函数。如果你和我一样熟悉 Spark，或者熟悉 Python 的推导式，那么 `map` `filter` `flatMap` `foreach` 这些函数应该用起来很自然。和 `for` 一样，Scala 会自动产生与之前相似的类型。

```scala
List(1, 2, 3).map(_ + 1)         // List(2, 3, 4)
ArrayBuffer(1, 2, 3).map(_ + 1)  // ArrayBuffer(2, 3, 4)
Array(1, 2, 3).map(_ + 1)        // Array(2, 3, 4)
```

```scala
List("abc", "abcdge").indices  // scala.collection.immutable.Range = Range 0 until 2

val f = (l: List[Int]) => List(l.partition _, l.takeWhile _, l.dropWhile _, l.span _, l.forall _, l.exists _)
f(List(3, 4, 2, 1, 7, 5)) map {_{_ > 2}} foreach println
// (List(3, 4, 7, 5),List(2, 1))
// List(3, 4)
// List(2, 1, 7, 5)
// (List(3, 4),List(2, 1, 7, 5))
// false
// true
```

## 折叠

```scala
def sum(xs: List[Int]): Int = (0 /: xs) (_ + _)
```

这里使用了左折叠的操作。一个折叠操作与三个值有关：`(z /: xs) (op)`，即开始值、列表和操作符。如果要在开头排除操作符的副作用，例如：

```scala
val l = List("a", "b", "c")

("" /: l)(_ + " " + _)           // " a b c"
(l.head /: l.tail)(_ + " " + _)  // "a b c"
l.reduce(_ + " " + _)            // "a b c"
```

类似地，`:\` 操作符向右折叠，同时初始值和列表也要反过来。这样也遵循了之前定义右结合操作符时使用的 `:` 朝向被调用者的原则。也就是 `(List(a, b, c) :\ z)(op)`。当然，也可以使用 `foldLeft` 和 `foldRight`。此外，还可以使用 `reduceLeft` 和 `reduceRight`，它们不接收初始值，直接使用开头或结尾作为初始值。相应地，如果列表为空，它们会抛出异常。

考虑一个将 `List[List[T]]` 转换为 `List[T]` 的 `flatten` 操作。由于拼接列表这个操作满足结合律，有：

```scala
def flattenLeft[T](xss: List[List[T]]) = (List[T]() /: xss) (_ ::: _)
def flattenRight[T](xss: List[List[T]]) = (xss :\ List[T]()) (_ ::: _)
```

但这两种实现的性能有所不同。由于 `:::` 的时间代价与前者的长度成正比，所以 `flattenRight` 的性能要比 `flattenLeft` 好得多。类似地，可以实现一个基于折叠的线性复杂度的 `reverse` 方法：

```scala
def reverseLeft[T](xs: List[T]) = (List[T]() /: xs) {(ys, y) => y :: ys}
```

注意到在这两个函数中，都使用了 `List[T]()` 而非 `Nil` 来提供类型推断。

## List 对象的方法

```scala
Range                                            // scala.collection.immutable.Range
Range(1, 10, 2)                                  // scala.collection.immutable.Range
List.range(1, 10, 2)                             // List(1, 3, 5, 7, 9)
List.concat(List(1, 2), List(3, 4), List(5, 6))  // List(1, 2, 3, 4, 5, 6)
```

在目前的实现中，`:::` 是使用 `ListBuffer` 实现的，而 `List.concat` 继承自 `scala.collection.StrictOptimizedIterableOps`，是基于 `Iterable` 实现的。

最后，我们提供一个直接进行 zip 操作的方法，以下两种方式是等价的：

```scala
(List(1, 2, 3), List("a", "b", "c")).zipped.map{(i: Int, s: String) => i + " " + s}     // List(1 "a", 2 "b", 3 "c")
(List(1, 2, 3) zip List("a", "b", "c")).map{ case (i: Int, s: String) => i + " " + s}   // List(1 "a", 2 "b", 3 "c")
```

区别是，第一种的 `map` 接受的参数可以直接接收两个参数，而第二种中接收到的是一个 `Tuple2[Int, String]`。

## Scala 的类型推断

比较两个排序函数：

```scala
msort((x: Char, y: Char) => x > y)(list)
list sortWith (_ > _)
```

`_ > _` 这样的简单写法适用于后者但并不能适用于前者，这里就涉及到了 Scala 的类型推断。Scala 的类型推断是基于程序流的。`sortWith` 是 `list` 对象的方法，所以我们能够知道 `T` 的类型，而 `msort` 不能。回忆 `msort` 的定义：

```scala
def msort[T](less: (T, T) => Boolean)(xs: List[T]): List[T] = ???
```

当我们传入 `(x: Char, y: char) => x > y` 时，`T` 才被推断为 `Char` 类型。因此，可以这样调用：

```scala
msort[Char](_ > _)(list)
```

手动给 `T` 赋予值之后，就能够正常地进行推断了。另一种更好的方法是：

```scala
def msort[T](xs: List[T])(less: (T, T) => Boolean): List[T] = ???
```

这样，在第一个参数处，就可以直接得到类型参数，无需再手动指定了。不过，这样的结果是失去了柯里化的方便。因此我们得到了一个原则：在提供 API 时，尽量把数据结构放在前面，函数放在后面。

然后我们回头来看上面的 `flatten` 函数，函数里使用了 `xs :\ List[T]()` 而非 `Nil` 或等价的 `List()`。这是因为，如果使用 `List()`，那么折叠过程的第一步需要一个 `(List[T], List[Nothing]) => List[T]` 的操作符，而之后的部分则需要 `(List[T], List[T]) => List[T]` 类型，无法统一。
