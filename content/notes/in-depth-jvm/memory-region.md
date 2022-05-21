+++
title = 'JVM Memory Regions'
+++

# JVM 内存分区

JVM 规范中定义了五个分区：PC，栈，本地栈，堆和方法区。在这里我们还加入了方法区中的运行时常量池和不属于 JVM 管理的直接内存。

## 程序计数器 PC Program Counter

-   线程私有
-   正常情况下指向运行的字节码，native 方法时为 0
-   JVM 规范中唯一一个未规定 OOM 的内存区域

## 栈 Java VM Stack

大部分 JVM 将栈实现为可扩展的。当栈深度达到限制的最大值时，报 StackOverflowError。当已经没有内存来扩展时，报 OOM。

### 栈帧

一个栈帧代表栈中的一个方法的信息。内容包括：

-   局部变量表
-   操作数
-   动态链接
-   方法出口

其生命周期为一个方法从调用到返回的全过程。

### 局部变量表

局部变量表保存八种基本类型的数据和对象的引用。以 slot 为单位，每个 slot 为 4 Byte。因此 double 类型占用两个 slot。

## 本地方法栈 Native Method Stack

与 Java 栈类似，本地方法栈是 native 方法的栈。JVM 规范中未做强制规定，HotSpot 虚拟机将 Java 栈和本地方法栈合二为一一同进行管理。

## Java 堆 Heap

Java 堆是我们最熟悉的内存区域，这个区域是在各个线程间共享的，将发生 GC 等过程。原则上，所有的对象都会被分配在这里。随着逃逸分析和 JIT 等技术的成熟以及栈上分配和标量替换等技术的广泛应用，有些对象不一定被分配在堆里。

在堆内部，可以根据 GC 算法分为新生代和老生代，而新生代更加细致地划分为 Eden 空间，From Survivor 空间和 To Survivor 空间等。从内存分配的角度来看，可能划分出多个 Thread Local Allocate Buffer（TLAB）等。当然，这些分区并没有实质上的区别，划分的目的是进行不同策略的 GC。

## 方法区 Method Area

各个线程共享的内存区域，用于存储 Class Loader 加载的：

-   类信息
-   常量
-   静态变量
-   JIT 编译后的代码

等内容。也被称为非堆（Non-Heap）。

在 Hotspot 早先的版本，方法区与字符串常量池（Interned Strings）一起被实现为堆的 "永久代"。

## 运行时常量池 Runtime Constant Pool

运行时常量池是方法区的一部分，用来存储 Class 文件中定义的各种字面量和符号引用等。在旧版本中，字符串常量池就是一个典型的例子，说明我们可以在运行时向这个常量池动态地添加对象。

## 直接内存

直接内存，顾名思义就是跳过 JVM 直接在系统内存中进行管理的区域。典型的例子是 Java 1.4 中引入的 NIO 中的 Buffer。通过 native 方法实现，将 Buffer 直接实现在系统中，能够避免在系统内存与 JVM 进程内存之间反复进行拷贝。显然，这个区域不会受到 JVM 参数的影响。当然，当整个系统的内存不足时，同样会发生 OOM。

# HotSpot 的去永久代

## 方法区的变化与永久代

永久代事实上是旧版本 Oracle JDK / OpenJDK 对 JVM 规范中**方法区**的一种实现。在 HotSpot 中，如果永久代发生了内存不足，则会触发 Full GC 以回收常量和加载的类，而 Full GC 通常是我们十分不想见到的。而且，对类进行卸载和回收的条件相当苛刻，再加上字符串常量池的大小，也导致这个位置的 OOM 仍然十分常见。

-   JDK 6 中，方法区的 JIT 编译后代码被储存在 native 内存的 JVM-codeCache 部分，其余均为永久代
-   JDK 7 中，Symbol 的存储移动到了 native 内存，静态变量移动到了 `java.lang.Class` 对象的末尾，位于堆中。
-   JDK 8 中，彻底移除了永久代，将整个方法区转移到元空间 Metaspace。（`-XX:MaxMetaspaceSize`）这个区域位于 Native Heap 中，因此几乎没有容量的限制，除了 JVM 参数。

## 字符串常量 String Interning

### 字符串常量池

