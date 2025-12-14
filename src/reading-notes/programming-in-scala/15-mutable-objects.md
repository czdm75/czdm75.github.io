# 15. Mutable 对象

Mutable 类内部经常使用 `var` 进行定义，但这不是绝对的。例如，一个使用 `var` 引用进行缓存的类可以是纯函数式的，只要其行为对于相同的输入来说是始终相同的：

```scala
class A { def getValue: Int = ... }
class CachedA extends A {
  private var value: Option[Int] = None
  override def getValue: Int = if (value.isDefined) value.get else super.getValue
}
```

Scala 对于类中非私有的 `var` 变量直接提供了 getter 和 setter，这样不仅为未来的修改留出空间，甚至可以让我们自己定义一个虚拟的变量出来。以下的类定义对调用者来说是等价的：

```scala
class A {
  var a = _  // 注意在 Scala 中必须赋一个初值，否则定义的是一个抽象变量
}

class B {
  private[this] var v = 0
  def a: Int = v
  def a_=(x: Int) = v = x
}
```

当我们访问或修改对象中的 `var` 变量时，无论是否显式写出来，实际上都是调用了两个方法。这样，有必要的话我们就可以在 getter 和 setter 中添加逻辑，或者实现一些更复杂的逻辑。例如，定义一个温度类：

```scala
class Therometer {
  var celsius: Float = _
  def fahrenheit = celsius * 9 / 5 + 32
  def fahrenheit_= (f: Float) = celsius = (f - 32) * 5 / 9
  override def toString = fahrenheit + "F/" + celcius + "C"
```
