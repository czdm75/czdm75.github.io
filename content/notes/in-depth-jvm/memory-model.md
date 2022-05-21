+++
title = 'JVM Memory Model'
+++

# Java 内存模型

这里的 Java 内存模型（Java Memory Model, JMM）是 JVM 规范试图提出的一种平台无关的内存模型，目的是既能够达到平台无关的效果，同时能够发挥缓存、多核等硬件模型的性能。

在这个模型中，每个线程都有自己的**工作内存**，与用于存储所有变量的**主内存**区分。二者之间的关系大致相当于 CPU Cache 与主存的关系，工作内存中存储的是主内存中存储的变量的一份拷贝。线程所有的对变量的修改都必须在工作内存中进行。需要注意的是，这里的变量指的是一个引用（Reference），而其指向的对象位于 JVM 堆中，也就是位于主内存区域。

如果和 JVM 内存布局相关联的话，工作内存大致上相当于虚拟机栈的一部分，主内存主要对应于 JVM 堆中的对象实例。

## Java 内存模型的经典描述

-   lock（锁定）：作用于主内存的变量，把一个变量标识为一条线程独占状态。
-   unlock（解锁）：作用于主内存变量，把一个处于锁定状态的变量释放出来，释放后的变量才可以被其他线程锁定。
-   read（读取）：作用于主内存变量，把一个变量值从主内存传输到线程的工作内存中，以便随后的load动作使用
-   load（载入）：作用于工作内存的变量，它把read操作从主内存中得到的变量值放入工作内存的变量副本中。
-   use（使用）：作用于工作内存的变量，把工作内存中的一个变量值传递给执行引擎，每当虚拟机遇到一个需要使用变量的值的字节码指令时将会执行这个操作。
-   assign（赋值）：作用于工作内存的变量，它把一个从执行引擎接收到的值赋值给工作内存的变量，每当虚拟机遇到一个给变量赋值的字节码指令时执行这个操作。
-   store（存储）：作用于工作内存的变量，把工作内存中的一个变量的值传送到主内存中，以便随后的write的操作。
-   write（写入）：作用于主内存的变量，它把store操作从工作内存中一个变量的值传送到主内存的变量中。

相应地，对这些操作进行了顺序的规定：

-   不允许read和load、store和write操作之一单独出现
-   不允许一个线程丢弃它的最近assign的操作，即变量在工作内存中改变了之后必须同步到主内存中。
-   不允许一个线程无原因地（没有发生过任何assign操作）把数据从工作内存同步回主内存中。
-   一个新的变量只能在主内存中诞生，不允许在工作内存中直接使用一个未被初始化（load或assign）的变量。即就是对一个变量实施use和store操作之前，必须先执行过了assign和load操作。
-   一个变量在同一时刻只允许一条线程对其进行lock操作，lock和unlock必须成对出现
-   如果对一个变量执行lock操作，将会清空工作内存中此变量的值，在执行引擎使用这个变量前需要重新执行load或assign操作初始化变量的值
-   如果一个变量事先没有被lock操作锁定，则不允许对它执行unlock操作；也不允许去unlock一个被其他线程锁定的变量。
-   对一个变量执行unlock操作之前，必须先把此变量同步到主内存中（执行store和write操作）。

## Volatile 规则

JMM 为 `volatile` 提供了一系列特殊的规则。这些规则主要包含两个语义：

首先，任何一条线程对 `volatile` 变量的修改都会被其他线程**立刻得知**。也就是说，变量的值在各个线程之间是一致的。即使实际上某一瞬间是不一致的，在下一次使用之前，线程都会从主内存重新刷新变量的值。

然而，这并不意味着这个变量是线程安全的，因为作用在变量上的操作仍然不是原子的。所以，即使变量值的变化能够立刻反映到其他线程，也可能出现操作的冲突。因此，对于全局计数器这一类变量，使用 `synchronized` 才是正确的选择。`volatile` 变量更适合全局开关的角色：

``` java
volatile boolea shutdownRequested;
void shutdown() { shutdownRequested = true; }
void doWork() {
    while (!shutdownRequested) {
        // actual work
    }
}
```

