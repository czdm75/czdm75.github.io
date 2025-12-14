# 21. Scala 集合系统

## 集合 Trait 提供的方法

```txt
Traversable
  Iterable
    Seq
      IndexedSeq
        Vector
        ResizableArray
        GenericArray
      LinearSeq
        MutableList
        List
        Stream
      Buffer
        ListBuffer
        ArrayBuffer
    Set
      SortedSet
        TreeSet
      HashSet(immutable and mutable)
      LinkedHashSet
      BitSet
      EmptySet, Set1, Set2, Set3, Set4
    Map
      SortedMap
        TreeMap
      HashMap(immutable and mutable)
      LinkedHashMap
      EmptyMap, Map1, Map2, Map3, Map4
```

`Traversable` 定义了我们用到的绝大部分方法：判断尺寸，折叠，`foreach`，`map`，`mkString`，`++`，`groupBy`，集合间的转换等等。`Iterable` 有两个重要的方法返回 `Iterator` ：`grouped` 和 `sliding`，分别提供元素分组和滑动窗口。`zip` 也由这个特质提供。

下面的三个特质 `Seq` `Set` 和 `Map` 都继承了 `PartialFunction`，拥有 `definedAt` 方法，目的是让我们以 `seq(index)` 的方式进行下标访问，以 `set(elem)` 的方式测试是否存在，以及以 `map(key)` 的方式取得值。

`Seq` 提供的有用方法包括 `union` `diff` `intersect` `distinct` `sorted` `sortBy` `sortWith` `reverse` `update` 等。其下的两个特质，`LinearSeq` 和 `IndexedSeq` 分别标记着擅长 prepend、append 操作的链表实现和擅长随机访问的数组实现。

`Map` 的 `get` 方法返回的是一个 `Option[V]` 类型的对象，因此是安全的。

**集合之间的相等性判断是基于元素的，而无视类型。因此，可变集合不应当被用于 HashMap 的 Key。**

## 创建集合

除了熟悉的 `apply` `empty` 之外，几乎所有的集合类型的伴生对象还提供了一系列创建集合的其他方法，包括 `concat` `fill` `tabulate` `range` `iterate` 等，由于接收的参数都是表达式而非直接的变量，提供了一些高阶的使用方式。

```scala
List.concat(Array(1, 2), Traversable(3, 4))
// List[Int] = List(1, 2, 3, 4)

import scala.util.Random
List.fill(3)(1)
// List(1, 1, 1)
List.fill(2, 3)(1)
// List(List(1, 1, 1), List(1, 1, 1))
List.fill(2, 3)(Random.nextInt(10))
// List(List(1, 7, 4), List(1, 5, 3))

List.tabulate(5)(_ + 1)
// List(1, 2, 3, 4, 5)
List.tabulate(3, 3)(_ * 10 + _)
// List(List(0, 1, 2), List(10, 11, 12), List(20, 21, 22))

List.range(10, 0, -1)
// List(10, 9, 8, 7, 6, 5, 4, 3, 2, 1)

List.iterate(5, 5)(_ + 1)
// List(5, 6, 7, 8, 9)
List.iterate(2, 5)(_ * 2)
// List(2, 4, 8, 16, 32)
```

`collection.JavaConversions` 中提供了 `Iterable` `Iterator` `Buffer` `Map` `Set` 转换到对应的 Java 对象的隐式转换。因为 Scala 集合的底层与 Java 集合兼容，所以转换的代价是 O(1) 的。甚至，两次转换后得到的其实还是原来的集合。

## 具体的不可变集合类

### Stream

`Stream` 与列表类似，使用和列表相似的 `#::` 和 `Stream.empty` 来构建，但流是惰性的。在没有访问的情况下，其整个 `tail` 都是尚未求值的。因此可以构建一个无限递归的无限流：