JDK 为八种基本类型和 `String` 提供了常量池。其中，字符串常量池在 JDK 7 中被从永久代移除。字符串进入常量池的方式有两种：

-   使用双引号声明字符串
-   使用 `String::intern` 方法

其核心思想是，对于可能多次使用的字符串，将它们的引用指向同一个位于常量池中的字符串，以避免重复分配对象。

在 native 部分，字符串常量池实际上是一个固定长度、使用拉链法的 HashMap。因此，当字符串常量数量超过了 Map 容量之后之后，性能就会开始下降。JDK 7u40 之前，这个值是 1009，之后提高到了 60013，基本上能够满足我们的需求。可以使用 `-XX:StringTableSize` 参数来调整。与一般的 HashMap 一样，容量达到存入数据的两倍时能够比较好地避免碰撞。

### 更多实现细节

在 JDK 6 和 JDK 7 下分别执行同一段代码:

``` java
public static void main(String[] args) {
    String s = new String("1");
    s.intern();
    String s2 = "1";
    System.out.println(s == s2);

    String s3 = new String("1") + new String("1");
    s3.intern();
    String s4 = "11";
    System.out.println(s3 == s4);

    String s5 = new String("1") + new String("1");
    String s6 = "11";
    s5.intern();
    System.out.println(s5 == s6);
}
```

结果分别为 `false false false` 和 `false true false`。在这里，我们就能一定程度上看到 JDK6 和 JDK 7 中实现的区别。

从第一段代码开始。首先，我们知道的是，所有双引号声明的字符串都会在常量池中生成，而 `new` 操作符得到的对象则被分配在堆中。然后，当我们调用 `intern` 方法时，字符串将会被放在常量池中。问题是，当前我们使用的这个引用呢？

实际上，这时我们 `new` 得到的 `String` 对象仍然存在，其位置位于一般的 Java 堆中。只是，其内容指向了常量池，即 JDK 6 中的永久代位置，或后来的堆中的常量池部分。相比之下，我们直接使用双引号声明的 `s2` 则是直接指向常量池中的对象。第三段代码也是同样的情况。

那么，在第二段代码中，JDK 6 和 7 有什么区别呢？因为我们知道，JDK 7 之后。常量池已经被从永久代移到了一般的堆中，这意味着我们完全可以不再存储一个对象，而是直接在常量池中存储一个引用，这个引用和 `s3` 指向同一个对象。也就是说，常量池中存储的是一个指向 Java Heap 中某个对象的引用。

而到了第三段代码，执行顺序的变化也导致了对象引用关系的变化。当然，Java 语言规范并没有规定 `==` 符号在对象上的关系。使用 `equals` 仍然是合适的做法。下面的图分别展示了 JDK 6 和 JDK 7 中的情况

![String Intern](../string-intern.png)

# 对象的创建

## JVM 中对象创建的流程

1.  判断是否需要类加载
2.  分配内存
3.  初始化为全 0
4.  设置对象头
5.  执行 `<init>`

## 分配内存的细节

### 寻找可用内存空间

首先要考虑的是对象是否需要使用连续的内存空间。在 HotSpot 中，对象的空间始终是连续的。因此，当内存碎片和大对象的创建同时出现时，就可能需要很多的 GC 来移动对象腾出整块的空间。在 IBM J9 等 JVM 中可以使用不连续的空间，但相应地会提高性能代价。

当使用带有 Compact 的 GC 器，即已用内存是连续的时，只需要维护一个指向已用与未使用部分分界线的指针并进行移动即可。这种方式叫做 Bump the Pointer。相应地，如果内存不连续，就需要使用 Free List 来寻找。

### 多线程分配和 TLAB

创建对象在虚拟机中是非常频繁的行为，因此线程冲突是十分常见的。有可能一个线程在指针位置创建一个对象时，同时另一个线程又使用了这个指针。解决这个问题的方案有两种，一种是通过 CAS 和失败重试等方式来保证更新操作的原子性，另一种是使用 TLAB（Thread Local Allocation Buffer）。

TLAB 位于 Java 堆的 Eden 区（复制 GC 算法中允许分配新对象的区域）中，为每个线程分配一个。每个线程在创建对象时，只能在自己的 TLAB 中进行分配。这样，就从根源上避免了线程之间的分配冲突。TLAB 并不需要太大，用尽之后还可以继续分配新的区域。当然，分配新的 TLAB 需要进行线程间的同步，避免多个线程分配到同一个区域。这个特性可以通过 `-XX:-UseTLAB` 来设置。

