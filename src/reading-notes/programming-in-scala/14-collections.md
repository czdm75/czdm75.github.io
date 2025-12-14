# 14. 集合

`Iterable` 指代一个集合，而 `Iterator` 指代对这个集合的操作。实际上，`Iterator` 继承了 `IterableOnce` 这个 trait。所以 `Iterator` 只能遍历一次。

## 序列、集合、映射 Seq Set Map

Scala 的数组和 Java 数组是对齐的，可以直接进行操作，虽然形式上 Scala 的数组是泛型的，这使得它们在变型（逆变和协变）问题上略有不同。

`ListBuffer` 是可变版本的 `List`。在循环中拼接一个 `List` 时，要么需要使用 `var`，要么需要进行递归。如果操作不是尾递归，就有必要使用 `ListBuffer` 来控制栈的深度。

`Queue` 和 `Stack` 包括可变和不可变的版本，分别使用 `enqueue` `dequeue` 和 `push` `pop` 来操作。对于不可变的版本，取元素操作会返回一个 `Tuple[T, Queue[T]]`。

字符串的隐式转换类 `RichString` 也是一个 `Seq[Char]`。

`Set` 和 `Map` 都可以使用 `+` `-` 来增加或删除元素，使用 `++` `--` 删除一个集合里的元素。对于可变版，还可以使用对应的 `+=` `-=` 等。对 `Map`，删除时只需要传入 Key，增加时则是传入一个元组。用箭头来生成 `Tuple2[A, B]` 作为键值对的语法就适合这种场景。

出于性能优化的原因，Scala 为 0\~4 个元素的不可变集合与映射直接提供了类 `scala.collection.immutable.Map.Map4` 等一些，更大容量的则使用 `HashSet` 作为默认。这种小尺寸的集合比可变版本的更加紧凑，更加节省内存，访问时也通常更节省时间。

Scala 只提供了不可变版本的排序后的集合 `TreeSet` `TreeMap`，基于红黑树实现。其元素需要混入 `Ordered` 或能够隐式转换为 `Ordered`。

```scala
import scala.collection.immutable.TreeSet
var t = TreeSet(1, 3, 4)
t += 2
t  // TreeSet(1, 2, 3, 4)
```

在这个例子中，虽然 `t` 指向一个不可变集合，但因为它是一个 `var`，所以 `t` 会指向一个新创建的对象。这个原则适用于所有 `Immutable var`，只要操作符以 `=` 结尾。

`Set` 和 `Map` 都有 `toArray` 和 `toList` 方法，虽然会造成元素的拷贝。生成的顺序与 `elements` 方法的返回值一样。如果要从其它集合转换为 `Set` 和 `Map`，或者在可变与不可变之间转换，就需要使用 `++`。

## 元组

之前已经提到过模式匹配的一个特例：

```scala
val (a, b) = (1, 2)
// a: Int = 1
// b: Int = 2

val a, b = (1, 2)
// a: (Int, Int) = (1,2)
// b: (Int, Int) = (1,2)
```

第二种情况更类似 C 风格的传统代码。
