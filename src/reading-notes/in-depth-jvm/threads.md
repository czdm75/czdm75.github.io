# Java 线程

## Java 中的线程

Windows 和 Linux 提供了一对一的线程模型，即 LWT 形式。同时，现代操作系统使用的都是抢占式的线程调度模型。在 Java 中，调用 `Thread.yield()` 可以让出线程时间，但在抢占式调度下没有什么办法来确定当前线程是否会被抢占，或者阻止抢占。

Java 提供了十个级别的线程优先级（`Thread.MIN_PRIORITY`），但优先级并不是可靠的规定。例如，Windows 下只有七个系统优先级，而且 Windows 包括了基于线程状态改变优先级的机制。

与操作系统类似，Java 的线程有五种互相转换的状态，加上定时等待，一共六种：

- 操作系统提供的线程调度的运行和等待状态都被包括在了 Java 线程的 Running 状态中。
- 无限期等待（Waiting）的可能性包括：
  - 无超时的 `Object.wait()`
  - 无超时的 `Thread.join()`
  - `LockSupport.park()`
- 限期等待（Timed Waiting）的可能性包括：
  - `Thread.sleep()`
  - 有超时的 `Object.wait()` 和 `Thread.join()`
  - `LockSupport.parkNanos()`
  - `LockSupport.parkUntil()`

## 线程安全

### 线程安全的数据

可以把数据 / 变量分为五类，其"线程安全程度"由强至弱排序：

- 不可变 Immutable

    如 final 的基本类型，或 `String`、枚举类、以及 `java.lang.Number` 的一些子类，如 `Double` 和 `BigInteger`。不过，用于自增的 `AtomicInteger` 显然不是不可变的。技术上，只要将所有带有状态的变量全部声明为 final，这个类就是不可变的。

- 绝对线程安全

    绝对线程安全的数据，其所有操作都被封装了同步能力，因此任何外部调用都不需要进行同步措施。这个定义实际上很难达成。例如，从一个集合中取得一个下标，删除这个元素，再次访问。这个过程中的每一个步骤都是线程安全的，但由于取出了一些信息使其脱离了数据结构的维护，整个操作仍然会发生线程安全问题。一些数据结构的 `merge()` 等方法就是为了解决这样的问题，但也只能解决其中常用的一部分。

- 相对线程安全

    与上面的相对应，对于数据结构的每一个单独操作都是线程安全的，这也是我们通常使用的数据结构能够达成的水平，包括已经被舍弃的 `Vector` `HashTable` 和 其他 Concurrent 的集合。

- 线程兼容

    这一类对象本身不是线程安全的，可以通过同步手段来保证线程安全，也就是我们用到的绝大部分集合。

- 线程对立

    完全不兼容线程安全的例子，这种情况很少，`Thread.resume()` 和 `Thread.suspend()` 的共同使用是一个例子。无论是否进行同步，都有可能造成死锁。因此这两个方法已经被声明废弃了。

### 互斥同步

常见的互斥同步方法包括临界区（Critical Section）、互斥量（Mutex）、信号量（Semaphore）等。在 Java 中，最常用的方式就是通过 `synchronized` 关键字。编译时，这个关键字会在修饰的代码块前后加入 `monitorenter` 和 `monitorexit` 指令，分别接收一个引用作为参数来指明操作锁的对象。这个过程是可重入的，对象上的锁包括一个计数器。

由于 Java 上线程的操作是映射到操作系统的，`sychronized` 映射到操作系统的 `mutex`，因此对线程进行阻塞的过程需要通过系统调用来完成，这是一个重量级的操作。

除此之外，也可以使用 `java.util.concurrent.ReentrantLock`，二者的基本功能相似。API 层面的的可重入锁加入了一些高级功能：

- 等待可中断，即通过 `tryLock(long timeout, TimeUnit unit)` 方法实现了可超时的加锁
- 公平锁，即多个线程在等待同一个锁时，获得锁的顺序与申请锁的时间顺序相同。通过 `ReentrantLock(boolean fair)` 构造方法来实现。
- 锁绑定多个条件。一个 `ReentrantLock` 对象可以绑定多个 `Condition` 对象。相比之下，`wait()/notify()` 的模式就需要嵌套多个锁。

在 JAVA 1.6 之后，由于锁优化的完善，synchronized 的性能逐渐追赶上了可重入锁。因此，功能是选择使用锁的更重要的标准。

### 非阻塞同步

互斥同步是一种典型的悲观锁，不能拿到锁的线程需要进行阻塞，这个代价相对较高，需要经过用户态与内核态的转换、维护计数器、检查所有被阻塞的线程等一系列操作。

另一种可能的办法是乐观策略：先进行操作，如果发生了冲突再进行补救措施。随着硬件指令集的发展，"进行操作" 和 "检查是否有冲突" 这两个操作得以原子地完成，乐观策略才能实用。这类指令包括:

- Test - and - Set 测试并设置
- Fetch - and - Increment 获取并增加
- Swap 交换
- Compare and Swap 比较并交换，即 CAS
- Load - Linked / Store - Condition 加载链接 / 条件存储，即 LL / SC

其中，后两条是现代处理器引入的指令，其目的和功能类似。

CAS 有三个操作数，分别是内存位置 V、旧的预期值 A 和新值 B，仅当内存位置的数据与 A 相等时，才使用 B 去替换，并一定返回 V 处的旧值。在现代处理器上，这是一个原子操作，在 JDK 1.5 之后由 `sun.misc.Unsafe` 类中的 `compareAndSwapInt()` 和 `compareAndSwapLong()` 等方法提供，JIT 将方法调用过程省去，编译成一条原子指令。当然，Unsafe 不能正常调用，使用这些方法的例子包括 `java.util.concurrent.atomic.AtomicInteger` 中的 `compareAndSet()` `getAndIncrement()`等。

```java
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
