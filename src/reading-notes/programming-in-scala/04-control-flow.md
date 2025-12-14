# 4. 控制流

## for

```scala
for (i <- 1 to collection.length - 1) {}
for (i <- 1 until collection.length) {}
for (i <- 1 until 4 if i / 2 = 0) {}

for (i <- 1 to 10 if i % 2 == 0;                 // notice the semicolumn
     j <- 1 to 10 if j % 2 != 0) print((i, j))
for {i <- 1 to 10 if i % 2 == 0
     j <- 1 to 10 if j % 2 != 0} print((i, j))
```

在圆括号内 Scala 不会自动推断分号。for 语句内也可以依赖前一个变量，甚至另外定义变量。

```scala
val col = Seq(Seq(1, 2), Seq(3, 4), Seq(5, 6))
for {i <- col; j <- i} print(j)

for {i <- col; x = i.length; j <- i} print(x)    // 222222
```

## if

```scala
// 两个分支的值是相同的类型，那么返回值是这个类型。对于数值类型，可能自动转换
{ x: Int => if (x == 0) 1 else 1 }    // Int => Int
{ x: Int => if (x == 0) 1 else 1.2 }  // Int => Double
{ x: Int => if (x == 0) 1 else 'a' }  // Int => Int

// 两个分支的值是不同的类型，那么返回值是它们的公共父类。
class Base()
object Sub1 extends Base
object Sub2 extends Base

{ x: Int => if (x == 0) Sub1 else Sub2 }  // Int => Base

// 对于两个没有太多亲属关系的类型，常常得到的是 Any Object(即AnyRef) AnyVal，取决于值的类型是引用还是值。
{ x: Int => if (x == 0) x else false }      // Int => AnyVal
{ x: Int => if (x == 0) "zero" else Sub1 }  // Int => Object
{ x: Int => if (x == 0) "zero" else 1 }     // Int => Any

// 赋值语句的值是 Unit，属于数值类型，情况类似
{ x: Int => if (x == 0) x else { val a = 1 } }       // Int => AnyVal
{ x: Int => if (x == 0) "zero" else { val a = 1 } }  // Int => Any

// throw 语句的类型是 Nothing。与 Unit 不同，不会被算入到寻找公共父类的过程。
{ x: Int => if (x == 0) "zero" else throw new Exception }  // Int => String
```

在求取父类的过程中，需要了解到 Scala 类型的继承结构。虽然 Scala 把数值类型在语法上包装成了对象，但它们不继承于 Object / AnyRef 而是继承于 AnyVal，在编译后会变成原本的基本类型。Nothing 和 null 的情况类似。

## try-catch / try-finally

`throw` 语句有返回值，但其返回值是 `Nothing`，不会被算到求取父类的过程中。`catch` 语句采用模式匹配语法。由于 `try` 和 `finally` 都有返回值，在 `finally` 中返回值会造成一些特殊的结果，最好的方式是绝对避免在 `finally` 语句中返回值，而只用来进行资源释放一类的工作。

```scala
def f(): Int = try return 1 finally return 2  // Int = 2

def f(): Int = try 1 finally 2  // Int = 1
// warning: a pure expression does nothing in statement position; you may be omitting necessary parentheses
- Language//   def f(): Int = try 1 finally 2
//                                ^

def f(): Int = try 1 finally return 2  // Int = 2
```

## 将 break 和 continue 转化为尾递归

在没有 `break` 和 `continue` 的情况下，一个简单的办法是使用布尔值和 `if` 来控制流，不过这不够函数式。但实际上，许多类似的问题都可以被转化为尾递归。例如，假设我们在一系列文件名中寻找第一个不以 `-` 开头的 Scala 源文件的下标：

```java
int i;

for (int i = 0; i < args.length; i++) {
  if (args[i].startsWith("-")) continue;
  if (args[i].endWith(".scala")) {
    i = 1;
    break;
  }
}
```

那么，如果使用尾递归，可以变成：

```scala
def searchFrom(i: Int): Int = {
  if (i >= args.length) -1
  else if (args(i).startsWith("-")) searchFrom(i + 1)
  else if (args(i).endsWith(".scala")) i
  else searchFrom (i + 1)
}

val i = searchFrom(0)
```

在这里，`continue` 语句被替换成了一个递归的以 `i+1` 为参数的调用。而且，由于是尾递归，可以被编译器优化。

当然，标准库里也提供了对 `break` 的扩展语法，但这种方式使用异常捕获来运行。这样做的目的是，即使 `breakable` 出现在另一个函数内，通过异常的抛出，也能实现跨函数的 `break`。

```scala
import scala.util.control.Breaks._

breakable {
  while(true) {
    if (something) break
  }
}
```

## 作用域

Scala 在作用域上与 Java 的主要区别是 Scala 允许在更小的作用域上使用相同的名字来覆盖外面的变量。

```scala
for (i <- 1 to 3) { for (i <- 4 to 6) print(i) }  // 456456456

val a = 1;
{
  val a = 2
  print(a)
}
print(a)
// 21
```

值得顺便一提的是，在 REPL 中之所以可以随意覆盖变量，是因为解释器对每一行输入都划分了一个新的作用域。也就是说：

```scala
scala> val a = 1
scala> val a = 2
scala> print(a)
```

实际上等同于：

```scala
val a = 1;
{
  val a = 2;
  {
    print(a)
  }
}
```