`volatile` 的另一个语义是禁止指令重排。普通的变量只能在所有涉及到赋值的时刻保证变量值修改的正确结果，这在单线程情况下已经足以保证程序正确运行了。然而在多线程情境下，可能会出现更多的问题：

``` java
volatile boolean initialized = false;
// actual initialization...
initialized = true;

// in another thread
while (!initialized) {
    sleep();
}
doSth();
```

这种情况下，另一个线程的 `doSth()` 需要等待第一个线程中的初始化结束并对其进行使用。如果不使用 `volatile`，第一个线程中的 `initialized = true` 赋值就有可能在初始化尚未结束时就进行。在单线程情况下这并不是问题，因为 JVM 的指令重排会保证接下来使用数据时的正确性。但多线程情况下，我们无法保证线程间指令的顺序，就有可能访问到尚未初始化完成的数据。这个特性直到 JDK 1.5 才正式完成，也是从这时开始，双锁的单例模式得以在 Java 中实现。

``` java
public class Singleton {
    private volatile static Singleton instance;
    public static Singleton getInstance() {
        if (instance == null) {
            synchronized (Singleton.class) {
                if (instance == null) {
                    instance = new Singleton();
                }
            }
        }
        return instance;
    }
}
```

如果我们查看这段代码相关的编译结果，就能够看到，在赋值后执行了一个起**内存屏障**作用的指令 `lock addl $0x0. (%esp)`。这条指令的后半部分把寄存器加 0 没有实际作用，而 `lock` 的作用是将本 CPU 的 Cache 写入内存，同时引起其他 CPU 的 Cache 中的相同位置**无效化**。也就是说，所有的 CPU 需要再次使用这个变量都需要重新从主存中取得。对应到 JMM，这个操作就相当于让所有的线程再次使用这个变量都需要重新 read 和 load。

同时，从指令重排的角度来讲，这个 `lock` 操作强制硬件完成所有这个变量依赖的运算结果。也就是起到了 "内存屏障" 的作用。

可以确定的是，`volatile` 变量的读取操作基本上和普通变量一样，写操作则要略慢。不过，考虑到 JVM 还会为锁进行优化和消除，它不一定会比 `synchronized` 变量快很多。

最后，我们回到 JMM 来看 `volatile` 相关的规则：

-   对于 `volatile` 变量 V 和线程 T，只有在上一个操作是 load 时，才能执行 use。又考虑到 load 和 read 必须一起出现，也就是说，需要重新从主内存读取变量的值。
-   T 对 V 的 assign 操作和 store 操作必须对于变量 V 连续出现。再考虑到 store 和 write 的关系，也就是说每个线程的修改操作必须要写入到主内存去。
-   当一个线程修改两个变量时，若 use 或 assign 更早，那么对应的 store 或 write 也更早（禁止重排）。

## 例外情况和特性总结

对于 long 和 double 这样的 64 位数据，出于性能考虑，JVM 规范并不要求其原子性。不过在目前的 64 位商业实现中，是能够保证原子性的，尤其是 double，因为 CPU 为其提供了运算单元。JDK 9 中已经包含了与之相关的参数。

JMM 的特征大致可以总结为：

-   原子性：我们大致可以认为 JMM 中所有涉及变量的操作都是原子的。（long 和 double 类型的问题完全可以被忽略）。如果需要更大范围的原子操作，lock 和 unlock 操作也能完成。JVM 通过 `monitorenter` 和 `monitorexit` 字节码来完成这两件事，也就是 Java 代码中的 `synchronized` 关键字。
-   可见性（Visibility）是指在一个线程修改了变量之后，其他线程也能够观察到改变。`volatile` 变量相对于普通变量提供了更加可靠的可见性保证。
-   有序性（Ordering）Java 在线程内是完全有序的，而在跨线程的观察中是无序的。这是由指令重排功能和工作内存的设计决定的。

可以注意到，使用 `synchronized` 加锁是上述这些特性的万能解决方案。

## 先行发生原则

经过上面的讨论，我们最终得到了一系列的**先行发生**（Happens Before）的规则。这些规则可以认为是 JMM 的一个等价的简化描述，可以用于判断线程安全性。

