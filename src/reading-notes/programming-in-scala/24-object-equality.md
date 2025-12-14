# 24. 编写相等性方法

## 常见的 equals 方法的错误

编写 `equals` 方法并不是一件十分简单的事。常见的 `eqauls` 的错误包括：

错误的方法签名。Scala 中的 `equal` 应当是 `def equals(other: Any): Boolean`，使用 `Any` 之外的类型作为参数只会得到一个重载。而且，无论 Java 还是 Scala，使用哪一个重载都是由静态类型而非运行时类型决定的。就是说：

```scala
object A { def f(a: Any) = "Any"; def f(a: String) = "String" }
val s: Any = "a"
A.f(s)  // Any
```

常见的错误是使用 `def equals(other: this.type): Boolean`。这种情况下，如果外部传来一个父类引用持有的对象，被选择的就会是我们还没有覆盖的 `equals(other: Any)`，从而带来错误。进一步，Scala 还把 `Any` 中的 `==` 定义为 `final` 的，这样就不会犯 `def ==(other: Any)` 的错误。

另一个常见的错误是没有同时覆盖 `hashCode`，这会让对象在集合里的行为十分诡异。一个简单有效的办法是把所有有意义的值都放进一个 Tuple，然后使用 Scala 提供的方法，即 `override def hashCode = (a, b).##`。

基于类似的原因，不能把可变的值用于相等性的判断，这同样会让集合的行为十分诡异。如果你在一个 `HashSet` 中保存一个可变的对象，那么 `set contains elem` 将会是 `false`，因为 HashCode 发生了变化。但 `set.iterator contains elem` 又会是 `true`，因为它的确在集合的数据结构里。总之，不要这样做。

## 定义相等性

除了上面三种较为容易解决的问题之外，我们要来看一下相等关系的性质。相等关系应该满足这样的数学性质：

- 自反，即 `x equals x` 为真。
- 对称，即 `x equals y` 与 `y equals x` 相等。
- 可传递，即当 `x equals y`，`y equals z`，那么 `x eqauls z`。
- 一致，即多次调用同一个表达式的值相等。
- 对空值 NULL，值为假。

当面向对象的继承关系和相等性出现在一起时，情况就变得十分复杂了。考虑一个简单的 `Point` 类及其子类：

```scala
class Point(val x: Int, val y: Int) {
  override def hashCode = (x, y).##
  override def equals(other: Any) = other match {
    case that: Point => this.x == that.x && this.y == that.y
    case _ => false
  }
}
object Color extends Enumeration {
  val Red, Orange, Green, Blue = Value
}
class ColoredPoint(x: Int, y: Int, val color: Color.value) extends Point(x, y) {
  ...
}
```

我们直觉上写出来的代码大致是这样的：

```scala
override def equals(other: Any) = other match {
  case that: ColoredPoint => this.color == that.color && super.equals(that)
  case _ => false
```

注意我们这里并不需要重写 `hashCode` 方法（虽然推荐这样做）。原因是，实际上我们只需要保证两个相等的对象的 `hashCode` 相等，而不需要反过来保证。对于 HashMap 这样的类来说，其实现会首先使用 `hashCode` 进行查找，但最终仍然是通过 `equals` 方法来判断是否相等。毕竟避免哈希碰撞是大部分情况下是不可能的，而 HashMap 会使用拉链法来解决这个问题。

不过，这样的实现是存在问题的：

```scala
val p = new Point(1, 2)
val cp = new ColoredPoint(1, 2, Color.Red)
p == cp  // true
cp == p  // false
mutable.HashSet[Point](p) contains cp  // true
mutable.HashSet[Point](cp) contains p  // false
```

`Point` 类的 `equals` 方法只关心坐标，所以能够返回 `true`。这样，相等关系的对称性就被破坏了。

这时我们想到的修补方法可能是，允许 `cp equals p`。也就是：

```scala
override def equals(other: Any) = other match {
  case that: ColoredPoint => // same as before
  case that: Point => super.equals(that)
  case _ => false
}
```

现在 `cp == p` 和 `p == cp` 都是真值了。但是，相等关系的可传递性又被破坏了，因为 `redp == p`，`bluep == p`，但 `redp != bluep`，除非我们完全放弃对颜色相等性的判断。

看起来，放宽 `equals` 的条件是不可行的。更加可行的方式是，不允许 `ColoredPoint` 与 `Point` 相等。也就是说，在父类 `Point` 中：

```scala
override def equals(other: Any) = other match {
  case that: Point => this.x == that.x && this.y == that.y && this.getClass == that.getClass
  case _ => false
}
```

不过，这样定义的结果是，基于 `Point` 产生的匿名子类的对象：`new Point(1) { override val y = 2}` 和 p 也不相等了。

一种变通的方式是使用 `canEqual` 方法。

```scala
class Point(val x: Int, val y: Int) {
  override def hashCode = (x, y).##
  override def equals(other: Any) = other match {
    case that: Point => (that canEqual this) && (this.x == that.x) && (this.y == that.y)
    case _ => false
  }
  def canEqual(other: Any) = other.isInstanceOf[Point]
}

class ColoredPoint(x: Int, y: Int, val color: Color.Value) extends Point(x, y) {
  override def hashCode = (super.hashCode, color).##
  override def equals(other: Any) = other match {
    case that: ColoredPoint => (that canEqual this) && super.equals(that) && this.color == that.color
    case _ => false
  }
  override def canEqual(other: Any) = other.isInstanceOf[ColoredPoint]
}
```

大致上可以认为，我们手动判断了相等关系的可交换性。

## 泛型的相等性

考虑一个二叉树类的定义，先暂时忽略变型问题：

```scala
trait Tree[T] {
  def elem: T
  def left: Tree[T]
  def right: Tree[T]
}
object EmptyTree extends Tree[Nothing] {
  def elem = throw new NoSuchElementException("EmptyTree.elem")
  def left = throw new NoSuchElementException("EmptyTree.left")
  def right = throw new NoSuchElementException("EmptyTree.right")
}

class Branch[T]( val elem: T, val left: Tree[T], val right: Tree[T]) extends Tree[T] {
  override def equals(other: Any) = other match {
    case that: Branch[T] => this.elem == that.elem && this.left == that.left && this.right == that.right
    case _ => false
  }
}
```

但是，编译时，Scala 编译器会提示我们，由于类型擦除，模式匹配中的类型参数无法进行比较。大多数情况下这还可以接受（不同类型引用的对象很难相等），但当遇到继承关系时，就会出现问题：

```scala
val b1 = new Branch[List[String]](Nil, EmptyTree, EmptyTree)
val b2 = new Branch[List[Int]](Nil, EmptyTree, EmptyTree)
b1 == b2  // true
```

这里，由于实际的对象都是 `Nil`，两个对象被判断为相等。但实际上，二者的类型参数是不同的。至于这种情况应当判定为相等还是不相等，看法不一。不过，如果只是想消除编译器的警告的话，可以简单地标明类型参数的存在：

```scala
case that: Branch[_] => ...
```

这样，至少我们向编译器表明，我们的确知道这里可以是任何类型，编译器也不会再抛出 `unchecked` 警告。下划线也可以使用一个小写字母来代替，总之它不是这个方法所在对象的类型参数 `T`。相应地，`canEqual` 方法也去匹配一个 `Branch[_]` 类型。