```scala
def fibForm(a: Int, b: Int): Stream[Int] = a #:: fibForm(b, a + b)
val fibList = fibForm(1, 1).take(7).toList  // List(1, 1, 2, 3, 5, 8, 13)
```

### Vector

`Vector` 是一个适合复杂访问的序列类型。其内部是一个宽而浅、每个节点能装下 32 个元素的树结构。虽然理论上其访问的时间是对数级的，但实际上五层的树结构已经足以装下 2\^30 个元素了。因此，访问的时间可以认为事实上是常量级别时间的。基于同样的原因。当在向量中改变一个元素时，只需要复制从根部到这个节点的所有节点，也就是不超过五个，其需要复制的量级也是"事实上的常量级别"。

### Map & Set

Scala 中的 `HashSet` `HashMap` 实现和 Vector 类似，也是通过这种形式的一棵树，只不过改用了哈希前缀树的形式。`TreeSet` 和 `TreeMap` 则是使用红黑树来实现。

`BitSet` 内部使用 `Long` 来二进制地存储整数。如果需要一系列几百量级以内的整数集合，使用这个数据结构的效率非常高，因为一个 `Long` 就能保存 64 个整数位置。

最后，`ListMap` 以键值对列表的形式存储。仅当第一个元素被经常访问时，其效率才比较高，并不常用。

## 具体的可变集合类

`DoubleLinkedList` 是双向链表，在使用迭代器迭代时删除元素的效率是 O(1) 的（单向链表删除的效率是 O(n) 的）。

`MutableList` 是 `mutable.LinearSeq` 的默认实现，包括一个单向链表和一个指向最后一个节点的引用。这样，在向列表追加时效率从 O(n) 提升到了 O(1)。

可变版本的 `Queue` 用 `+=` 替换了 `enqueue`，`dequeue` 也只是直接返回（因为队列本身发生了变化）。

## 数组

### 使用数组

Scala 中的 `Array` 是 Java 数组的简单包装，但这个类兼容于 `Seq` 引用，也支持 `Seq` 的操作。不过，这两件事并不是以同样的方式实现的。如果用一个 `Seq` 类型的引用接收数组，数组会被隐式转换为 `WrappedArray`。在这个类上调用方法，得到的仍然是 `WrappedArray`。但如果直接在数组上进行调用，会被隐式转换为 `ArrayOps`，得到的结果仍然是数组，`ArrayOps` 可以被回收，现代虚拟机甚至能把这个过程内联掉。

```scala
val seq: Seq[Int] = a1
// Seq[Int] = WrappedArray(1, 2, 3)
seq.reverse
// Seq[Int] = WrappedArray(3, 2, 1)

val ops: collection.mutable.ArrayOps[Int] = a1
// scala.collection.mutable.ArrayOps[Int] = [I(1, 2, 3)
ops.reverse
// Array[Int] = Array(3, 2, 1)
```

这种方式的实现方法是，转换为 `ArrayOps` 的隐式转换定义在 `Predef`，转换为 `WrappedArray` 的转换被定义在 `scala.LowPriorityImplicits` 中，这个类是 `Predef` 的超类，所以优先级更低。

### 泛型数组

在 Java 中，由于历史原因，数组不是泛型的，所以有：

```java
public <T> void fun() { System.out.println(new T[0]); }
// 错误:
// 创建泛型数组
// public <T> void fun() { System.out.println(new T[0]); }
//                                            ^------^
```

由于 Scala 的数组是直接用 Java 数组表示的，这个问题同样存在。

```scala
def fun[T](t: T) = Array(t)
// error: No ClassTag available for T
//   def fun[T](t: T) = Array(t)
//                           ^
```

这时我们需要为数组提供一个 `ClassTag`，作用类似于泛型的类型参数，用来表示被擦除的类型。

```scala
def fun[T: ClassTag](t: T) = Array(t)  // [T](t: T)(implicit evidence$1: scala.reflect.ClassTag[T])Array[T]
```

