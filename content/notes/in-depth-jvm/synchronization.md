# Java 中的 synchronized 关键字

对函数使用 `synchronized` 关键字时，如果是静态函数，相当于 `synchronized(this.class)` 代码块。反之，则相当于 `synchronized(this)` 代码块。

`synchronized` 代码块只能锁住对象。最经济的锁对象是 `byte[0]`，只需要三条字节码，比 `new Object()` 更少。锁住一个 static 变量，实际上相当于锁住一个类。锁住的对象必须是 private 的。否则，引用所指向的对象将能够被外部修改，破坏同步。

``` java
public class Synchronize {
    private String name;
    private static final byte[] sb = new byte[0];
    private final byte[] b = new byte[0];
    private Synchronize(String name) {
        this.name = name;
    }
    public static void print(String s) {
        System.out.println("start" + s);
        try {
            Thread.sleep(1000);
        } catch (InterruptedException e) {
        }
        System.out.println("stop" + s);
    }


    public void syncStatic() {//锁住一个静态变量
        synchronized (sb) {
            print(name);
        }
    }
    public void syncMember() {//锁住一个实例变量
        synchronized (b) {
            print(name);
        }
    }
    public static synchronized void syncClass() {//锁住整个类
        print("static");
    }
    public synchronized void syncObject() {//锁住对象
        print(name);
    }
}
```

在字节码的实现中，方法的同步和代码块的同步采用了不同的两种方式：

``` bytecode
  public void syncMember();
    descriptor: ()V
    flags: (0x0001) ACC_PUBLIC
    Code:
      stack=2, locals=3, args_size=1
         5: astore_1
         6: monitorenter                      //!!!获得对象b的monitor锁
         7: aload_0
        14: aload_1
        15: monitorexit                       //!!!释放对象b的monitor锁
        16: goto          24

  public static synchronized void syncClass();
    descriptor: ()V
    flags: (0x0029) ACC_PUBLIC, ACC_STATIC, ACC_SYNCHRONIZED
    //!!!常量池中增加了ACC__SYNCHRONIZED常量，同步工作由JVM完成
    Code:
      stack=1, locals=0, args_size=0
         0: ldc           #6                  // String static
         2: invokestatic  #5                  // Method print:(Ljava/lang/String;)V
         5: return
      LineNumberTable:
        line 21: 0
        line 22: 5
```

# 轻量级锁和偏向锁

`synchronized` 关键字实现的锁通过对象的 monitor 锁调用**操作系统的 mutex 锁**，执行过程中需要在内核态与用户态之间切换，因此这种方式的开销比较大，被称为重量级锁。JDK1.6 之后，引入了轻量级锁和偏向锁。

对象头（Object Header）结构：

  ------------------------------------------------------------------------
   **长度（32bit JVM 中）**          **内容**               **作用**
  -------------------------- ------------------------ --------------------
        1字宽（32bit）              Mark Word               与锁有关

            1字宽             Class Metadata Address   对象所属类型的指针

       1字宽（仅数组）             Array Length             数组长度
  ------------------------------------------------------------------------

对象 Mark Word 的内容：

  ----------------------------------------------------------------------------------------------------------------------
   **锁状态**             **23bit**             **2bit**     **4bit**     **1bit（是否为偏向锁）**   **2bit（锁状态）**
  ------------ ------------------------------- ---------- -------------- -------------------------- --------------------
    GC 标记                  空                    空           空                   空                      11

      无锁          对象hashCode（25bit）       hashCode   对象分代年龄              0                       01

     偏向锁                线程id                Epoch     对象分代年龄              1                       01

    轻量级锁    指向栈中锁记录的指针（30bit）     指针         指针                 指针                     00

    重量级锁      指向互斥量的指针（30bit）       指针         指针                 指针                     10
  ----------------------------------------------------------------------------------------------------------------------

锁可以从低级向高级膨胀，不能反向转化。轻量级锁适用于**两个线程交替执行而不冲突**的情况，此时轻量级锁可以节省性能损耗。当两个线程同时访问锁（两个线程"相撞"时），轻量级锁膨胀为重量级锁，以处理同步过程。此后锁不能再变回来。

偏向锁适合**只有一个线程正在运行**的情况，可以进一步减少同步成本的消耗。偏向锁认为只有自己一个线程正在执行，因此直到竞争出现或者执行完成才释放锁。偏向锁可以膨胀为轻量级锁。

这种处理锁的方式方式被称为乐观锁：发生碰撞后再进行处理。相应的，重量级锁属于悲观锁：无论如何，都要进行同步处理。

