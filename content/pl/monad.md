+++
title = 'Scala: Monad, from Scala Perspective'
math = true
+++

# 从 Scala 视角看 Monad

## 群 Group

群是由一个集合与一个二元运算组成，满足四个性质：封闭性、结合律、单位元和逆元。以整数集 $\mathbb{Z}$ 与加法运算组合起来的群为例：

-   封闭性：所有集合内的元素经过二元运算得到的结果仍然在这个集合内。即，任意两个整数相加的结果仍为整数。
-   结合律：加法结合律 $(a+b)+c=a+(b+c)$。
-   单位元 / 幺元：存在一个元素与任意元素运算结果仍为那个元素。即，0 加任何整数结果都为那个整数。
-   逆元：对任意一个元素，总存在一个元素使得二者相运算结果为单位元。在这里即相反数。

如果省略逆元要求，则为一个幺半群（独异点，Monoid）。如果再省略单位元，则为一个半群（Semigroup）。从上面的结论可知，自然数与加法为一个幺半群，正整数与加法为一个半群。

群的性质落实到程序设计中形成了一定操作的可能性。因为集合在运算上的封闭，`op` 的参数和返回值是同样的类型。基于集合的归约操作也依赖运算的性质。因为运算满足结合律，所以归约操作可以并行化。因为幺半群的操作有幺元，我们可以从一个起始点进行归约，也就是 Scala 中的 `reduce` 和 `fold` 方法。

仍然以上面的加法为例。在求一个整数集合的和时，我们可以将集合拆分进行分布式计算，这是因为结合律。如果不使用 `reduce` 而使用 `fold` ,则需要一个初始值。这个初始值理所当然地就是 0，即运算的幺元。

从程序设计的角度来讲，幺半群可以用这样的形式表示。其封闭性通过类型参数保证，而幺元的正确性则需要我们自己来保证。

``` scala
trait Monoid[A] {
  def op(a1: A, a2: A): A
  def zero: A
}

val intAddMonoid = new Monoid[Int] {
  def op(a1: Int, a2: Int) = a1 + a2
  def zero = 0
}

val intMultiplyMonoid = new Monoid[Int] {
  def op(a1: Int, a2: Int) = a1 * a2
  def zero = 1
}

val stringMonoid = new Monoid[String] {
  def op(a1: String, a2: String) = a1 + a2
  def zero = ""
}

def listMonoid[A] = new Monoid[List[A]] {
 def op(a1: List[A], a2: List[A]) = a1 ++ a2
 def zero = Nil
}

def optionMonoid[A] = new Monoid[Option[A]] {
 def op(a1: Option[A], a2: Option[A]) = a1 orElse a2
 def zero = None
}
```

## 范畴 Category

范畴有三个条件：一个范畴由一系列物件 Object 构成的类 $ob(C)$ 和物件之间的态射组成的类 $hom(C)$ 构成。每一个态射是一个物件指向另一个物件的保持结构的一种关系。态射可以复合，就是说，如果对物件 $a,b,c$，有 $f:a\rightarrow b,g:b\rightarrow c$，那么存在 $g\circ f:a\rightarrow c$。一个范畴还满足两条公理：

-   这些态射满足结合律。$f\circ(g\circ h)=(f\circ g)\circ h$。
-   范畴存在单位元。即，对每一个物件 $x$，都有 $f:x\rightarrow x$。

可以看到，范畴的定义和群有些相似，但比群要更抽象，条件更少。一系列物件不一定是一整个类，也可以是一个一般的集合（称小范畴）；可以是一个很具体的对象，也可以是抽象的类型或集合。一系列态射也很泛化，可以是各种类型的态射，每个态射间也不一定非常相似。

例如，所有的群及群同态构成一个范畴 $Grp$。所有的集合及集合之间的全函数构成一个范畴 $Set$。所有预序关系（满足自反、传递）通过单调函数构成一个范畴 $Ord$。

一个幺半群是一个小范畴。其中只有一个物件，即幺半群中的集合，态射当然也只有一个单位态射，由幺半群的集合的元素给出，态射的复合由运算符给出。

