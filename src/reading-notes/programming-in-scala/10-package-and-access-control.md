# 10. 包和访问

## 包

可以在一个文件里使用多个包：

```scala
package com
package example
package a {
  class A
}
package b {
  class B(val v: a.A)
  class C(val v: D)
}
class D
```

上面的代码生成了四个类：`com.example.D` `com.example.a.A` `com.example.b.B` `com.example.b.C`。而且，在同一个包里的类可以简略地互相访问。只有嵌套起来的包才可以这样去访问，这是符合直觉的，比如：

```scala
package a { class A }
package a.b { class B(val v: A) }  // Syntax Error, Won't Compile
```

如果不规定的话，最外层的包名为 `__root__`。

```scala
object A  // __root__.A
package a { object A }  // a.A
```

除此之外，还可以定义包对象，通常用于定义一些 Util 函数。这里的函数在整个包都能访问到。

```scala
// in file com/example/package.scala
package object example { def fun = ... }

// another file
import com.example.fun
...
```

## 引入

```scala
import com.example.a              // only the package
import com.example.a.{A, B}       // two objects
import com.example.a.A._          // all in the object A
import com.example.a._            // all objects in package a
import com.example.a.{A => C, B}  // rename A to C
```

```scala
import Laptops._
import Fruits.{Apple => _, _}  // all in the Laptops, and all except Apple in Fruits
```

Scala 环境默认引入了一些变量，后引入的会覆盖先引入的。例如，`scala.StringBuilder` 会覆盖掉 `java.lang.StringBuilder`。

```scala
import java.lang._
import scala._
import Predef._
```

## 访问控制

```scala
class Outer {
  class Inner { private def f = 1 }
  new Inner().f  // works in Java, not in Scala
}

package example {
  class Super { protected def fun = 1 }
  class Another{ def fun = { new Super().fun } }  // works in Java(protected in same package), not in Scala
}
```

Scala 的成员是默认公共的。除了上述两条比 Java 更合理的限制之外，Scala 还提供了更细致的访问控制。

```scala
package com
package example
class Clazz {
  private[com] def fun = 0      // accessible for all in com and subpackages of com
  private[example] def fun = 0  // same as default in java
  private[Clazz] def fun = 2    // same as private in java
  class Inner {
    private[Inner] def fun = 3  // same as private in scala
    private[this] def fun = 4   // only in this object
  }
}
```

最后一个问题是伴生对象。理论上来说，伴生对象和它伴生的类是两个不同的类型（type）。不过显然这两者应该共享访问控制才合适，Scala 也是这么做的。对单例对象来说 `protected` 没有意义。