# 轻量级锁

## 加锁

1.  在代码进入同步块的时候，如果同步对象锁状态为无锁状态（锁标志位为"01"状态，是否为偏向锁为"0"），虚拟机首先将在当前线程的栈帧中建立一个名为锁记录（Lock Record）的空间。

2.  拷贝对象头中的 Mark Word 复制到锁记录中。

3.  拷贝成功后，虚拟机将使用 CAS 操作（Compare And Swap）尝试将对象的 Mark Word 更新为指向 Lock Record 的指针，并将 Lock record 里的 owner 指针指向对象 Mark Word。

4.  如果这个更新动作成功了，那么这个线程就拥有了该对象的锁，并且对象Mark Word的锁标志位设置为"00"，即表示此对象处于轻量级锁定状态。

    即：此时栈中有一个 Lock Record 段，其由两个部分组成：

    -   owner 指针，指向被锁的对象
    -   从对象拷贝而来的 Mark Word，即内容为hashcode-age-01的段。与此同时，对象头部的 Mark Word 内容为一个 30bit 的指向此位置的指针，及锁标记位00.

5.  如果这个更新操作失败了，虚拟机首先会检查对象的Mark Word是否指向当前线程的栈帧：

    如果是，就说明当前线程已经拥有了这个对象的锁，那就可以直接进入同步块继续执行。

    否则，说明多个线程竞争锁，轻量级锁就要膨胀为重量级锁，锁标志的状态值变为"10"，Mark Word中存储的变成指向重量级锁（互斥量）的指针，后面等待锁的线程也要进入阻塞状态。而当前线程便尝试使用自旋来获取锁。

## 解锁

再次使用 CAS 操作将 Displaced Mark Word 换回，如果失败，则说明锁已经膨胀为重量级锁。

# 偏向锁

JVM 用一个 epoch 表示一个偏向锁的时间戳。

## 获取

当一个线程访问同步块并获取锁时，会在对象头和栈帧中的锁记录里存储锁偏向的线程ID，以后该线程在进入和退出同步块时不需要花费CAS操作来加锁和解锁，而只需简单的测试一下对象头的Mark Word里是否存储着指向当前线程的偏向锁，如果测试成功，表示线程已经获得了锁，如果测试失败，则需要再测试下Mark Word中偏向锁的标识是否设置成1（表示当前是偏向锁），如果没有设置，则使用CAS竞争锁，如果设置了，则尝试使用CAS将对象头的偏向锁指向当前线程。

## 撤销

偏向锁使用了一种等到竞争出现才释放锁的机制，所以当其他线程尝试竞争偏向锁时，持有偏向锁的线程才会释放锁。偏向锁的撤销，需要等待全局安全点（在这个时间点上没有字节码正在执行），它会首先暂停拥有偏向锁的线程，然后检查持有偏向锁的线程是否活着，如果线程不处于活动状态，则将对象头设置成无锁状态，如果线程仍然活着，拥有偏向锁的栈会被执行，遍历偏向对象的锁记录，栈中的锁记录和对象头的Mark Word，要么重新偏向于其他线程，要么恢复到无锁或者标记对象不适合作为偏向锁，最后唤醒暂停的线程。

# 其他优化

## 适应性自旋（Adaptive Spinning）

当线程在获取轻量级锁的过程中执行CAS操作失败时，是要通过自旋来获取重量级锁的。为了减少CPU资源的浪费，指定自旋的次数，如果还没获取到锁就进入阻塞状态。可以使用 JVM 参数 `-XX:PreBlockSpin` 来调整，默认是 10。JDK 1.6 以后使用了适应性自旋，如果自旋成功了，那么这个锁的平均时间可能并不长，使用自选获得锁的几率比较大，更加经济，下次自旋的次数会更多，如果自旋失败了，则自旋的次数就会减少。

## 锁粗化（Lock Coarsening）

将多次连接在一起的加锁、解锁操作合并为一次，将多个连续的锁扩展成一个范围更大的锁，避免在无冲突情况下反复加锁解锁的代价。不过，在代码编写时，还是应当把同步块限制的尽量小。

## 锁消除（Lock Elimination）

锁消除即删除不必要的加锁操作。利用代码逃逸技术，如果判断到一段代码中，堆上的数据不会逃逸出当前线程，那么可以认为这段代码是线程安全的，不必要加锁。对于通常的代码，编写者应当已经明白数据是否进行逃逸。但作为库被使用的代码中可能有许多不必使用的同步块，如已经被废弃的 `StringBuffer`。