一个有向图也是一个小范畴，其物件是图中的顶点，态射是有向图中的边。虽然并不是任意两个物件都能连接，但有向图的路径的确能够复合，满足结合律，也拥有单位态射。

范畴的概念之所以难以理解，可能是因为其太过于"高阶"。时刻需要记住的是，范畴是一个抽象程度非常高，限制非常少的概念。

落回到类型系统上。如果我们把一个函数看作一个类型到另一个类型的态射，那么一系列类型及它们之间的函数就是一个范畴。例如，整数类型，字符串类型和 `toString` 函数，加上两种类型的 `identity` 函数，构成一个范畴。

## 函子 Functor

这样做的目的是引入下一个概念：函子。函子是范畴到范畴之间的映射，也可以解释为**小范畴范畴**中的态射。花点时间思考一下这个概念：这个范畴中的物件为小范畴，态射为小范畴到小范畴的态射，也就是范畴到范畴之间的映射。函子被这样定义：

设 $C,D$为范畴。

-   对于 $C$ 中的每一个对象，函子 $F$，或者说这两个范畴之间的映射 $F$ 将 $\forall X \in C$映射至 $F(X)\in D$。
-   对每个态射 $\forall f:X\rightarrow Y\in C$ 映射至 $F(f):F(X)\rightarrow F(Y)\in D$。
-   $\forall X\in C$，很自然地，有 $F(\mathbf{id}\_X)=\mathbf{id}\_{F(X)}$。
-   $\forall f: X\rightarrow Y\in C,g:Y\rightarrow Z\in C$，有 $F(g\circ f)=F(g)\circ F(f)$。

最后，由一个范畴映射到它本身的函子称作自函子。以所有的小范畴为物件，以函子为态射，也可以组成一个范畴，称**小范畴范畴**。自函子即是其中的单位态射。

再次落回到类型系统上来。如果我们将一系列函数连接起来的类型称为一个范畴，那么函子就可以是这样的两个范畴之间的映射。依然是上面的例子，一个范畴由：

``` scala
Int
String
def toString(i: Int): String = i.toString
def identity(i: Int): Int = i
def identity(s: String): String = s
```

组成，另一个范畴可能由：

``` scala
List[Int]
List[String]
def mapToString(ints: List[Int]): List[String] = ints map toString
def identity(ints: List[Int]): List[Int] = ints
def identity(strs: List[String]): List[String] = strs
```

那么，这两个范畴可以用一个函子进行映射。前者可以抽象为 `Identity` 的范畴，后者抽象为 `List` 的范畴，而这个函子把 `Int` 和 `String` 装进列表，并将对应的态射映射为对应的集合版本。

可以看到，我们真正要做的重要的事情是，把 `toString` 映射成 `mapToString`。不过，如果用代码来实现的话，我们就没有必要用 `Int` 和 `String` 来限制自己了。一个 `List` 的函子可以描述为：

``` scala
object ListFunctor {
  def map[A, B](fa: List[A], f: A => B): List[B] = fa map f
}
// 使用函子
ListFunctor.map[Int, String](List(1, 2, 3), _.toString)
ListFunctor.map(List(1, 2, 3), (i: Int) => i.toString)
```

在上面的调用中，参数 `f` 就是我们要映射的范畴中的态射。如果我们再将函子抽象一下的话，就需要使用高阶类型：

``` scala
trait Functor[F[_]] {
    def map[A,B](fa: F[A], f: A=>B): F[B]
}
```

这样，无论 `F` 是 `Option` `Array` 还是其他类型，我们都可以为它们定义适合的 `map` 函数。当然，这里并没有为范畴中的物件做映射。在我们这个例子里，这件事由`List.apply` 完成。这个通用的函子 Trait 能够将类型 `A B` 所在的范畴映射到 `F[A] F[B]` 所在的范畴。

