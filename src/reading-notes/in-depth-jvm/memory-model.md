# Java 内存模型

这里的 Java 内存模型（Java Memory Model, JMM）是 JVM 规范试图提出的一种平台无关的内存模型，目的是既能够达到平台无关的效果，同时能够发挥缓存、多核等硬件模型的性能。

在这个模型中，每个线程都有自己的**工作内存**，与用于存储所有变量的**主内存**区分。二者之间的关系大致相当于 CPU Cache 与主存的关系，工作内存中存储的是主内存中存储的变量的一份拷贝。线程所有的对变量的修改都必须在工作内存中进行。需要注意的是，这里的变量指的是一个引用（Reference），而其指向的对象位于 JVM 堆中，也就是位于主内存区域。

如果和 JVM 内存布局相关联的话，工作内存大致上相当于虚拟机栈的一部分，主内存主要对应于 JVM 堆中的对象实例。

## Java 内存模型的经典描述

- lock（锁定）：作用于主内存的变量，把一个变量标识为一条线程独占状态。
- unlock（解锁）：作用于主内存变量，把一个处于锁定状态的变量释放出来，释放后的变量才可以被其他线程锁定。
- read（读取）：作用于主内存变量，把一个变量值从主内存传输到线程的工作内存中，以便随后的load动作使用
- load（载入）：作用于工作内存的变量，它把read操作从主内存中得到的变量值放入工作内存的变量副本中。
- use（使用）：作用于工作内存的变量，把工作内存中的一个变量值传递给执行引擎，每当虚拟机遇到一个需要使用变量的值的字节码指令时将会执行这个操作。
- assign（赋值）：作用于工作内存的变量，它把一个从执行引擎接收到的值赋值给工作内存的变量，每当虚拟机遇到一个给变量赋值的字节码指令时执行这个操作。
- store（存储）：作用于工作内存的变量，把工作内存中的一个变量的值传送到主内存中，以便随后的write的操作。
- write（写入）：作用于主内存的变量，它把store操作从工作内存中一个变量的值传送到主内存的变量中。

相应地，对这些操作进行了顺序的规定：

- 不允许read和load、store和write操作之一单独出现
- 不允许一个线程丢弃它的最近assign的操作，即变量在工作内存中改变了之后必须同步到主内存中。
- 不允许一个线程无原因地（没有发生过任何assign操作）把数据从工作内存同步回主内存中。
- 一个新的变量只能在主内存中诞生，不允许在工作内存中直接使用一个未被初始化（load或assign）的变量。即就是对一个变量实施use和store操作之前，必须先执行过了assign和load操作。
- 一个变量在同一时刻只允许一条线程对其进行lock操作，lock和unlock必须成对出现
- 如果对一个变量执行lock操作，将会清空工作内存中此变量的值，在执行引擎使用这个变量前需要重新执行load或assign操作初始化变量的值
- 如果一个变量事先没有被lock操作锁定，则不允许对它执行unlock操作；也不允许去unlock一个被其他线程锁定的变量。
- 对一个变量执行unlock操作之前，必须先把此变量同步到主内存中（执行store和write操作）。

## Volatile 规则

JMM 为 `volatile` 提供了一系列特殊的规则。这些规则主要包含两个语义：

首先，任何一条线程对 `volatile` 变量的修改都会被其他线程**立刻得知**。也就是说，变量的值在各个线程之间是一致的。即使实际上某一瞬间是不一致的，在下一次使用之前，线程都会从主内存重新刷新变量的值。

然而，这并不意味着这个变量是线程安全的，因为作用在变量上的操作仍然不是原子的。所以，即使变量值的变化能够立刻反映到其他线程，也可能出现操作的冲突。因此，对于全局计数器这一类变量，使用 `synchronized` 才是正确的选择。`volatile` 变量更适合全局开关的角色：

```java
volatile boolea shutdownRequested;
void shutdown() { shutdownRequested = true; }
void doWork() {
    while (!shutdownRequested) {
        // actual work
    }
}
```

`volatile` 的另一个语义是禁止指令重排。普通的变量只能在所有涉及到赋值的时刻保证变量值修改的正确结果，这在单线程情况下已经足以保证程序正确运行了。然而在多线程情境下，可能会出现更多的问题：

```java
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

```java
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

- 对于 `volatile` 变量 V 和线程 T，只有在上一个操作是 load 时，才能执行 use。又考虑到 load 和 read 必须一起出现，也就是说，需要重新从主内存读取变量的值。
- T 对 V 的 assign 操作和 store 操作必须对于变量 V 连续出现。再考虑到 store 和 write 的关系，也就是说每个线程的修改操作必须要写入到主内存去。
- 当一个线程修改两个变量时，若 use 或 assign 更早，那么对应的 store 或 write 也更早（禁止重排）。

## 例外情况和特性总结

对于 long 和 double 这样的 64 位数据，出于性能考虑，JVM 规范并不要求其原子性。不过在目前的 64 位商业实现中，是能够保证原子性的，尤其是 double，因为 CPU 为其提供了运算单元。JDK 9 中已经包含了与之相关的参数。

JMM 的特征大致可以总结为：

- 原子性：我们大致可以认为 JMM 中所有涉及变量的操作都是原子的。（long 和 double 类型的问题完全可以被忽略）。如果需要更大范围的原子操作，lock 和 unlock 操作也能完成。JVM 通过 `monitorenter` 和 `monitorexit` 字节码来完成这两件事，也就是 Java 代码中的 `synchronized` 关键字。
- 可见性（Visibility）是指在一个线程修改了变量之后，其他线程也能够观察到改变。`volatile` 变量相对于普通变量提供了更加可靠的可见性保证。
- 有序性（Ordering）Java 在线程内是完全有序的，而在跨线程的观察中是无序的。这是由指令重排功能和工作内存的设计决定的。

可以注意到，使用 `synchronized` 加锁是上述这些特性的万能解决方案。

## 先行发生原则

经过上面的讨论，我们最终得到了一系列的**先行发生**（Happens Before）的规则。这些规则可以认为是 JMM 的一个等价的简化描述，可以用于判断线程安全性。

- 程序次序规则 Program Order Rule：一个线程内，按照程序控制流顺序，前面的代码先行发生于后面的代码。
- 管程锁定规则 Monitor Lock Rule：一个 unlock 操作先行发生于时间上后来对同一个锁的 lock 操作。
- volatile 变量规则 Volatile Variable Rule：一个 volatile 变量的写操作先行发生于后面对这个变量的读操作。
- 线程启动规则 Thread Start Rule：Thread 对象的 `start()` 方法先行发生于线程的所有操作。
- 线程终止规则 Thread Terminate Rule：线程的所有操作都先行发生于现成的终止，如 `Thread.join()` 的返回或 `Thread.isAlive()` 的返回假。
- 线程中断规则 Thread Interrupt Rule：对线程 `interrupt()` 方法的调用先行发生于线程内部的代码检测到中断。
- 对象终结规则 Finalize Rule：一个对象的初始化完成先于其 `finalize()` 方法的开始。
- 传递性 Transitivity

这些线性发生规则完全表示了 Java 中所有不需要加锁的情景。考虑一个最简单的情景：线程 A 调用了一个 setter，线程 B 调用了对应的 getter。使用上面的规则进行分析，会发现所有的规则都不适用。因此，即使调用 getter 的操作在时间上晚发生，也无法保证得到的结果是线程 A 刚刚设定的，整个程序线程不安全。解决办法也可以一定程度上从规则中看到：使用 `synchronized` 或 `volatile`，分别对应了第二和第三条规则。
