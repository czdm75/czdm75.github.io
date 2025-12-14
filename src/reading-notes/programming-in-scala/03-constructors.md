# 3. 构造函数

对于 Scala 类：

```scala
class A(i: Int) {
  println(i)
}
```

大致上相当于这样一个 Java 类：

```java
public class A {
  public A(int i) {
    System.out.println(i)
  }
}
```

此外，还可以使用 `require` 对构造函数的参数进行限制，如果不满足则会自动抛出 `IllegalArgumentException`。

```scala
class Rational(n: Int, d: Int) {
  require(d != 0)
  override def toString = n + "/" + d
}

new Rational(1, 0)
// java.lang.IllegalArgumentException: requirement failed
//   at scala.Predef$.require(Predef.scala:212)
//   ... 33 elided
```

这样声明的 `n` 和 `d` 的作用域在类内，相当于两个 `private val`，因此 `toString` 可以访问，但无法从对象外使用 `obj.n` 访问。要在对象外访问，要将其声明为字段。

```scala
class Rational(val n: Int, val d: Int)
```

```scala
class Rational(n: Int, d: Int) {
  val numer = n
  val denom = d
}
```

要创建其他的构造函数，使用 `this` ：

```scala
class Rational(val n: Int, val d: Int) {
  def this(n: Int) = this(n, 1)
}
```

其他构造函数必须首先调用主构造函数，这样就保证了 Scala 中对象的单入口。

最后，对于这样一个有理数类，还缺少一个合适的隐式转换，以便其和一般的整数一起工作：

```scala
implicit def intToRational(x: Int) = new Rational(x)
```

隐式转换遵守作用域的规则。因此，通常需要进行导入。