剩下的问题是，为每一种类型映射（通常是泛型）都定义一个函子似乎太过麻烦。实际上，我们这里的 `map` 函数和 `List` 类里自带的 `map` 并没有什么区别。也就是说，`List` 这样的自带 `map` 函数，能够将一个范畴里类型间的态射转换成另一个范畴里类型间的态射（这里是从 `Int => String` 映射为 `List[Int] => List[String]`）就是一个函子。Scala 里的很多容器都是函子，这些容器通常也直接用 `apply` 方法提供了类型的映射。对于态射的组合，我们则可以通过链式的 `map` 或者对参数使用 `andThen` 来解决。

``` scala
// 并非真实的标准库
object List{
  def apply(elem: A) = new List(elem)   //映射物件
}
class List[A] {
  def map[B](f: A => B): List[B] = ...  //映射态射
}

List(1).map(_.toString)
List(1).map((i: Int) => i.toString)
```

### 函子与逆变、协变

函子相关的还有两个概念：协变函子和反变函子（逆变函子）。上面描述的即为协变函子，反变函子则是将态射的方向反转，即：

-   $\forall f:X\rightarrow Y,F(f):F(Y)\rightarrow F(X)$
-   $\forall f: X\rightarrow Y\in C,g:Y\rightarrow Z\in C,F(g\circ f)=F(f)\circ F(g)$

即，反变函子将态射的方向反转过来。

我们已经知道容器是一个典型的函子。如果我们将一个范畴中类型的继承关系作为态射，那么函子对继承关系的映射的结果应当是这些类型的对应容器类的继承关系，于是我们就能够联想到我们熟悉的泛型类型的逆变和协变。显然，容器是协变函子意味着容器是协变的，容器是逆变函子意味着容器是逆变的。上面的 `List` 就是一个协变的容器：

``` scala
class List[+A] {
  def map[B](f: A => B): List[B] = ...
}
```

在 TypeLevel 提供的 cats 库中，提供了逆变函子的定义，其对应的 `contramap` 函数定义如下：

``` scala
def contramap[A, B](fa: F[A])(f: B => A): F[B]
```

这个函数将 `A => B` 映射为 `F[B] => F[A]`，典型的例子如 `Ordering.on` ：

``` scala
def on[U](f: U => T): Ordering[U] = new Ordering[U] {
  def compare(x: U, y: U) = outer.compare(f(x), f(y))
}
```

这个函数将 `U => T` 映射为 `Ordering[T] => Ordering[U]`。

## 自函子范畴上的幺半群

从 $C$ 到 $D$ 的函子范畴，记作 $[C, D]$，是以所有的协变函子 $F:C\rightarrow D$ 为对象，以函子之间的自然变换为态射的范畴。如果 $C,D$ 是同一个范畴，那么函子 $F$ 就是自函子，这个范畴也就是自函子范畴。

如果 $C,D$ 是范畴，$F,G$ 是二者之间的函子。那么 $\forall X\in C$，给出一个在 $D$ 的对象间的态射： $\eta_X:F(X)\rightarrow G(X)$，称为 $\eta$ 在 $X$ 处的分量，使得 $\forall f:X\rightarrow Y\in C$，有 $\eta_Y\circ F(f)=G(f)\circ\eta_X$。

