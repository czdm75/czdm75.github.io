# 7. 集合

## 集合

![Java Collection Interfaces](../collection-interfaces.png)

- `Collections.nCopies(n, o)` 返回一个特殊的内部类 `Coolections$CopiesList`，能够作为多个拷贝的 `List` 来使用，但实际只存储一份。
- `Queue` 是一个队列，`Deque` 是双向队列。
- 鼓励在代码中使用接口。如 `List<T> l = new ArrayList<>()`。
- 同样地，在编写有关集合的代码时，尽量使用接口作为参数，以扩大适用范围。

### Collections 的一些静态方法

| **静态方法**                                             | **功能**              |
| -------------------------------------------------------- | --------------------- |
| `boolean disjoint(Collection<?> c1, Collection<?> c2)`   | 判断是否有重复        |
| `void copy(List<? super T> dest, List<? extends T> src)` | 复制                  |
| `boolean replaceAll(List<T> list, T oldVal, T newVal)`   | 替换                  |
| `void fill(List<? super T> list, T obj)`                 | 填充                  |
| `int frequency(Collection<?> c, Object o)`               | 数量                  |
| `int indexOfSubList(List<?> source, List<?> target)`     | 子列表，也有 last方法 |

### 迭代器

- 由 `Iterable<T>` 定义的方法：`Iterator<T> iterator()`，使用 `hasNext()` 和 `next()` 来访问所有元素。也可以直接使用 foreach 循环。
- `remove()` 方法用来移除**刚刚返回的元素**，而不是现在指向的元素。这意味着，两次 `next()` 之间只能调用一次 `remove()`。
- `ListIterator<T>` 作为子接口，加入了 `set` `add` 和 `previous` 方法。
- 许多非线程安全的集合类的迭代器是 fail-fast 的。这意味着如果其他线程改变了集合，将会抛出 `ConcurrentModificationException`。

## 常见的集合

### Set

- `SortedSet` 接口提供顺序访问，`NavigableSet` 接口提供访问邻居元素的方法。`TreeSet` 实现了这两个接口。

- `HashSet` 的性能与元素的 `hashCode()` 相关。显然，碰撞越弱，性能越好。

- `Set` 的元素必须实现 `Comparable<T>`，或者在构造函数提供 `Comparator<T>`。

- `SortedSet` 提供的方法：`first()` `last()` `headSet()` `subSet()` `tailSet`。

- `NavigableSet` 提供的方法：`higher()` `ceiling()` `floor()` `lower()` `pollFirst()` `pollLast()` 等。

### Map

- `TreeSet` 提供顺序访问，但性能更弱。

- 当 Key 不存在时，会返回 null。但对于使用了装箱类的 Map，对 null 的拆箱操作就会引发异常。因此，最好使用 `getOrDefault()` 方法，提供缺省值。

- `merge` 方法可以用来更新 Map 中的计数器：

    ```java
    counts.merge(word, 1, Integer::sum);
    ```

    如果 word 键不存在，就会新建并设为 1。否则，就会加 1。

- `HashTable` 是线程安全的，不接受 null。相比之下 `ConcurrentHashMap` 更加实用。

| 方法                                                           | 功能                                                                                              |
| -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `V putIfAbsent(K key, V value)`                                | 如果不存在，则插入。如果存在，不修改，返回 Map 内的值。                                           |
| `V compute(K key, V value, BiFunction<...> remappingFunction)` | （系列）如果存在 key 的对应 v，就对 v 和传入的 value 进行运算。如果运算结果是 null，就删除 Entry。|
| `V remove(Object key)`                                         | 删除并返回 value。`replace()` 类似。                                                              |
| `boolean remove(Object key, Object value)`                     | 如果 Map 内的 key 和 value 都对应，删除。                                                         |
| `Set<K> keySet()`                                              | 类似的还有 `values()` `entrySet()`                                                                |

### Properties - 一种 Map

```java
Properties setings = new Properties();
settings.put("width", "200");
try (OutputStream out = Files.newOutPutStream(path)) {
    settings.store(out, "ProgramProperties");
}
```

得到：

```properties
##ProgramProperties
##{DateTime}
width=200
```

Properties 文件是 ASCII 编码的。Unicode 字符将会以 escape 形式（`\uxxxx`）存储。

```java
try (InputStream in = Files.newInputStream(path)) {
    settings.laod(in);
}
String title = settings.getProperty("title", "defaultValue");
// don't use get(), for it consumes (Object, Object) rather than String
System.getProperties();  // Some system properties
```

### BitSet

内部实现是一个 `long[]`，因此效率比 `boolean[]` 更高。（Java 规范并没有规定 `boolean` 的大小。）注意这个类并没有实现 `Collection<Integer>` 接口，是一个独立的类。

提供的方法包括某个或某个范围内的 `get` `set` `clear` `flip` 等逻辑操作、`previous` `next` 等。

### 枚举 Set 和 Map

`EnumSet` 包含静态工厂方法：

```java
Set<WeekDay> always = EnumSet.allOf(Weekday.class);
Set<WeekDay> never = EnumSet.noneOf(Weekday.class);
Set<WeekDay> workday = EnumSet.range(Weekday.MON, Weekday.FRI);
Set<WeekDay> three = EnumSet.of(Weekday.MON, Weekday.TUE, Weekday.WED);
```

`EnumMap` 是以枚举类型为 key，任意指定 value 的 Map。

```java
EnumMap<Weekday, String> map = new EnumMap<>(Weekday.class);
map.put(WeekDay.MON, "abc");
```

### 队列，栈，优先级队列

`Stack` 类是历史遗留，不应该被使用。通常，使用 `Queue` 和 `Deque` 已经足够。如果不关心线程安全，可以直接使用 `ArrayDeque`。

优先级队列以任意顺序插入，而只会弹出最小元素。可以用于作业调度，始终弹出优先级最高的任务。使用 `add()` 和 `remove()`。

### WeakHashMap

弱哈希表解决这样一个问题：即使 key 已经不再被使用了，由于 Map 对其的引用，它并不会被 GC 掉。技术上说，`WeakHashMap` 使用 `WeakReference`。总之，当其唯一引用来自 Map 时，就自动删除。

## 视图

集合视图（View）是一个轻量级的集合对象，它可以用来访问元素，但并不储存元素。与 SQL 中的视图非常类似。例如，`keySet` `values` `Arrays.asList` 方法都是这样，其中所有引用的元素都同时被原来的集合引用。这也意味着对这些元素的修改将会体现到原来的集合中。下面是一些常见的视图。

```java
// Ranges
List<String> nextFive = sentence.subList(5, 10)；

// in Navigable interface
NavigableSet<E> headSet(E toElement, boolean inclusive);
NavigableSet<E> subSet(E fromElement, E toElement, boolean inclusive);
NavigableSet<E> tailSet(E fromElement, boolean inclusive);

// other set and map, similar
SortedSet<String> asOnly = words.subSet("a", "b");
//subMap, headMap, tailMap...
```

```java
// empty views and singleton views
Collections.emptyMap();
Collections.singletonMap();
```

```java
// Immutable/readonly views
public class Person {
    public List<Person> getFriends() {
        return Collections.unmodifiableList(friends);
    }
}
```

这样，在视图上进行插入会抛出异常。另外，由于我们之前看到的泛型的不完全安全性，可以让视图来帮我们检查插入对象的类型：

```java
List<String> strings = Collections.checkedList(new ArrayList<>(), String.class);
```

`Collections` 还提供了适合并发的视图，但更加推荐使用 `concurrent` 包中的数据结构，而非这些视图。
