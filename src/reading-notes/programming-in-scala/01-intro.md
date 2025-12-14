# 1. 入门

## apply 方法

对于代码：

```scala
val arr = Array("a", "b")
arr(0)
arr(0) = "c"
```

实际上是调用了：

```scala
val arr = Array.apply("a", "b")
arr.apply(0)
arr.update(0, "c")
```

## 列表

Scala 默认的 `List` 是 Immutable 的。可以对列表进行拼接：

```scala
val l = List(1, 2)
1 :: l    // List(1, 1, 2)
l ::: l   // List(1, 2, 1, 2)
l :: l    // List(List(1, 2), 1, 2)
```

首先，由于 List 是 Immutable 的，所以所有的拼接操作都返回一个新的 List。

三冒号的写法比较容易理解：它将两个列表连接起来。对于双冒号，则是将前面的元素与后面的列表连接起来。在第四行代码中，由于双冒号前面的元素被作为**一个**对象来操作，因此得到的是一个具有嵌套结构的 `Any` 列表。

而且，理所应当，双冒号应当是右结合的操作符，因为任何一个对象不应该持有这样与其没有太大直接关系的操作，所以 `::` 方法应该是列表的方法而不是所有对象都具有的方法。但是，通常调用的操作符是左结合的，如 `1 + 2` 实际为 `1.+(2)`。Scala 简单地使用冒号来区分，如果操作符的最后一个字符是冒号，那么操作符就是右结合的。

对于列表，在其前面增加元素是一个高效的 O(1) 的操作，而 append 则是一个 O(n) 的操作，其中 n 是列表的长度。相应地，可变的集合的追加操作就是高效的，例如 `ArrayBuffer`。这两种相反方向的集合适用于不同的场景。也可以使用 `::` 来创建列表，再调用 `reverse()`。列表还提供了其他一些函数：

```scala
val l = List(1, 2 ,3)
l.init          // List(1, 2)
l.tail          // List(2, 3)
l.drop(2)       // List(3)
l.dropRight(2)  // List(1)
```

`List()` 或 `Nil` 表示空列表。如果要从头使用双冒号来定义一个列表当然是可行的，但最后一个元素必须是 `Nil`，因为 `::()` 是列表上的方法。形如：

```scala
val list = 1 :: 2 :: 3 :: Nil
```

最后一个 `::` 需要在 `Nil` 上进行调用。

## 元组

元组实际上是在 `scala` 包中定义的一系列类。其访问方法是：

```scala
val t = (1, "a")  // class: scala.Tuple2[Int, String]
t._1  // 1
t._2  // "a"
```

元组的序号之所以从 1 开始是继承了其他语言，如 ML 的传统。

元组与列表的区别是，元组对每个元素保留泛型的类型参数，它能够保留每一个元素的类型信息，而列表不能。它只能保留所有元素的父类型。

```scala
(1, "a")  // Tuple2[Int, String]
List(1, "a")  // List[Any]
```

## Immutable / Mutable, Set 与 Map

在 Scala 中，`List` 总是 Immutable 的，`Array` 总是 Mutable 的。`Array` 还有长度可变的版本 `ArrayBuffer`。而对于 Set 和 Map，Scala 分别提供了可变与不可变的两种类型，使用包和 Trait 进行区分：

![Inherite Relationship of Set](../collection-hierarchy.png)

出于函数式的考虑，Scala 默认引入的是不可变的版本，使用可变的版本则需要显式调用。当然，也可以显式地指定要使用的集合的实现版本：

```scala
import scala.collection.mutable
val s = mutable.Set("a", "b")

import scala.collection.immutable.HashSet
val s2 = HashSet("b", "c")
```

在对这两种集合进行操作时，就会形成不同的模式：

```scala
var m = Set("a", "b")
m += "c"  // m is a new Set now

val m = mutable.Set("a", "b")
m += "c"  // still the Set before
```

对于 Map 来说，主要的区别是其使用二元元组（`Tuple2`）作为输入元素，这里用到了 Scala 中生成元组的函数 `->()`：

```scala
Map(1 -> "a", 2 -> "b")  // Map(1 -> a, 2 -> b)
1 -> "a"                 // (1,a)
```
