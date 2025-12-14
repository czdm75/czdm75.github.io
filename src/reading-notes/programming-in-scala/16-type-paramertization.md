# 16. 类型参数化

## 一个摊还 O(1) 复杂度的函数式队列

函数式数据结构通常期望使用递归来进行操作并避免状态的暴露，这让编程模型更优雅统一，但同时，与随机访问的数据结构相比，会将复杂度从 O(1) 提高到 O(n)。不过，通过一系列精妙的设计，函数式的数据结构同样可以具有高性能。虽然这方面的研究尚不完善，但 Scala 混合编程范式的特点让我们能够比较容易地做到这一点。

首先我们简单地使用列表来实现一个队列。由于列表是前追加的数据结构，我们的第一反应是使用一个翻过来的列表：

```scala
class SlowHeadQueue[T](elems: List[T]) {
  def head = elems.last
  def tail = new SlowHeadQueue(elems.init)
  def enqueue(x: T) = new SlowHeadQueue(x :: elems)
}
```

这个数据结构 `enqueue` 是 O(1) 的，而 `head` 和 `tail` 是 O(n) 的。但是，我们可以考虑将 `head` 操作和 `tail` 操作分开，即使用两个背对背的列表来处理。

```scala
class Queue[T](private val leading: List[T], private val trailing: List[T]) {
  private def mirror = if (leading.isEmpty) new Queue(trailing.reverse, Nil) else this
  def head = mirror.leading.head
  def tail = {
    val q = mirror
    new Queue(q.leading.tail, q.trailing)
  }
  def enqueue(x: T) = new Queue(leading, x :: trailing)
}
```

现在，仅当 `leading` 为空时，才会发生 O(n) 的操作。由于要使得 `leading` 为空需要 O(n) 次的 `tail` 操作，所以这个数据结构的摊还成本是 O(1) 的。

现在，剩下的问题是，构建这个队列看起来非常奇怪，需要传入两个队列。所以，我们需要另一个构造函数。实现方式有几种：

```scala
class Queue[T] private(private val leading: List[T], private val trailing: List[T]) {
  def this() = this(Nil, Nil)
  def this(elems: T*) = this(elems.toList, Null)
}
// you can only call this() from the outside now
Queue
Queue(1, 2 ,3)
Queue(List(1, 2, 3): _*)
```

更好的办法是使用伴生对象：

```scala
object Queue {
  def apply[T](xs: T*) = new Queue[T](xs.toList, Nil)
}
```

实际上，在 Scala 中，既然已经有了 `apply` 方法，我们就完全不再有必要把带有具体实现的 `Queue` 类暴露出来。所以，通常我们会这样做：

```scala
trait Queue[T] {
  def head: T
  def tail: Queue[T]
  def enqueue(x: T): Queue[T]
}
object Queue {
  def apply[T](xs: T*): Queue[T] = new QueueImpl[T](xs.toList, Nil)
  private class QueueImpl[T](
    private val leading: List[T],
    private val trailing: List[T]
  ) extends Queue[T] { ... }
}
```

## 泛型变型

### 变型

Scala 中的泛型默认是不变的。`[+T]` 表示协变，`[-T]` 表示逆变。Scala 编译器会自动检查代码中类型参数被使用时的正确性。简单地理解，生产者是协变的，消费者是逆变的。典型的例子是 Scala 中的函数类型：

```scala
trait Function1[-T1, +R] extends AnyRef { ... }
```

```scala
class A[+T] { def get: T = ??? }
class A[-T] { def set(x: T) = ??? }

class A[-T] { def get: T = ??? }
// error: contravariant type T occurs in covariant position in type => T of method get
//  class A[-T] { def get: T = ??? }
//                   ^
class A[+T] { def set(x: T) = ??? }
// error: covariant type T occurs in contravariant position in type T of value x
//  class A[+T] { def set(x: T) = ??? }
//                       ^
```