![](https://upload.wikimedia.org/wikipedia/commons/thumb/f/f2/Natural_transformation.svg/175px-Natural_transformation.svg.png)

解释这个概念时常用 Haskell 中的概念 **Hask 范畴**。这个范畴的物件是 Haskell 中所有的类型，态射是所有的*全函数*，态射的复合类似于 `andThen`。对应到 Scala，也就类似于我们上面的 `Int String List[Int] List[String] List[List[Int]]` 等等所有类型组成的范畴。于是，我们上面的 `ListFunctor` `OptionFunctor` 等等函子就全部都是 Hask 范畴上的自函子。在这个范畴上看自然变换，例如，取 $X$ 为 `Int` $Y$ 为`String` $F$ 为 `List` $G$ 为 `Option`，那么相应地，$f$ 为 `Int.toString`，$F(f)$ 和 $G(f)$ 就是 `List[Int].map(_.toString)` 和 `Option[Int].map(_.toString)`。于是，自然变换 $\eta$ 就是一个从 `List` 容器到 `Option` 容器的映射 `List[T] => Option[T]`。

于是，以自函子为物件、以自然变换为态射可以构成一个范畴，也就是**自函子范畴**。这时，我们在范畴这个概念的基础上又抽象了一层。

那么，我们希望这个范畴具有什么样的性质，或者说我们希望这些态射是什么样的呢？在这里，对于函子 $M$，我们希望单位态射 $\eta:1_H\rightarrow M$，（这里的 $1_H$ 表示 范畴中的物件 $H$ 的单位态射）态射的复合 $\mu：M\circ M\rightarrow M$。符合这样性质的自函子就是单子 Monad。（由于单子是一个自函子，所以只需要一个态射及其复合即可定义它。）显然，这个复合操作完美地复合封闭性、结合律和单位元。因此，单子是一个自函子范畴上的幺半群。

## 用 Monad 解决回调地狱

考虑一个简单的除法函数：

``` scala
def safeDiv(a: Double, b: Double) = b match {
  case 0 => None
  case _ => Some(a / b)
}
```

如果我们想对一系列数字做链式的除法，会发生什么？可能有这样的两种实现：

``` scala
safeDiv(6, 3) match {
  case None => None
  case o => safeDiv(2, o.get)
}

safeDiv(6, 3).map(safeDiv(2, _))  // type: Option[Option[Double]]
```

注意这里的 `Option` 是一个函子，在第二种实现中，我们使用函子的 `map` 解决了 `safeDiv` 无法适用于 `Option` 类型的问题。显而易见，无论哪一种体验都不会太好。所以，我们可以定义这样一个方法，将我们从多层嵌套的 `Option` 中解救出来：

``` scala
def joinOption[T](o: Option[Option[T]]) = o match {
  case None => None
  case _ => o.get
}
joinOption(safeDiv(6, 3).map(safeDiv(1, _)))  // type: Option[Double]

implicit class JoinableOption[T](o: Option[Option[T]]) {
  def join = o match {
    case None => None
    case _ => o.get
  }
}
safeDiv(6, 3).map(safeDiv(2, _)).join.map(safeDiv(1, _)).join  // type: Option[Double]
```

这里的 `join` 就实现了我们上面提到的态射的复合：$M\circ M\rightarrow M$。也就是说，一个 `Option` 紧接着一个 `Option` 的结果仍然是一个 `Option`。因此，`Option` 是一个单子。

我们对这个 `join` 还是比较满意的。那么，按照上面的例子，如果这里的容器是 `List`，`join` 应该怎样实现呢？这个函数应当把两层嵌套的 `List` 变成单层的 `List`，也就是我们熟悉的 `flatten`。`flatten` 和 `map` 结合起来，我们发现实际上这种情况需要的是 `flatMap`。

``` scala
def explode3(i: Int) = List(i, i, i)
List(1, 2, 3).map(explode3).flatten  // type: List[Int]
List(1, 2, 3).flatMap(explode3)  // type: List[Int]
safeDiv(6, 2).flatMap(safeDiv(2, _))  // type: Option[Double]
```

同时，我们也印证了之前的想法：`flatMap` 的抽象是一种 Monad。在 Haskell 中，则是实现 `Join`，即 `>>=`（bind）算符，即成为一个 Monad。要定义一个严格的 Monad，我们通常需要它拥有两种操作：

``` scala
trait Monad[M[_]] {
 def unit[A](a: A): M[A]   //identity
 def join[A](mma: M[M[A]]): M[A]
}
// or, another expression
trait Monad[M[_]] {
 def unit[A](a: A): M[A]
 def flatMap[A, B](fa: M[A])(f: A => M[B]): M[B]
}
// or, another way
trait Monad[M[_]] {
  def unit[A](a: A): M[A]
  def compose[A, B, C](f: A => M[B], g: B => M[C]): A => M[C]
}
```

无论哪一种，都能够体现 $M\circ M\rightarrow M$。

或者说，对于我们一直使用的容器类型的例子，取 `M` 为 `List`，这个容器应该具有：

``` scala
object List[T] {
  def apply(elem: T): List[T] = ???
}
class List[T] {
  def flatMap[B](f: T => List[B]): List[B] = ???
}
```

当然，真实的 Scala 库使用了更加泛用的写法。

更重要的是，在 Haskell 和 Scala 中都为 Monad 提供了语法糖（`do` 和 `for`），以优雅地解决这里的拆包问题。

``` scala
val o = Some(Some(Some(1)))

for { x <- o; y <- x; z <- y } yield z
// results Some(1), in order to keep structure like o
for { x <- o; y <- x; z <- y } println(z)
// results 1, for z is an Int(1)
```

注意，这里 `z` 的值是 `Int` 1，但如果使用 `yield`，将会产生一个 `Some(1)`。这是因为 `yield` 的通常做法是从一个集合生成另一个集合，所以进行了一次打包。如果用 `List` 能够描述得更清楚：

``` scala
val l = List(List(1, 2), List(3, 4))
for {x <- l; y <- x} yield y
// results List(1, 2, 3, 4)
```

Haskell 中的语法糖也类似：

``` haskell
safeDiv a b >>= (\x -> safeDiv c x >>= (\y -> safeDiv d y >>= (\z -> safeDiv e z)))

al = do
  x <- safeDiv a b
  y <- safeDiv c x
  z <- safeDiv d y
  safeDiv e z
```

## Functor, Applicative, Monad

现在我们从另一个方向来理解 Monad 等概念。在前面的例子中，我们都是使用的"容器"来理解，但实际情况中完全可以不只是容器。Haskell 世界常用上下文 Context 来表示这个概念。在 Scala 世界中，也不仅仅容器可以接收类型参数。首先回顾一下 Functor 和 Monad，在下面的描述中，我们都省略比较显而易见的 `unit` 的定义：

函子解决了这样一个问题，将一个完全 Context 外（即接收 Context 外值并返回 Context 外的值）的函数应用在一个 in Context 的值上（例如容器中的值），得到一个 in Context 的值，表现为 `map` 方法。如果将 Context 看做一个盒子，那么就相当于在 `map`（Haskell 中的 `fmap`）的内部，将盒子中的值拿出来，应用函数，再装回同样的盒子里去。

``` scala
def map[A, B](a: F[A])(f: A => B): F[B]
// A => B ➡️ F[A] => F[B]
```

单子解决了这样一个问题，将一个本来接收 Context 外的值，而返回 in Context 值的函数，应用到一个 in Context 的值上，并返回一个"单层的" in Context 的值。如果使用盒子比喻的话，那么相当于减少了盒子的层数。

``` scala
def join[A](a: M[M[A]]): M[A]  // flatten
def flatMap[A, B](fa: M[A])(f: A => M[B]): M[B]  // map + join
// A => M[B] ➡️ M[A] => M[B]
def compose[A, B, C](f: A => M[B], g: B => M[C]): A => M[C]  // unit + map + join
// A => M[B] => M[M[C]] ➡️ A => M[C]
```

我们之前已经知道单子是一个函子。因为只需要满足单子所需的条件，就已经是一个函子，所以，`map` 完全可以通过 `flatMap` 来实现：

``` scala
def map[A, B](fa: F[A])(f: A => B): F[B] = flatMap(fa)((a: A) => unit(f(_)))
```

现在我们引入另外一个概念，可应用函子 Applicative。数学的角度上并不常提起它，编程的角度上则重要一些。

可应用函子解决了这样一个问题：将一个 in Context 的函数，应用到一个 in Context 的值上。也就是：

``` scala
def apply[A, B](fab: F[A => B])(fa: F[A]): F[B]
// F[A => B] ➡️ F[A] => F[B]
def map2[A, B, C](fa: F[A], fb: F[B])(f: (A, B) => C): F[C]
```

`apply` 函数非常符合我们的定义，`map2` 则不是非常直观。更广泛地说，任意数量参数的 `map` 都属于 Applicative 的管辖范围。通过柯里化，我们能够从 `apply` 构造 `map2`。在柯里化的语境下，`(A, B) => C` 同时也是一个 `A => (B => C)`。所以有：

``` scala
def map2[A, B, C](fa: F[A], fb: F[B])(f: (A, B) => C): F[C] = {
  val fb2c: F[B => C] = apply(unit(f))(fa)  // apply[A, B]
  val fc = apply(fb2c)(fb)  // apply[B, C]
  fc
}
```

可以看出的是，在面向对象的世界中可应用函子并不常见，一个 `apply` 形式的场景是从高阶函数中获得一个用 `Option` 包裹起来的函数，并应用到另一个 `option` 值上。相比之下，`map2` 的用法要常见得多，但经常是将多个容器 zip 起来以元组作为参数列表，再利用 lambda 表达式将 `map2` 变成 `map`。

这时回头看，我们发现单子同时也是一个可应用函子，因为 `map2` 完全可以用 `flatMap` 实现：

``` scala
def map2[A, B, C](fa: F[A], fb: F[B])(f: (A, B) => C): F[C] =
  flatMap(fa)(a => map(fb)(b => f(a, b)))
```

回到 Scala 的定义中，我们得到了：

``` scala
trait Functor[F[_]] {
  def unit[A](a: A): F[A]
  def map[A, B](f: A => B): F[B]
}
trait ApplicativeFunctor[F[_]] extends Functor {
  def apply[A, B](f: F[A => B])(fa: F[A]): F[B]
}
trait Monad[M[_]] extends ApplicativeFunctor {
  def join[A](mma: M[M[A]]): M[A]
}
```

## 用 Monad 隔离副作用

IO 是 Haskell 中的一个 Monad（Haskell 选择将 Monad 显式地定义出来）。具体来说，Haskell 中的Monad 是一个 type class，可以暂时简单地理解为类似 `interface` 或 `trait` 的对方法的抽象，只是更加灵活。为了保证函数的纯正性，如果参数中包括 IO，Haskell 就要求返回值必须存在 IO。因此这样的"伪装"变得不可能：

``` haskell
Char -> Char = (Char -> IO Char) . (IO Char -> Char)
```

这里的 `.` 表示函数的复合，即我们之前说的态射的复合，大致相当于 Scala 中的 `andThen`。

通过对 IO 的这种要求，Haskell 保证将纯函数和非纯函数区分开来。不过，由于 `IO Char` 和 `Char` 并不一样，我们需要一种方式让真对 `Char` 的函数能够作用在 `IO Char` 上。这样，这种情况就十分类似于我们之前的 in Context 问题了。

在 Haskell 的 Monad 中，`unit` 操作叫做 `return`，`join` 或 `flatten` 操作叫做 `>>=`（bind）。让我们先从 IO Monad 的实际使用开始：

``` haskell
getChar :: IO Char
putChar :: Char -> IO ()
echo = (getChar >>= putChar) :: IO ()
```

将这个过程翻译成对等的 Scala，大致相当于：

``` scala
def getChar: IO[Char] = Some('a')  // IO[Char]
def putChar(c: Char): IO = { println(c); None }  // Char => IO
def echo = getChar flatMap putChar  // IO
```

因为 Haskell 中无法对 IO "拆包"，所以包含了 IO 的代码块就会始终包含着 IO。同时，通过 Monad，我们能够把不在 IO Context 内的函数应用到 IO 的变量身上。

所以，Haskell 对副作用的处理总结起来是这样的：将 IO 作为一个"标签"打在过程上，并且凡是和副作用有关的上层函数都会被打上这个标签，当遇到普通的纯函数时，通过函子的特性将函数应用在实际的变量上，并在结果中保留 IO 标签。特殊情况是，当遇到另外一个同样有副作用的函数时，就会出现两个 IO "标签"。这时，通过 Monad 的特性将 IO 标签限制到一个。同时，由于 Monad 没有定义 `IO -> ()`，所以打上 IO 标签的函数永远不可能变回纯函数。