大多数情况下，Scala 编译器能够自动推断出 `ClassTag` 的值。从上面的 shell 提示可以看到，这个写法的结果实际上是为 `fun` 增加了一个名为 `evidence$1` 的隐式参数而已。这个参数的类型是 `ClassTag`，位于反射包中，编译器在构建数组是会使用它。

当然，Scala 编译器的推断能力也不是无限的。如果这个类型本身是另外一个类型参数，我们就无法在这里进行推断，需要把外围的函数也加上 `ClassTag`。

```scala
def f[U](x: U) = fun(x)
// error: No ClassTag available for U
//   def f[U](x: U) = fun(x)
//                       ^
def f[U: ClassTag](x: U) = fun(x)  // [U](x: U)(implicit evidence$1: scala.reflect.ClassTag[U])Array[U]
```

## 集合性能总结

|               | head     | tail     | apply    | update   | prepend  | append   | insert |
| ------------- | -------- | -------- | -------- | -------- | -------- | -------- | ------ |
| **immutable** |          |          |          |          |          |          |        |
| List          | O(1)     | O(1)     | O(n)     | O(n)     | O(1)     | O(n)     | -      |
| Stream        | O(1)     | O(1)     | O(n)     | O(n)     | O(1)     | O(n)     | -      |
| Vector        | 事实O(1) | 事实O(1) | 事实O(1) | 事实O(1) | 事实O(1) | 事实O(1) | -      |
| Stack         | O(1)     | O(1)     | O(n)     | O(n)     | O(1)     | O(n)     | -      |
| Queue         | 摊还O(1) | 摊还O(1) | O(n)     | O(n)     | O(n)     | O(1)     | -      |
| Range         | O(1)     | O(1)     | O(1)     | -        | -        | -        | -      |
| String        | O(1)     | O(n)     | O(1)     | O(n)     | O(n)     | O(n)     | -      |
| **mutable**   |          |          |          |          |          |          |        |
| ArrayBuffer   | O(1)     | O(n)     | O(1)     | O(1)     | O(n)     | 摊还O(1) | O(n)   |
| ListBuffer    | O(1)     | O(n)     | O(n)     | O(n)     | O(1)     | O(1)     | O(n)   |
| StringBuilder | O(1)     | O(n)     | O(1)     | O(1)     | O(n)     | 摊还O(1) | O(n)   |
| MutableList   | O(1)     | O(n)     | O(n)     | O(n)     | O(1)     | O(1)     | O(n)   |
| Queue         | O(1)     | O(n)     | O(n)     | O(n)     | O(1)     | O(1)     | O(n)   |
| ArraySeq      | O(1)     | O(n)     | O(1)     | O(1)     | -        | -        | -      |
| Stack         | O(1)     | O(n)     | O(n)     | O(n)     | O(1)     | O(n)     | O(n)   |
| ArrayStack    | O(1)     | O(n)     | O(1)     | O(1)     | 摊还O(1) | O(n)     | O(n)   |
| Array         | O(1)     | O(n)     | O(1)     | O(1)     | -        | -        | -      |

---------

|                 | lookup   | add      | remove   | min      |
| --------------- | -------- | -------- | -------- | -------- |
| **immutable**   |          |          |          |          |
| HashSet/HashMap | 事实O(1) | 事实O(1) | 事实O(1) | O(n)     |
| TreeSet/TreeMap | log(n)   | log(n)   | log(n)   | log(n)   |
| BitSet          | O(1)     | O(n)     | O(n)     | 事实O(1) |
| ListMap         | O(n)     | O(n)     | O(n)     | O(n)     |
| **mutable**     |          |          |          |          |
| HashSet/HashMap | 事实O(1) | 事实O(1) | 事实O(1) | O(n)     |
| WeakHashMap     | 事实O(1) | 事实O(1) | 事实O(1) | O(n)     |
| BitSet          | O(1)     | 摊还O(1) | O(1)     | 事实O(1) |