## 对象初始化

首先，JVM 需要将对象的所有位都设置为 0，这也是基本类型的默认值的来源。如果使用 TLAB，这个步骤也可以提前到 TLAB 的分配。

对象头的主要内容包括对象所属的类、类的部分信息、对象的 HashCode、GC 年龄和锁的信息等。

然后，JVM 会调用 `<init>` 方法。这个方法会在任何形式的对象创建时被调用，包括 `new`、反序列化、反射或 `clone()`，其主要作用是按照类编写者的意愿对对象的实例变量进行初始化。类似的是针对静态变量等进行初始化的 `<cinit>` 方法。

具体来说，`<init>` 方法是被字节码中的 `invokespecial` 指令来调用，这个指令也被用在 `super` 关键字和私有方法的调用中。例如，`new StringBuffer` ：

``` bytecode
new java/lang/StringBuffer         ; create a new StringBuffer
dup                                ; make an extra reference to the new instance
                                   ; now call an instance initialization method
invokespecial java/lang/StringBuffer/<init>()V
                                   ; stack now contains an initialized StringBuffer.
```

在 `super` 关键字的应用中，例如：

``` java
public boolean equals(Object x) {
     return super.equals(x);
}
```

则会得到：

``` bytecode
aload_0  ; push 'this' onto the stack
aload_1  ; push the first argument (i.e. x) onto the stack
         ; now invoke Object's equals() method.
invokespecial java/lang/Object/equals(Ljava/lang/Object;)Z
```

# 对象的内存布局和访问

对于 HotSpot 这种使用连续内存分配对象的虚拟机，对象的内存布局比较简单，可以分为对象头、实例数据和 Padding 三部分。

## 对象头

对象头（Object Header）结构：

|  长度（32bit JVM 中）| 内容                   | 作用               |
| -------------------- | ---------------------- | ------------------ |
| 1字宽（32bit）       | Mark Word              | 与锁有关           |
| 1字宽                | Class Metadata Address | 对象所属类型的指针 |
| 1字宽（仅数组）      | Array Length           | 数组长度           |

对象 Mark Word 的内容：

| 锁状态   | 23bit                        | 2bit     | 4bit         | 1bit（是否为偏向锁） | 2bit（锁状态）|
| -------- | ---------------------------- | -------- | ------------ | -------------------- | ------------- |
| GC 标记  | 空                           | 空       | 空           | 空                   | 11            |
| 无锁     | 对象hashCode（25bit）        | hashCode | 对象分代年龄 | 0                    | 01            |
| 偏向锁   | 线程id                       | Epoch    | 对象分代年龄 | 1                    | 01            |
| 轻量级锁 | 指向栈中锁记录的指针（30bit）| 指针     | 指针         | 指针                 | 00            |
| 重量级锁 | 指向互斥量的指针（30bit）    | 指针     | 指针         | 指针                 | 10            |

数组类型多出一个数组长度域的原因是，VM 可以从对象的类型直接推断出对象的长度，却无法推断出数组的长度。此外，也不是所有的虚拟机都使用类型指针来指示类的信息。在 64 bit 的 JVM 下，除了字段的长度，还要考虑是否使用了指针压缩等技术。

## 实例变量和 Padding

接下来是实例变量的存储。通常，虚拟机选择将相同长度的数据（如 long 和 double，short 和 char）存放到一起，以避免内存空间的浪费。在这个前提之下，父类的实例变量会被存储在子类变量之前。如果开启了 `CompactFields`（默认启用）参数，则还可能将较短的变量插入到较长的变量之间的空隙。

总之，实例变量部分的策略和 Padding 部分的存在都是为了进行内存空间的对齐。JVM 中，对象的起始位置是以 8 Byte 为单位进行对齐的。不足的部分就需要 空白的 Padding 来填充。

## 访问对象的方法

JVM 规范中只规定了对象指向引用，而没有规定通过引用寻找对象的具体方法。引用寻址的实现主要有两种：

### 句柄访问

