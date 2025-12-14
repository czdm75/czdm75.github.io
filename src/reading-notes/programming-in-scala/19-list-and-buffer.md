# 19. List 与 ListBuffer

Scala 的 `List` 是一个 `sealed abstract class`，有两个子类：`::` 和 `Nil`。其中 `Nil` 是一个 `case object`。所以，不能直接使用 `new List()`，只能通过 `List.apply()` 来调用 `::` 类。由于 `Nil extends List[Nothing]` 且 `List` 是协变的，它能兼容任何列表类型。

`::`（cons, construct）类表示有元素的列表。它的构造接收两个参数，即列表的 `head` 元素和 `tail` 列表。然后，`List` 类中定义了这样的方法。要记得以冒号结尾的操作符是右结合的，所以：

```scala
def :: [B >: A](elem: B): List[B] =  new ::(elem, this)
```

使得 `1 :: 2 :: Nil` 这样的构造方式得以实现。

现在我们再来考虑类型问题。从结果上来说，一些不同类型的对象进行 `::` 操作，最终得到的结果应当是一个以其公共父类为类型参数的列表。这里通过上面这个方法的类型参数得以实现。当：

```scala
apple :: List(orange)
```

这里的 `::` 方法在 `List(orange)` 上被调用，那么上面的类型参数 `A` 是 `Orange`，`B` 则 应该是 `A` 的一个父类型。又因为接收的参数也是 `B` 类型，所以最终 `B` 被决定为 `Fruit` 类型。

接下来我们讨论 `ListBuffer`。考虑一个简单的 `map` 函数，如果直接用列表递归的方式来实现，由于 `::` 是右结合的，我们会得到一个糟糕的没有尾递归的函数：

```scala
def map[B](f: A => B): List[B] = xs match {
  case Nil => Nil
  case x :: xs1 => f(x) :: xs1.map(f)
}
```

面对这种情况，可以使用 `ListBuffer`，通过 `+=` 来向尾部追加这些结果。`ListBuffer` 中 `addOne` 的实现是这样的：（Scala 中 `List` 的 `map` 没有直接使用 `ListBuffer`，但实质是一样的）：

```scala
def addOne(elem: A): this.type = {
  val last1 = new ::[A](elem, Nil)
  if (len == 0) first = last1 else last0.next = last1
  last0 = last1
  len += 1
  this
}
```

可以看到，其基本想法是，直接把列表末尾的那一个 `::` 对象的 `next` 从原来的 `Nil` 修改为新的最后一个 `::` 对象。这样，前后追加和转换成 `List` 的操作都是 O(1) 的。当然，如果在输出为列表之后还要进行追加，仍然要进行复制，不过这种情况很少见。`ListBuffer`。这样做之所以可行，是因为 `::` 类的定义中，`next` 变量是一个 `private[scala] var`。这样，`scala` 包中的集合可以直接调用它，但对于用户来说，只有只读的 `tail` 可以访问，`next` 访问不到。于是，`List` 类仍然是 Immutable 的。

```scala
final case class :: [+A](override val head: A, private[scala] var next: List[A @uncheckedVariance]) extends List[A] {
  override def isEmpty: Boolean = false
  override def headOption: Some[A] = Some(head)
  override def tail: List[A] = next
}
```

总之，最终我们实现了一个 Immutable 的列表。让其对外部成为不变类的原因是，我们得以让许多列表共享中间的 `::` 链表节点结构，还分别提供了 prepend 和 append 操作。通常，`List` 的 `::` 更适合函数式、递归的分治，`ListBuffer` 的 `+=` 更适合传统的命令式编程范式。