## 视图 View

视图是实现惰性求值集合的方法。如果不使用视图，可以这样实现一个惰性的集合：

```scala
def lazyMap[T, U](l: List[T], f: Int => U) =
  new Iterable[U] {
    def iterator = l.iterator map f
  }
```

有了 `View`，我们就可以交给 Scala 来实现一个惰性求值的，元素完全一样的集合：

```scala
List(1, 2, 3).view.map(_ + 1).map(_ * 2).force
// Seq[Int] = List(4, 6, 8)
List(1, 2, 3).view.map(_ + 1).map(_ * 2).map(_ - 1).filter(_ > 5).slice(0, 2).reverse
// scala.collection.SeqView[Int,Seq[_]] = SeqViewMMMFSR(...)
```

从得到的中间结果的类型可以看到，对于每一次操作，`SeqView` 的类型后面会多一个字母，表示封装了一定的操作。而当我们用 `force` 取得结果时，运算的中间结果并不会被创建，这非常重要。考虑这样一个情形：

```scala
aLargeCollection.take(100000).find(condition)
aLargeCollection.view.take(100000).find(condition)
```

由于 `View` 的惰性求值，如果能够在集合的前半段就找到目标，那么长达 100000 的中间结果的大部分将完全不存在于内存中。

另一个有意义的用法是，当我们想要修改可变集合中的一个窗口，也可以使用 `View`。这样，修改和切片两个操作就被很好地解耦。

```scala
val a = ArrayBuffer(1, 2, 3, 4, 5, 6)
val part = a.view.slice(1, 4)
for (i <- 0 until part.length) part(i) += 1
a
// scala.collection.mutable.ArrayBuffer[Int] = ArrayBuffer(1, 3, 4, 5, 5, 6)
```

不过这并不是一个非常好的例子，因为创建闭包和视图所消耗的 CPU 和内存几乎一定大于在这么小的集合上进行操作的消耗。如果代码是有副作用的，那么在惰性求值的情况下，事情会变得更加复杂。

## 迭代器 Iterator

`Iterator` 提供了大部分 `Seq` `Traversable` `Iterable` 中提供的方法，但行为不太一样。例如，迭代器的 `map` 返回另一个迭代器，并且只包含迭代器后面的元素的结果，`foreach` 同理。而且 `map` 和 `foreach` 都会让迭代器到达集合的末尾，继续调用 `next` 则会抛出 `NoSuchElementException`。对于 `dropWhile`，迭代器会在找到第一个不符合条件的元素后停下。唯一能改变这种情况的标准方法是使用 `duplicate`:

```scala
val it = Iterator(1, 2, 3, 4, 5)
val t = it.duplicate
t._1.next  // 1
t._1.next  // 2
t._1.next  // 3
it.next    // 4
t._2.next  // 1
```

这样得到的两个迭代器之间是独立的，但原来的 `it` 则会跟随两个迭代器中更快的一个。显然这种行为很难控制，所以原则上我们可以认为原来的 `it` 不再可用。

针对这样的特点，Scala 类库在 `Traversable` 和 `Iterator` 之上抽象出了 `TraversableOnce`，表示可以一次性地遍历访问，但访问后集合的状态不作保证。

不过迭代器这样的特点也有一点不方便，例如我们实现一个类似 `skipWhile` 的函数：

```scala
def skipWhile[T](it: Iterator[T])(pred: T => Boolean) = while (pred(it.next))
```

这样的话，第一个不符合条件的元素被识别出来的时候，我们就已经失去它而指向下一个了。这时需要使用的是 `BufferedIterator` 这个 Trait 的实例，可以使用 `head` 方法来查看第一个元素而不会跳过它。

```scala
def skipWhile[T](ite: BufferedIterator[T])(pred: T => Boolean) = while(pred(it.head)) it.next()
skipWhile(it.buffer)(pred)
```
