# 9. 特质 Trait

## Trait

Trait 和 Java 的接口的最大差别是我们混入（mix in）Trait 而不是实现接口。混入一个 trait 有两种方式：

```scala
class A
trait T extends A
object O extends T
val v: A = O
```

如果使用 `extends` 关键字，那么新的对象就会继承于 trait 的父类。如果 trait 没有显式继承，那么这个对象理所当然地会继承于 `AnyRef`。

```scala
trait Tr
object Obj extends A with T with Tr
val v: T = Obj
val v: A = Obj
val v: Tr = Obj
```

## 瘦接口 VS 胖接口

在 Java 8 之前，Java 世界通常使用的是瘦接口。例如，虽然 String 的大部分方法都可以用于任何字符序列，`CharSequence` 接口提供的仍然很少。在没有默认方法的情况下，实现一个胖接口是一件十分累人的工作。在 Java 8 引入默认方法之前，这个接口只有四个方法：

```java
public interface CharSequence {
    int length();
    char charAt(int index);
    CharSequence subSequence(int start, int end);
    public String toString();
    public default IntStream chars() {...}
    public default IntStream codePoints() {...}
}
```

由于历史原因，即使到了 Java 8，为了与旧库的兼容，我们也不敢再贸然加入太多的新默认方法了。Java 世界的常规做法是定义一个抽象类来加入 Java 8 的默认方法所做的事情，再继承，但这样又失去了定义接口的意义。在 Scala 中，你只需要实现 `compare`，就能得到一系列方法：

```scala
trait Ordered[A] extends Any with java.lang.Comparable[A] {
  def compare(that: A): Int
  def <  (that: A): Boolean = (this compare that) <  0
  def >  (that: A): Boolean = (this compare that) >  0
  def <= (that: A): Boolean = (this compare that) <= 0
  def >= (that: A): Boolean = (this compare that) >= 0
  def compareTo(that: A): Int = compare(that)
}
```

## 混入

```scala
class A { def f(s: String) = print(s + " from class A") }
class B
trait C extends A { override def f(s: String) = super.f(s + " from trait C") }
```

这样编写特质之后，我们发现的第一件事是，因为我们在 `C` 的定义中明确了 `A`，所以 `B` 不能混入 `C`，因为它不是 `A` 的子类。

```scala
class D extends B with C  // error: illegal inheritance;
class D extends A with C
```

其次，值得注意的是 `C` 中定义了一个包含 `super` 的方法。这个定义在传统的面向对象模式中比较奇怪，如果是一个类，我们可以明确地知道 `super` 指向哪一个类的哪一个方法，而对于 `interface` 来说则不能。这体现了 Scala 中 trait 可堆叠的特性。类的 `super` 是静态绑定的，而 trait 中的 `super` 则是动态绑定的。这也是为什么这个方法的标签为 `override`。另外，我们也可以直接在定义 `object` 时混入 trait，甚至在 `new` 一个对象时混入：

```scala
object o extends A with C
o f "abc"               // abc from trait C from class A
(new A with C) f "abc"  // abc from trait C from class A
```

接下来我们来**堆叠** trait。

```scala
trait D extends A { abstract override def f(s: String) = super.f(s + " from trait D") }
(new A with C with D) f "abc" // abc from trait D from trait C from class A
(new A with D with C) f "abc" // abc from trait C from trait D from class A
```

基本上来说，`super` 的顺序取决于混入的顺序。换句话说，在处理多重继承问题时，混入的顺序就决定了方法动态绑定时线性化的顺序。在多重继承这个问题上，Java 的做法是，类优先于接口的默认方法，两个接口的默认方法的冲突则会报错。在 Scala 中，两个 trait，或一个 trait 和一个类中的普通方法相遇时也会报错，而对于有父类的 trait 中的 `abstract override` 方法，则能够通过这种方式来扩展功能。举个这样的例子：

```scala
abstract class IntItem { def get: Int; def put(x: Int) }

trait DoubleItem extends IntItem { abstract override def put(x: Int) = super.put(2 * x) }
trait PlusOneItem extends IntItem { abstract override def put(x: Int) = super.put(1 + x) }

class BasicIntItem extends IntItem { private var i: Int = 0; def get = i; def put(x: Int) = {i = x} }

val a = new BasicIntItem with DoubleItem with PlusOneItem
a.put(2)
a.get  // 6

val b = new BasicIntItem with PlusOneItem with DoubleItem
b.put(2)
b.get  // 5
```