-   程序次序规则 Program Order Rule：一个线程内，按照程序控制流顺序，前面的代码先行发生于后面的代码。
-   管程锁定规则 Monitor Lock Rule：一个 unlock 操作先行发生于时间上后来对同一个锁的 lock 操作。
-   volatile 变量规则 Volatile Variable Rule：一个 volatile 变量的写操作先行发生于后面对这个变量的读操作。
-   线程启动规则 Thread Start Rule：Thread 对象的 `start()` 方法先行发生于线程的所有操作。
-   线程终止规则 Thread Terminate Rule：线程的所有操作都先行发生于现成的终止，如 `Thread.join()` 的返回或 `Thread.isAlive()` 的返回假。
-   线程中断规则 Thread Interrupt Rule：对线程 `interrupt()` 方法的调用先行发生于线程内部的代码检测到中断。
-   对象终结规则 Finalize Rule：一个对象的初始化完成先于其 `finalize()` 方法的开始。
-   传递性 Transitivity

这些线性发生规则完全表示了 Java 中所有不需要加锁的情景。考虑一个最简单的情景：线程 A 调用了一个 setter，线程 B 调用了对应的 getter。使用上面的规则进行分析，会发现所有的规则都不适用。因此，即使调用 getter 的操作在时间上晚发生，也无法保证得到的结果是线程 A 刚刚设定的，整个程序线程不安全。解决办法也可以一定程度上从规则中看到：使用 `synchronized` 或 `volatile`，分别对应了第二和第三条规则。

# Java 线程

## Java 中的线程

Windows 和 Linux 提供了一对一的线程模型，即 LWT 形式。同时，现代操作系统使用的都是抢占式的线程调度模型。在 Java 中，调用 `Thread.yield()` 可以让出线程时间，但在抢占式调度下没有什么办法来确定当前线程是否会被抢占，或者阻止抢占。

Java 提供了十个级别的线程优先级（`Thread.MIN_PRIORITY`），但优先级并不是可靠的规定。例如，Windows 下只有七个系统优先级，而且 Windows 包括了基于线程状态改变优先级的机制。

与操作系统类似，Java 的线程有五种互相转换的状态，加上定时等待，一共六种：

-   操作系统提供的线程调度的运行和等待状态都被包括在了 Java 线程的 Running 状态中。
-   无限期等待（Waiting）的可能性包括：
    -   无超时的 `Object.wait()`
    -   无超时的 `Thread.join()`
    -   `LockSupport.park()`
-   限期等待（Timed Waiting）的可能性包括：
    -   `Thread.sleep()`
    -   有超时的 `Object.wait()` 和 `Thread.join()`
    -   `LockSupport.parkNanos()`
    -   `LockSupport.parkUntil()`

## 线程安全

### 线程安全的数据

可以把数据 / 变量分为五类，其"线程安全程度"由强至弱排序：

-   不可变 Immutable

    如 final 的基本类型，或 `String`、枚举类、以及 `java.lang.Number` 的一些子类，如 `Double` 和 `BigInteger`。不过，用于自增的 `AtomicInteger` 显然不是不可变的。技术上，只要将所有带有状态的变量全部声明为 final，这个类就是不可变的。

-   绝对线程安全

    绝对线程安全的数据，其所有操作都被封装了同步能力，因此任何外部调用都不需要进行同步措施。这个定义实际上很难达成。例如，从一个集合中取得一个下标，删除这个元素，再次访问。这个过程中的每一个步骤都是线程安全的，但由于取出了一些信息使其脱离了数据结构的维护，整个操作仍然会发生线程安全问题。一些数据结构的 `merge()` 等方法就是为了解决这样的问题，但也只能解决其中常用的一部分。

-   相对线程安全

    与上面的相对应，对于数据结构的每一个单独操作都是线程安全的，这也是我们通常使用的数据结构能够达成的水平，包括已经被舍弃的 `Vector` `HashTable` 和 其他 Concurrent 的集合。

-   线程兼容

    这一类对象本身不是线程安全的，可以通过同步手段来保证线程安全，也就是我们用到的绝大部分集合。

