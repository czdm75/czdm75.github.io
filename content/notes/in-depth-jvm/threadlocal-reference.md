+++
title = 'ThreadLocal and Reference'
+++

# ThreadLocal 与 Java 中的引用

## ThreadLocal 类

`ThreadLocal<T>` 类可以被赋值，并会将自身作为 key，值作为 value 存入 `Thread` 对象中的 `ThreadLocalMap` 中。这样，就能够保证使用这个类包装的变量仅存在于当前线程。而且，即使两个线程分别访问同一个 `ThreadLocal` 对象，从 `get()` 方法获取到的值也都是各自线程中的值。

``` java
void func() {
    tl.set(Thread.currentThread().getName());
    System.out.println(tl.get());
}

ThreadLocal<String> tl = new ThreadLocal<>();
new Thread(() -> {this::func}).start();
// Thread-0
new Thread(() -> {this::func}).start();
// Thread-1
```

### 构造

`ThreadLocal<T>` 类的构造方法没有任何内容和参数。

每个对象有一个 `final int threadLocalHashCode`。这个变量的值由静态构造，来自一个 `AtomicInteger` 累加上一个魔法值。这个魔法值以尽量避免碰撞为依据。这个变量最终会被用在 `ThreadLocalMap` 的 hash 过程中。

### set 方法

`set(T value)` 方法接收要传入的值，使用 `getMap(Thread t)` 方法得到当前所在线程的 `ThreadLocalMap`。如果结果为 null，就调用 `createMap(Thread t, Object value)`。否则，直接将参数值存入到 map 中：`map.set(this, value)`。

``` java
ThreadLocalMap getMap(Thread t) {
    return t.threadLocals;
}

void createMap(Thread t, T firstValue) {
    t.threadLocals = new ThreadLocalMap(this, firstValue);
}
```

### get 方法

`get()` 方法同样首先取得当前线程的 `ThreadLocalMap`。如果 map 为 null 或当前对象并没有事先 `set()` 过，则会将 null 作为初始值存入 map 并返回。

``` java
public T get() {
    Thread t = Thread.currentThread();
    ThreadLocalMap map = getMap(t);
    if (map != null) {
        ThreadLocalMap.Entry e = map.getEntry(this);
        if (e != null) {
            @SuppressWarnings("unchecked")
            T result = (T)e.value;
            return result;
        }
    }
    return setInitialValue();
}

private T setInitialValue() {
    T value = initialValue();
    Thread t = Thread.currentThread();
    ThreadLocalMap map = getMap(t);
    if (map != null)
        map.set(this, value);
    else
        createMap(t, value);
    return value;
}

protected T initialValue() {
    return null;
}
```

### 嵌套类 Entry

`new Thread(() -> {this::func}).start();` 这个类用于存储实际的 Thread Local 变量，继承了 `WeakReference<ThreadLocal<?>>`。`Entry` 在 `WeakReference` 的基础上增加了一个 `value` 域。

这个类以父类 `WeakReference` 中存储的 `ThreadLocal` 对象为 key，以自己定义的 `value` 对象为 value，作为 Map 的 Entry 使用。

``` java
static class Entry extends WeakReference<ThreadLocal<?>> {
    /** The value associated with this ThreadLocal. */
    Object value;

    Entry(ThreadLocal<?> k, Object v) {
        super(k);
        value = v;
    }
}
```

## Java 的四种引用

-   强引用
-   软引用
-   弱引用
-   虚引用

后三种在 `java.lang.ref` 包中都有对应的类。它们的父类 `Reference` 是一个抽象类。由于与这些类有关的很多操作由 JVM 完成，如果需要继承，必须从这三个类继承，而不能直接继承 `Reference`。

### 软引用和弱引用

`java.lang.ref.WeakReference` 和 `java.lang.ref.SoftReference`，二者的功能类似。区别是，软引用的对象只有在内存不足时才会被 GC，而弱引用的对象在失去强引用之后就会直接进入可被 GC 的状态。因此，软引用比弱引用更"强"一些。

二者的典型应用场景类似于如上的 `ThreadLocal`：我们在一个集合内维护着所有元素的列表。当代码中已经不再使用这些元素时，这些元素就能够自动离开集合。

``` java
T objs;
T objw;
SoftReference sref = new SoftReference(objs);
WeakReference wref = new WeakReference(objw);
objs = null;
objw = null;
// full gc
sref.get(); // objs
wref.get(); // null
// null
```

软引用的对象内部有一个时间戳 `private long timestamp`，其值在每次 `get` 时被更新为当前时间。具体来说，软引用类内有一个静态变量 `static private long clock`，其值由垃圾收集器维护。在每次调用 `get` 方法时，如果引用指向的对象不是 null，就对时间戳进行更新。这个值有利于 GC 的优化。

### 虚引用和引用队列

虚引用甚至不能用来访问引用的对象，对其调用 `get` 只能返回 `null`。这个类的存在是为了使用其另外一个功能，这个功能在另外两种引用中同样存在：

``` java
public PhantomReference(T referent, ReferenceQueue<? super T> q) {
    super(referent, q);
}
public WeakReference(T referent, ReferenceQueue<? super T> q) {
    super(referent, q);
}
```

引用队列是一个链表包装类，其结点就是 `Reference` 对象。当引用被 GC 释放时，我们可能希望执行一些其他的操作。

``` java
ReferenceQueue<T> rq = new ReferenceQueue<T>();
T obj = ...;
WeakReference<T> weakReference = new WeakReference<T>(bytes, rq);
map.put(weakReference, value);
obj = null;
// full gc
while((k = (WeakReference) rq.remove()) != null) {
    // do sth.
}
```

`ReferenceQueue` 有三个开放的方法。`poll()` 返回元素或 null，`remove()` 在队列为空时阻塞，`remove(long)` 提供超时时间。

不过，需要注意的是，虚引用并不是非常弱的。