在这个例子里，因为父类 `IntItem` 是一个抽象类，所以 trait 中的类也要一起声明为抽象类（因为其中的 `super.put` 方法还是抽象的）。然后，我们提供了一个实现 `BasicIntItem`。这样，就能体现出 trait 中方法的动态绑定，`super.put` 最终被绑定到了 `BasicIntItem.put` 上去。然后，可以看到由于混入顺序的不同，方法的调用顺序不同，最终产生的逻辑也不同。如果我们在方法中加入的功能是可以**堆叠**的，就应该考虑这类方法。

## 细节和使用

首先来讨论一下什么时候应该使用特质。首先，如果一个特性可以出现在一些互相关系并不大的类的类中时，特质通常是一个比抽象类更好地选择。这和"抽象类描述本质而接口描述功能"的想法是统一的。另外，在分发代码或者和 Java 协作是也要考虑。Java 可以自如地继承 Scala 类，继承特质就比较麻烦。例外是，完全抽象没有实现体的特质会直接被翻译成接口，所以这种没有问题。此外，在分发代码时，如果特质发生了改变，其下游的类需要跟随进行重新编译。所以如果一个接口可能被下游继承实现，抽象类可能是更好的选择。

堆叠这种方式是解决多重继承的其中一个解决办法，可以参考 Python 使用的 [C3 线性化](https://zh.wikipedia.org/wiki/C3%E7%BA%BF%E6%80%A7%E5%8C%96)来更好地理解它。由于 Scala使用了单继承 + trait 的方式，类的线性化方式比 C3 要简单。在 Java 中遇到方法冲突时，或者是 Scala 中的非 `override` 方法发生冲突时，我们会采用这样的方式：

```java
interface B { public default void f(String s) { System.out.println(s + " from interface B"); } }
interface C { public default void f(String s) { System.out.println(s + " from interface C"); } }

class D implements B, C {}  // ERROR, method conflict, won't compile

class D implements B, C { public void f(String s) { System.out.println(s + " from class D"); } }
new D().f("abc") // abc from class D
```

或者在 Scala 里：

```scala
trait A {def f = 1}
trait B {def f = 2}
class C

new C with A with B { override def f = 3 }.f  // 3
```

和堆叠 trait 相比，这样的方式很难同时继承两个接口方法的功能。除非你能够把两个方法都重新实现一次，如果我们写出这样的方法（伪代码）：

```scala
class A { def f(s: String) = println(s + " from A") }
trait B extends A { override def f(s: String = super(s + " from B")) }
trait C extends A { override def f(s: String = super(s + " from C")) }
object o extends A with B with C ( override def f(s: String) = { B.super(s); C.super(s)})
o.f("abc")
// abc from B from A
// abc from C from A
```

由于这里的菱形继承关系，父类 `A` 的方法会被调用两次，这是我们不想看到的。

## 线性化规则

最后我们来谈谈线性化的规则。相比 Python 或 C++ 这种支持多继承的语言，单继承 + 接口的语言的线性化要简单一些。基本的原则是，一个类型的线性化的后半部分是其父类的线性化。

```scala
class Animal
trait Furry extends Animal
trait HasLegs extends Animal
trait FourLegged extends HasLegs
class Cat extends Animal with Furry with FourLegged
```

于是根据这个规则我们有：

![inheritance graph](https://g.gravizo.com/svg?%20digraph%20G%20%7B%20rankdir=BT%20Animal%20-%3E%20AnyRef%20-%3E%20Any%20Furry%20-%3E%20Animal%20HasLegs%20-%3E%20Animal%20FourLegged%20-%3E%20HasLegs%20Cat%20-%3E%20FourLegged%20Cat%20-%3E%20Furry%20Cat%20-%3E%20Animal%20%7D)

`Animal`、`Furry` 和 `Four` 的线性化显而易见。对于 Cats，按照 trait 混入的顺序。`Furry` 第一个被混入，因此也第一个线性化，得到 `Furry -> Animal -> AnyRef -> Any`。然后混入 `FourLegged`，得到 `FourLegged -> HasLegs -> Furry -> Animal -> AnyRef -> Any`，最后线性化 `Cats` 本身，这个顺序也就是 `super` 的查找顺序。
