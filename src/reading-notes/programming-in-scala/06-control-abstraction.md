# 6. 控制抽象

## 减少代码重复

上一章的内容，有些让人觉得 Scala 中传递函数略显麻烦，但接下来我们就能看到这样做的好处。在 Python 这类动态语言中，我们常常会将一个函数作为参数传入，以便复用代码。作为静态语言，Scala 不能随意把函数直接传入，而是通过上一章的那些隐式转换达到同样的效果。在 Python 中，我们可能会这样做：

```python
def files_maching(query, method):
    for f in get_files():
        if method(f.get_name, query):
            yield f
```

这里的 `method` 可以是字符串的 `contain` `regex_match` 之类的函数。总之，到了 Scala，我们通常会这样做：

```scala
def filesMatching(query: String, matcher: (String, String) => Boolean) = {
  for (file <- files; if matcher(file.getName, query))
    yield file
}

// caller
def filesEnding(query: String) = filesMatching(query, _.endsWith(_))
// '''_.endsWith(_)''' is equivalent with
// '''(fileName: String, query: String) => filename.endsWith(query)'''
```

仔细观察一下的话，我们还可以进一步简化：

```scala
def filesMatching(matcher: String => Boolean) = {
  for (file <- files; if matcher(file)) yield file
}

def filesEnding(query: String) = filesMatching(_.endsWith(query))
def filesContaining(query: String) = filesMathcing(_.contains(query))
```

这类高阶函数在 Scala 类库中的应用，比如 `List` 类型的 `exists` 方法：

```scala
def containsOdd(nums: List[Int]) = nums.exists(_ % 2 == 1)
def containsNeg(nums: List[Int]) = nums.exists(_ < 0)
```

## 柯里化

现在我们可以回头再看看柯里化了。柯里化常被用在与高阶函数相关的场景，使我们自己创建的函数看起来就像语言提供的特性一样。对上面的例子来说，`filesMatching` 函数还能够明显地看出我们编写的影子，`filesEnding` 函数使用起来就非常简单了。也就是说，柯里化通过将部分参数预先定义，让我们的 API 看起来更加简洁。对于 Haskell 这样每一个函数只允许一个参数的语言来说，几乎所有的函数都是柯里化的。

## 编写控制结构 & 传名参数（by-name parameter）

先来考虑一个简单的情况，下面这个结构连续执行一个操作两次：

```scala
def twice(op: Double => Double, x: Double) = op(op(x))
twice(_ + 1, 5)  // returns 7
```

理解了这个例子之后，我们再来考虑一个常见的实际场景： try-with-resources。我们很自然地会写出这样的代码：

```scala
def withPrintWriter(file: File, op: PrintWriter => Unit) = {
  val writer = new PrintWriter(file)
  try {
    op(writer)
  } finally {
    writer.close()
  }
}

withPrintWriter(new File("path/to/file"), writer => writer.println("something"))
```

用我们目前为止得到的思维模式来考虑，将参数拆分成多个参数列表，代码和上面基本相同。单个参数的参数列表和多参数最大的区别是，我们就得以使用大括号来调用：

```scala
def withPrintWriter(file: File)(op: PrintWriter => Unit) = {
  val writer = new PrintWriter(file)
  try {
    op(writer)
  } finally {
    writer.close()
  }
}

withPrintWriter(new File("path/to/file")) { writer =>
  writer.println("something")
}
```

这样的控制结构给了我们一种完全不同的思路。

不过，这样的结构和我们熟悉的 `try-catch` 块值类相比，多了一个参数 `writer`，看起来不太像原生的语言特性。如果我们并不需要在这里传入参数，能不能把参数部分也省略掉呢？考虑一个断言的实现：

```scala
def myAssert(predicate: () => Boolean) =
  if (assertionEnabled && !predicate())
    throw new AssertionException

myAssert(() => a > b)
myAssert(a > b)  // won't compile
```

在这里，显然第二种调用方式更优雅一些，但在这样的实现中没有办法实现。因此，我们使用传名参数：

```scala
def myAssert(predicate: => Boolean) =
  if (assertionEnabled && !predicate)
    throw new AssertionException

myAssert(a > b)
```

当然，实际上还有另外一种实现方式：

```scala
def myAssert(predicate: Boolean) = ...
```

这两种实现方式的区别在于断言条件被计算的时刻。如果直接实现为 `Boolean`，那么无论断言是否开启（`assersionEnabled`），`a > b` 都会被执行。例如：

```scala
var assertionEnabled = false
def myAssert(predicate: => Boolean) = if (assertionEnabled && predicate) throw new AssertionError
def boolAssert(predicate: Boolean) = if (assertionEnabled && predicate) throw new AssertionError
def pred(): Boolean = throw new RuntimeException

myAssert(pred())   // no problem
boolAssert(pred()) // java.lang.RuntimeException
```

在断言关闭的情况下，如果断言条件抛出了异常，那么在使用 `predicate: => Boolean` 作为参数的实现中，`pred()` 将不会被执行，异常不会抛出。
