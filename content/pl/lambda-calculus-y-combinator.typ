#let meta = (
  title: "Lambda Calculus and Y Combinator",
  lang: "zh",
  year: 2019
)

= Lambda 演算与 Y 组合子

== λ 演算

=== λ 项

只有三种有效的 λ 项：

- 一个变量 $x$
- 一个抽象 $lambda f.lambda x.x$，大致上等同于 Python 中的 `lambda f, x: x`
- 函数的应用 $t s$，大致上等同于 Python 中的 `t(s)`

=== α-等价

在一个抽象中，变量的名字并不重要。例如 $lambda x.x$ 和 $lambda y.y$ 是 α-等价的。

描述这种变换的一种记法是使用：$t[x:=r]$，表示在 $t$ 中将所有的 $x$ 重命名为 $r$。于是有：

- $x[x:=r]=r$，将 $x$ 替换为 $r$
- $y[x:=r]=y$，$y$ 中不包括 $x$，无需替换
- $(t s)[x:=r]=(t[x:=r])(s[x:=r])$，对应用，将两部分分别替换
- $lambda x.t[x:=r]=lambda x.t$，替换前后并没有区别（α-等价）
- $lambda y.t[x:=r]=lambda y.t[x:=r]$

=== 自由变量与约束变量

自由变量被定义为这样的集合：

- 对于变量 $x$，其自由变量集合仅包括 $x$
- 对于抽象 $lambda x.t$，其自由变量集合为 $t$ 的自由变量集合去掉 $x$。
- 对于应用 $t s$，其自由变量为 $t$ 和 $s$ 自由变量集合的并集。

定义这些的目的是在进行β-归约时避免变量名的冲突。不严谨地说，自由变量就是"可以被替换"的变量集合。

=== β-归约

β-归约表示，在应用函数时，可以直接对结果中相应的变量进行替换。即，$(lambda x.t)s=t[x:=s]$。

在无类型 λ 演算中，有些形式无法被化到最简（称范式），如：

$
(lambda x. x x)(lambda y. y y) = (lambda y. y y)(lambda y. y y)
$

甚至，对于 $lambda x. x x y$，反而会越来越长（$y$ 越来越多），顺序也会影响归约会否进入死循环。好在，有两个定理解决了这个问题：

- Church-Rosser 定理：如果一个 λ 项有范式，那么这个范式（在α-等价的意义上）是唯一的。
- 对一个 λ 项，始终归约其最左侧最外侧的可约式，总能得到β-范式（如果存在）。

=== η-变换

η-变换并不是必需的，实际上它是前两者的一个推论：$lambda x.f x=f$，其中 $x$ 不是 $f$ 中的自由变量。

== 邱奇数

邱奇数是为 λ 演算而生的数的表示方式，同样基于皮亚诺公理（使用一个基础，即0或1，以及一个后继的定义来描述自然数）。我们令：

$
"zero" &= lambda f.lambda x.x \
"succ" &= lambda n.lambda f.lambda x.f(n f x)
$

记得柯里化的原则：$n f x=n(f)(x)$。于是，就能得到：

$
"one" &= "succ" "zero" \
&= (lambda n.lambda f.lambda x.f(n f x))(lambda g.lambda y.y) \
&= lambda f.lambda x.f x \
"two" &= lambda f.lambda x.f(f x) \
"three" &= lambda f.lambda x.f(f(f x))
$

在这个定义中，$f$ 的次数就意味着自然数的值。因此可以有：

$
"plus" &= lambda m.lambda n.m "succ" n \
"plus" "one" "two" &= (lambda g.lambda y.g y) (lambda n.lambda f.lambda x.f(n f x)) "two" \
&= (lambda y.(lambda n.lambda f.lambda x.f(n f x))y) "two" \
&= (lambda y.(lambda f.lambda x.f(y f x))) "two" \
&= (lambda n.lambda f.lambda x.f(n f x)) "two" \
&= "succ" "two"
$

同理，有：

$
"mult" &= lambda m.lambda n.lambda f.n(m f) \
"exp" &= lambda m.lambda n.n m \
"pred" &= lambda n.lambda f.lambda x.n(lambda g.lambda h.h(g f))(lambda u.x)(lambda u.u) \
"sub" &= lambda m.lambda n.m "pred" n
$

== 邱奇逻辑

$
"true" &= lambda a.lambda b.a \
"false" &= lambda a.lambda b.b \
"and" &= lambda p.lambda q.p q p \
"or" &= lambda p.lambda q.p p q \
"if" &= lambda p.lambda a.lambda b.p a b
$

更进一步，可以基于这些进行谓词逻辑的运算：

$
"iszero" &= lambda n.n (lambda x."false") "true" \
"leq" &= lambda m.lambda n."iszero" ("sub" m n) \
"eq" &= lambda m.lambda n."and" ("leq" m n)("leq" n m)
$

== Y 组合子

接下来考虑一个递归函数：

$
"fact" = lambda n."if"("iszero" n) "one" ("mult" n ("fact"("pred" n)))
$

直接进行递归是不行的，这里只是一个简单的演算，$"fact"$ 这个名字，甚至所有的名字都是不必要的。一种方式是这样的：

$
"fact1" &= lambda f.lambda n."if"("iszero" n) "one" ("mult" n (f f("pred" n))) \
"fact" &= "fact1" "fact1"
$

通过这种方式，使用 $f f$来代替原来的递归，就能做到调用自身。不过，这样做仍然不够优雅。因此，Haskell 和图灵分别发现了两个不动点组合子（常用的是前者）：

$
"Y" &= lambda f.(lambda x.(f(x x))(lambda x.(f(x x)))) \
"Theta" &= (lambda x.lambda y.(y (x x y))) (lambda x.lambda y.(y (x x y)))
$

由于其前半部分和后半部分相同，经常也写作：

$
"Y"=lambda f.(lambda g.g g)(lambda x.f(x x))
$

经过简单的演算，可以得到：

$
"Y" g &= lambda f.(lambda x.(f(x x))(lambda x.(f(x x)))) g \
&= (lambda x.(g(x x)))(lambda y.(g(y y))) \
&= g ((lambda y.(g(y y)))(lambda y.(g(y y)))) \
&= g ("Y" g)
$

除了符合函数不动点的定义之外，更重要的是在程序中它可以"再次调用函数"，直到抵达递归中的结束条件。需要注意的是这两种形式都使用于 call-by-name 的求值策略，传值调用下需要稍微修改一下形式：

$
"Z" &= lambda f.(lambda x.f(lambda y.x x y))(lambda x.f(lambda y.x x y)) \
"Theta_V" &= (lambda x.lambda y.(y(lambda z.x x y z)))(lambda x.lambda y.(y(lambda z.x x y z)))
$