Java 的数组默认是协变的，这是因为 Java 1.5 之前没有泛型时的历史原因，Scala 则默认不变型。当我们修改 Java 数组时，可能会得到 `ArrayStoreException`。而在 Scala 中，不能像 Java 一样直接赋值，只可能像 Java 的默认泛型一样 cast 数组（`arr.asInstanceOf[Array[Object]]`），然后才有可能发生 `ArrayStoreException`。也就是说，当能发生这个问题的情况时，你应该已经意识到了这个风险。

### 下界和上界

对我们上面的队列的例子，如果队列定义为协变的，`enqueue` 方法就会产生矛盾，因为 setter 是逆变点（消费者）。例如，如果这里没有限制，就可能会有：

```scala
val q: Queue[Fruit] = new Queue[Apple]
q.enqueue(new Orange)
```

这显然是不合理的。好在我们可以利用下界（相当于 Java 中的类型参数通配符）来限制 `enqueue` 参数的类型的范围：

```scala
class Queue[+T] (private val leading: List[T], private val trailing: List[T]) {
  def enqueue[U >: T](x: U) = new Queue(leading, x :: trailing)
}
```

现在，`enqueue` 的参数被限制在 `T` 的父类，其返回值也变成了 `Queue[U]` 而不是 `Queue[T]`。于是有：

```scala
val q: Queue[Fruit] = new Queue[Apple]
q.enqueue(new Orange)  // won't compile
q.enqueue(new Fruit)   // q.enqueue now only takes U >: Fruit, no Orange
```

有意思的是，这样的方式仅适用于符合函数式范式的 Immutable 的数据结构。而且，我们这里确定了正确的类型的同时，代码的逻辑也被确定了。也许我们一开始并没有想到这样安全地描述 `enqueue` 方法，但由于编译器的协变检查，我们必须要这样实现。这种方式也被叫做**类型驱动设计**（type-driven design）。在一些学术性质甚至比 Scala 更强的语言，如 Haskell 中，这件事体现得更加明显。

这也解释了为什么 Scala 采用了"声明点型变"而不是 Java 的"使用点型变"。作为类的定义者，我们可以在这里解决变型问题而不是将这些复杂的问题交给使用者。实际上，许多使用基于 Scala 的成熟框架的开发者，比如 Spark 的用户，完全不需要知道这些有关变型的知识。

另外，对象内私有的变量不需要进行这些限制，因为它并不会被对象外访问到。例如，上面的队列在 `leading` 为空时连续 `head` 的性能较差，因为每一次调用都要重新进行 `trailing.reverse`。可以在对象内部引入状态来解决：

```scala
class Queue[+T] private ( private[this] var leading: List[T], private[this] var trailing: List[T]) {
  private def mirror() = if (leading.isEmpty) {
    while (!trailing.isEmpty) {
      leading = trailing.head :: leading
      trailing = trailing.tail
    }
  }
  def head: T = { mirror(); leading.head }
  def tail: Queue[T] = { mirror(); new Queue(leading.tail, trailing) }
  def enqueue[U >: T](x: U) = new Queue[U](leading, x :: trailing)
}
```

这里的 `mirror` 使用了指令式编程的方式，目的是体现出，虽然这个过程中出现了关于类型参数 `T` 的 get 和 set 参数（修改 `leading` 和 `trailing` 的值），但我们完全不需要考虑变型问题，因为对象内部的 `T` 是已经确定的。所以，`private[this]` 的变量不会进行编译期的变型检查，只会进行基本的类型检查。

类似地，对于逆变的情况，也可以有上界。例如，对于一个排序函数，需要有：

```scala
def mergeSort[T <: Ordered[T]](xs: List[T]): List[T] = { ... }
```

这样，我们要求传入的列表中的对象必须是 `Ordered[T]` 的子类型。不过，这并不是使用 `Ordered` 特质的最佳方式。
