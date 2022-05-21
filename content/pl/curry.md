+++
title = 'Scala: Currying, Partially Applied, Partial'
+++

# Scala 中的柯里化、偏函数与部分应用函数

柯里化（Currying）、部分应用函数（Partially Applied Function）和偏函数（Partial Function）是 Scala 中三个非常容易混淆的元素。在 Groovy 中，部分应用函数也被称作了柯里化。实际上，二者的作用十分相似，思维上却有微妙的差异。

在传统面向对象方法中涉及到工厂或模板的场合，通常正是柯里化和部分应用函数发挥作用的地方。另一种常见的场合是，在一系列代码中需要为一个函数绑定某一个固定的参数。在传统方式中，我们可能会抽象一个函数出来。但在函数式范式中，我们可以直接将这个"临时函数"绑定在一个变量中。

## 概念上的区别

简单来说，柯里化的结果是"一串函数中的下一个"，而部分应用函数的结果是一个"减少了参数的函数"。对于函数 `f(x, y, z)`，其完全柯里化的结果是 `f(x)(y)(z)`。然后，我们将参数应用进去，例如规定 `x` 的值，得到的结果将是 `g(y)(z)`。反过来，对于部分应用函数，其结果将是 `g(y, z)`。

偏函数之所以会被混淆则是因为名字过于相似。偏函数的"Partial"意味着其只能处理其所接受的参数的一部分。这种情况常见于模式匹配。偏函数可以被连接起来，上一个函数无法处理的参数被传递至下一个函数去处理。

## Groovy & Clojure

``` groovy
def volume = {h, w, l -> h * w * l}
def area = volume.curry(1)
def lengthPartialApplied = volume.curry(1, 1)
def lengthCurried = volume.curry(1).curry(1)
```

Groovy 的这些操作实际上是部分应用而不是柯里化。不过，这两种操作毕竟是十分相似的。而且，Groovy 也允许我们返回一个函数，以进行函数的复合。

``` groovy
def composite = { f, g, x -> return f(g(x)) }
def func = composite.curry(func1, func2)
```

Clojure 中的情况略微类似：

``` clojure
(def substract-from-hundred (partial - 100))
(substract-from-hundred 10)     ; same as (- 100 10)    results 90
(substract-from-hundred 10 20)  ; same as (- 100 10 20) results 70
```

Clojure 原生同样只提供了部分应用。同样地，用它来实现柯里化也很容易。

## Scala

Scala 通过对多个参数列表的支持提供了良好的柯里化功能：

``` scala
def modN(n: Int)(x: Int) = x % n == 0
modN(5)(25)  // true

(1 to 10).toList.filter(modN(5))  // List(5, 10)
```

同时，Scala 也提供对部分应用的支持。针对同样的场景，可以有：

``` scala
def modN(n: Int, x: Int) = x % n == 0
(1 to 10).toList.filter(modN(5, _))  // List(5, 10)
```

Scala 中的偏函数实际上和另外二者关系不大，但名字使其很容易混淆。偏函数通常随着不完全的模式匹配出现。例如，集合的 `map` 和 `collect` 方法：

``` scala
List(1, 2, "a") map {case i: Int => i + 1}
// scala.MatchError: a (of class java.lang.String)
//   at .$anonfun$res17$1(<console>:12)
//   at scala.collection.immutable.List.map(List.scala:286)
//   ... 28 elided

List(1, 2, "a") collect {case i: Int => i + 1}  // List(2, 3)
```

抛出的 `MatchError` 意味着这个模式匹配无法处理 `String` 类型的对象。而在 `collect` 方法中，我们选择了将偏函数无法处理的部分直接抛弃。设计上来说，`map` 接收的参数类型为 `A => B`，`collect` 接收的参数类型为 `PartialFunction[A, B]`。`PartialFunction` Trait 提供了 `isDefinedAt` 方法来判断是否能够处理。Scala 还提供了 `andThen` `orElse` 这样的方法来将偏函数连接起来。

``` scala
def f: PartialFunction[Any, Int] = { case i => 0 }
def func: PartialFunction[Int, Int] = { case i if i > 0 => i + 1 }

List(-1, 1) map (func orElse f)  // List(0, 2)
```