这种方式在 Java 堆中划分了另外一个部分作为句柄池。句柄池中的每一条记录包括指向堆中对象实例的指针和指向方法区中类型数据的指针。这种方法的优势是，在堆中移动对象的实际位置（GC 时十分常见）时，只需要修改句柄中的指针，而不需要回到栈上修改每一个引用的指针。

### 直接指针访问

直接指针访问时，引用直接指向堆中的对象本身，对象内部包含指向方法区中类型数据的指针。这种方式的显著优势就是节省了一次寻址，鉴于对象访问十分频繁，这个优势也是十分可观的。我们前面在对象头中看到的类型数据指针就是这样的实现。

![Memory Object Reference](../mem-obj-ref.png)

# 制造 OOM

## 堆溢出

堆溢出是最容易产生的异常，只要不停地创建"活着"的对象即可。

``` java
// VM Args: -XX:MaxHeapSize=8M / -Xmx8m
// VM Args: -XX:InitialHeapSize=8M / -Xms8m
// VM Args: -XX:MaxNewSize=4M / -Xmn4m
// -XX:+HeapDumpOnOutOfMemoryError
public static void main(String args) {
    List<int[]> list = new ArrayList<>();
    while(true) {
        list.add(new int[100000]);
    }
}
```

通过 Heap Dump，可以查找对象的 GC Root，以排查内存泄漏的原因。否则，就要通过调整参数来避免。

## 栈溢出

在 HotSpot 中并不区分虚拟机栈和本地方法栈。栈部分的溢出有两种形式，`StackOverflow` 和 `OutOfMemory`。对于爆栈，只需要进行大量的递归即可。相关的参数为 `-Xss128k`，等价于 `-XX:ThreadStackSize=128k`。当每个栈帧的本地变量表较大时，爆栈时栈的深度相应就更小。

在单线程情况下，无论栈帧太大还是栈内存不足，抛出的都是 `StackOverflowError`。通常这个深度能够达到 1000 \~ 2000 的级别。OOM 则是出现在创建大量线程的情形，不过这时并不是栈的内存不足，而是无法再分配一个新的栈。

## 方法区溢出

前面我们知道，JDK 6 之前的永久代溢出是常见的，只需要通过大量的 `String.intern`，就能够让字符串常量池产生 OOM。而在 JDK 7 中，常量池转移到了堆中，且只保存一些引用，因此无法再发生这类 OOM。

另一种方法区 OOM 的可能性是加载大量的类。这种情况除了加载 ClassPath 中的类之外，还可能是加载大量的 JSP、反射机制的 `GeneratedConstructorAccessor`、动态代理、OSGi 或 CGLib 操作字节码等方式。无论如何，在 Hibernate、Spring 等框架中，这类情况并不少见。

在 JDK 8 中，随着"去永久代"，与之相关的参数变成了 `-XX:MetaspaceSize` 和 `-XX:MaxMetaspaceSize`。而在之前的版本中是 `-XX:PermSize` 和 `-XX:MaxPermSize`。其中前一个参数都是初始大小。JDK 8 中，默认将 Metaspace 的最大值设为了系统内存，因此不会发生溢出的情况。不过，默认的初始值比较小，扩展的过程会带来一些额外的 GC。

## 直接内存溢出

直接内存溢出只有两种可能，使用 Unsafe 方法或使用 NIO 的 `DirectByteBuffer`。

不过，JDK 的作者为此进行了一定的保护：`DirectByteBuffer` 并没有真的向系统申请内存。它只是发现内存不足以分配，于是手动抛出了异常。另一种方式 Unsafe 方法则要求只有 Bootstrap CLassLoader 加载的类的调用才能返回实例------也就是说只希望 JDK 提供的类来访问。我们可以通过反射来绕过这个规定，当然，并不推荐这样做。

``` java
// VM Args: -Xmx20M -XX:MaxDirectMemorySize=10M
public static void main(String[] args) throws Excecption {
    Field unsafeField = Unsafe.class.getDeclaredFields()[0];
    unsafeField.setAccessible(true);
    Unsafe unsafe = (Unsafe) unsafeField.get(null);
    while(true) {
        unsafe.allocateMemory(1024 * 1024);
    }
}
```

直接内存的 OOM 的特征是，不会在堆 Dump 中发现异常，因此，如果使用了 NIO，就要把这个因素考虑在内。
