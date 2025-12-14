# 23. 注解

注解实际上是调用了注解类的构造方法，所以能够支持构造方法所能支持的默认参数、带名参数、变长参数等，也能支持表达式，这点比 Java 要强。不过，如果要将一个注解传到其他地方，由于注解不是表达式，所以不能使用 `@`，而需要使用 `new`。常用的 Scala 提供的注解包括 `@Serializable` `@transient` `SerialVersionUID` `scala.reflect.BeanProperty`（生成getter setter）`tailrec` `unchecked`（关闭模式匹配检查）`native`（标记本地方法）`volatile` 等等。

Scala 不使用 Checked Exception（实际上这个检查也只在 javac 中进行，而不在 java 运行时中进行，所以并没有关系）。如果要让对接的 Java 代码看到 Checked Exception，也是用注解 `@throws(classOf[IOException])`。遗憾的是，Java 的注解必须在 Java 中编写，这考虑到了两种注解功能上的区别，也考虑到可能的 Scala 自己的反射的存在。
