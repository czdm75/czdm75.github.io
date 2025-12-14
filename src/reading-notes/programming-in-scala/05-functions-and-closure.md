# 5. 函数和闭包

## 局部函数

通常，对于小的 "工具函数"，我们会使用私有函数来处理：

```scala
object Util {
  def a(): Unit = {
    val a = 1;
    // sth
    b(a)
  }

  private def b(i: Int): Unit {
    // do sth
    print(i)
  }
}
```

通过私有函数，我们避免了 `b` 函数对整个 `Util` 对象的调用者的污染。不过，对于 `Util` 本身的编写者来说，如果 `b` 函数没有别的用处，仍然有些污染视线。因此，可以：

```scala
object Util {
  def a(): Unit = {
    val a = 1;
    // do sth

    def b(): Unit = {
      // do sth
      print(a)
    }

    b()
  }
}
```

与上面的实现的区别是，因为作用域共享，我们无需为 `b` 设定参数，它可以直接访问外面的 `a` 变量。

## 简化函数字面量

```scala
(x: Int) => x+1  // Int => Int

//因为前面的对象是 List[Int], x 的类型被推断为 Int
(1 :: 2 :: Nil).filter(x => x > 1)  // List[Int] = List(2)

// 常见的 PlaceHolder 语法
(1::2::Nil).filter(_ > 1)  // List[Int] = List(2)
val f = (_: Int) + (_: Int)  // (Int, Int) => Int
```

显然，在 `_+_` 这种语法中，每个参数只能出现一次。

## 部分应用函数 Partially Applied Function

下划线也可以用来一次代替多个参数：

```scala
def sum(a: Int, b: Int) = a + b;
val f = sum _
f(1, 2)
```

这时，实际上我们就定义了一个部分应用函数（虽然这里并不"部分"）。总之，部分应用函数意味着你并不提供所有的参数，而是提供几个，或者不提供参数（如上）。上面的 `sum` 函数也可以被这样使用：

```scala
val plus3 = sum(3, _: Int)
plus3(4)
```

进一步，如果我们在不提供任何参数的情况下再去掉下划线，就得到了：

```scala
List(1, 2).foreach(println)
```

这就相当于我们直接把函数 `println` 传入。

不过，这种省略下划线的形式要求 `foreach` 的参数本来就是一个函数类型。考虑这样的情况：

```scala
List(List(1, 2, 3), List(4, 5, 6)).map(_.tail)
List(List(1, 2, 3), List(4, 5, 6)).map(_.drop(1))
List(List(1, 2, 3), List(4, 5, 6)).map(_.drop)
// error: missing argument list for method drop in class List
// Unapplied methods are only converted to functions when a function type is expected.
// You can make this conversion explicit by writing `drop _` or `drop(_)` instead of `drop`.
```

在"纯函数式语言"，如 ML 和 Haskell 中，或者 Python 这样追求简单和统一的语言中，第三种写法通常是有效的。不过，这样做的结果通常只是打印出一串 `<function>`，这是 `drop` 这个函数对象的类型。正因为这个原因，所以有：

```scala
sum
// error: missing argument list for method sum
// Unapplied methods are only converted to functions when a function type is expected.
// You can make this conversion explicit by writing `sum _` or `sum(_,_)` instead of `sum`.
//   sum
//   ^
sum _  // (Int, Int) => Int
```

## 闭包 Closure

```scala
var c = 1;
val f = (x: Int) => x + c
f(1)  // 2

c = 2
f(1)  // 3
```

与 Java 8 不同，Scala 允许函数访问外部的可变的变量，这也带来了一些复杂的问题，例如：

```scala
val more = 3  // won't be used
def f(x: Int): Int => Int = {
  val more = 1
  (y: Int) => y + x + more
}
val fun = f(2)
fun(3)  // 6
```

这个例子有点复杂，其核心内容是，函数 `f` 中的变量 `x` 和 `more` 在调用 `fun` 时都已经离开了作用域，因为 `f` 函数已经返回。好在，Scala 帮助我们在闭包里保留了正确的值。

## 变长参数，命名参数，默认参数

```scala
def print(args: String*) = args.foreach(println)
print("abc", "bcd")
print(Array("abc", "bcd"))
// error: type mismatch;
// found   : Array[String]
// required: String
//   print(Array("abc", "bcd"))
//                  ^
print(Array("abc", "bcd"): _*)
```

```scala
def sum(a: Int, b: Int = 3, c: Int = 4) = a+b+c
sum(1, c = 5)
```

## 尾递归

Scala 会把尾递归优化为一个跳回函数开头的指令。例如，下面两种写法是等价的：

```scala
def find5(num: Int): Int = {
    var n = num
    while (num != 5) n++
}

def find5(num: Int): Int = if num != 5 find5(n+1) else num
```

这样，尾递归在内存和CPU的代价上都和循环一样，却避免了 `var` 的出现。不过，这样做会让 debug 时的堆栈看起来不太一样。可以使用 `-g:notailcalls` 来关闭尾递归。

限于 JVM 的能力，Scala 进行的尾递归比较有限，并没有对间接的尾递归进行优化。例如两个函数交替调用的情况：

```scala
def isEven(x: Int): Boolean = if (x == 0) true else isOdd(x - 1)
def isOdd(x: Int): Boolean = if (x == 0) false else isEven(x - 1)
```

## 柯里化 Currying

```scala
def oldSum(x: Int, y: Int) = x + y
def sum(x: Int)(y: Int) = x + y
def curried(x: Int) = {
  (y: Int) => x + y
}
val f = curried(1)
val plusOne = sum(1) _

f(2)  // 3
plusOne(2)  // 3
```

在 `curried` 函数的定义中，为了视觉上更加清楚，加上了大括号。