-   线程对立

    完全不兼容线程安全的例子，这种情况很少，`Thread.resume()` 和 `Thread.suspend()` 的共同使用是一个例子。无论是否进行同步，都有可能造成死锁。因此这两个方法已经被声明废弃了。

### 互斥同步

常见的互斥同步方法包括临界区（Critical Section）、互斥量（Mutex）、信号量（Semaphore）等。在 Java 中，最常用的方式就是通过 `synchronized` 关键字。编译时，这个关键字会在修饰的代码块前后加入 `monitorenter` 和 `monitorexit` 指令，分别接收一个引用作为参数来指明操作锁的对象。这个过程是可重入的，对象上的锁包括一个计数器。

由于 Java 上线程的操作是映射到操作系统的，`sychronized` 映射到操作系统的 `mutex`，因此对线程进行阻塞的过程需要通过系统调用来完成，这是一个重量级的操作。

除此之外，也可以使用 `java.util.concurrent.ReentrantLock`，二者的基本功能相似。API 层面的的可重入锁加入了一些高级功能：

-   等待可中断，即通过 `tryLock(long timeout, TimeUnit unit)` 方法实现了可超时的加锁
-   公平锁，即多个线程在等待同一个锁时，获得锁的顺序与申请锁的时间顺序相同。通过 `ReentrantLock(boolean fair)` 构造方法来实现。
-   锁绑定多个条件。一个 `ReentrantLock` 对象可以绑定多个 `Condition` 对象。相比之下，`wait()/notify()` 的模式就需要嵌套多个锁。

在 JAVA 1.6 之后，由于锁优化的完善，synchronized 的性能逐渐追赶上了可重入锁。因此，功能是选择使用锁的更重要的标准。

### 非阻塞同步

互斥同步是一种典型的悲观锁，不能拿到锁的线程需要进行阻塞，这个代价相对较高，需要经过用户态与内核态的转换、维护计数器、检查所有被阻塞的线程等一系列操作。

另一种可能的办法是乐观策略：先进行操作，如果发生了冲突再进行补救措施。随着硬件指令集的发展，"进行操作" 和 "检查是否有冲突" 这两个操作得以原子地完成，乐观策略才能实用。这类指令包括:

-   Test - and - Set 测试并设置
-   Fetch - and - Increment 获取并增加
-   Swap 交换
-   Compare and Swap 比较并交换，即 CAS
-   Load - Linked / Store - Condition 加载链接 / 条件存储，即 LL / SC

其中，后两条是现代处理器引入的指令，其目的和功能类似。

CAS 有三个操作数，分别是内存位置 V、旧的预期值 A 和新值 B，仅当内存位置的数据与 A 相等时，才使用 B 去替换，并一定返回 V 处的旧值。在现代处理器上，这是一个原子操作，在 JDK 1.5 之后由 `sun.misc.Unsafe` 类中的 `compareAndSwapInt()` 和 `compareAndSwapLong()` 等方法提供，JIT 将方法调用过程省去，编译成一条原子指令。当然，Unsafe 不能正常调用，使用这些方法的例子包括 `java.util.concurrent.atomic.AtomicInteger` 中的 `compareAndSet()` `getAndIncrement()`等。

``` java
public final int incrementAndGet() {
  for(;;) {
    int current = get();
    int next = current + 1;
    if (compareAndSet(current, next)) {
      return next;
    }
  }
}
```

不过，CAS 存在 ABA 问题：如果一个变量曾经被改成 C，又改回 A，CAS 就会认为变量从未改变过。当然，这种情况通常不会影响同步。虽然 Java 提供了 `AtomicStampedReference` 来解决这个问题，如果真的需要，直接进行互斥同步通常就可以接受。

### 无同步方案

同步只是实现线程安全的一种方式，线程安全不一定必须进行同步。

**可重入代码**是其中一种天然线程安全的代码，可以在其执行的过程中任意插入其他代码甚至递归。典型的可重入代码是，完全不使用堆空间，所有状态量都由参数传入，即一个无状态的代码块。在函数式编程中这种代码很常见。

使用 ThreadLocal 是另一种方式，通过将变量限制在线程内部，完全避免了线程冲突。在生产者-消费者模式中，通常会将一个资源在一个线程内消费完成。
