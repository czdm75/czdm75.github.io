# 11. 断言

断言有两种形式，一种是随时加入的传统断言，另一种是在返回结果之前进行的检查。断言接收第二个 `Any` 类型的参数，调用 `toString` 将结果作为错误信息显示。

```scala
def f(a: Int, b: Int) = {
  assert(a > 0)
  a + b
} ensuring(a < _, "abc")

f(-1, 1)  //java.lang.AssertionError: assertion failed
f(1, -1)  // java.lang.AssertionError: assertion failed: abc
f(1, 1)
```

`ensuring` 的实现大致是这样的：

```scala
@elidable(ASSERTION)
def assert(assertion: Boolean): Unit = { if (!assertion) throw new java.lang.AssertionError("assertion failed") }

@elidable(ASSERTION) @inline
final def assert(assertion: Boolean, message: => Any): Unit = {
  if (!assertion) throw new java.lang.AssertionError("assertion failed: "+ message) }

implicit final class Ensuring[A](private val self: A) extends AnyVal {
  def ensuring(cond: Boolean): A = { assert(cond); self }
  def ensuring(cond: Boolean, msg: => Any): A = { assert(cond, msg); self }
  def ensuring(cond: A => Boolean): A = { assert(cond(self)); self }
  def ensuring(cond: A => Boolean, msg: => Any): A = { assert(cond(self), msg); self }
}
```

也就是说，返回值被隐式转换为了一个 `Ensuring` 对象，这个对象有一个 `ensuring` 方法来接收这些参数，并进一步调用 `assert`。
