# 8. Stream

## 流

### 创建流

- 流不直接存储数据。它按需生成元素，或者直接引用自集合。
- 流不改变源数据。例如，`filter` 不会删除集合中的元素。
- 流可以是延迟执行的，因此也可以是无限的。例如，我们只从流中取出 5 个元素，那么就只会对 5 个元素执行运算。
- 由于以上的原因，只要流还没有完全使用结束，就不能修改原集合。否则，行为是未定义的。

大多数集合可以使用 `Collection` 接口的 `Stream()`。`parallelStream()` 生成一个并行流。

对于数组，使用静态方法 `Stream.of()`。这里实际上是一个可变参数，因此把元素一个个传入也是可行的。

`Arrays.stream(arrya, from, to)` 为数组的一部分生成流。`Stream.empty()` 返回一个空流。这里会自动进行泛型推断，也可以为方法指定泛型的类型变量 `Stream.<String>empty()`。

要创建无限流，可以使用 `generate` 或 `iterate` 方法。

```java
Stream<String> echos = Stream.generate(() -> "Echo");  // a Supplier<T>
Stream<Double> ramdoms = Stream.generate(Math::random);

Stream<BigInteger> integers = Stream.iterate(
    BigInteger.ZERO, n -> n.add(BigInteger.ONE));
// a seed and an UnaryOperator<T> to apply on it
```

除此之外还有很多其他地方的生成流的方法。例如，用于正则表达式的 `Pattern` 类有一个方法：`splitToStream` 用来使用正则表达式分割字符串成流。

### 流转换

对于包含流的流，使用 `flatMap` 将其展开。`limit(n)` 用来返回一个包含一定数量元素的子流。`skip(n)` 与其相反，是跳过前 n 个元素。例如，生成 100 个随机数：

```java
Stream<Double> randoms = Stream.generate(Math::random).limit(100);
```

`Stream.concat` 将两个流连接起来。当然，这时第一个流不能是无限的。

`sorted` 用来对流进行排序。可以使用 `Comparator`，也可以接收 `Comparable` 对象。

最后，`peek` 方法是一个 "透明的" 流，不改变流的内容，适合用来打印调试信息甚至打断点：

```java
intStream.peek(System.out::println).limit(20).toArray();
```

这样，将会打印 20 个随机数，并生成相应的数组。这里也可以看出之前说的延迟执行特性。

## Optional

### 创建 Optional

```java
Optional.empty();
Optional.of(obj);
Optional.ofNullable(obj);  // empty if null
```

### 使用 Optional

```java
String s = opt.get();  // if empty, throw NoSuchElementException
String s = opt.orElse("");  // default value
String s = opt.orElseGet(() -> System.getProperty("user.dir"));  // calculate default
String s = opt.orElseThrow(IllegalStateException::new);  // throw an exception
// use value in optional, or do nothing
opt.ifPresent(results::add);  // return void

// or check the return value
Optional<Boolean> added = opt.map(results::add);
//now added may be true or false(return of add) or empty optional(if opt is empty)
```

```java
public static Optional<T> f() {
    ...
}
//in the T class
Optional<U> g() {
    ...
}
Optional<U> result = s.f().flatMap(T::g);
// s.f() returns a Optional<T>, flatMap to T , then T::g
// overall ,if s.f() presents, call g. otherwise, return empty Optinal<U>
```

显然，这种方式可以链式地调用，任何一步返回 empty 都会终止。

## 收集和处理流

### 收集到集合内

```java
stream.toArray();
stream.collect(Collectors.toList());
stream.collect(Collectors.toSet());
stream.collect(Collectors.toCollection(TreeSet::new));

// collect to string
String result = stream.collect(Collectors.joining());
String result = stream.collect(Collectors.joining("\t"));
String result = stream.map(Object::toString).collect(Collectors.joining());

// same with long and double, sum and min and such
IntSummaryStatics summary = stream.collect(Collectors.summarizingInt(String::length));
double averageLength = summary.getAverage();
double maxLength = summary.getMax();
```

```java
// collect to Map
Map<Integer, String> idToName = people.collect(
    Collectors.toMap(Person::getId, Person::getName));
// or
Map<Integer, Person> idToPerson = people.collect(
    Collectors.toMap(Person::getId, Function.identity()));
// thows IllegalStateException if has same ids, or
Collectors.toMap(Person::getId, Function.identity,
                 (exsitingVal, newVal) -> existingVal);  //to keep the old one
// if you want to specify the map, since it is the 4th param, must write the 3rd
Collectors.toMap(Person::getId, Function.identity,
                 (exsitingVal, newVal) -> existingVal, TreeMap::new);
```

另外还有相应的 `toConcurrentMap` 方法。

### 归约操作 reduce

之前见到的`count` 方法就是一个简单的归约操作。类似地，`max` `min` 返回流中的最大值和最小值，返回一个 `Optional<T>` 对象。当流为空的时候，就不会返回 null。除此之外，还有 `findAny` 匹配，以及经常和 `filter` 一起使用的 `findFirst`。

如果需要知道流中是否含有匹配元素，使用 `anyMatch` 方法接受一个 `predicate<T>`，返回 `boolean`。类似的还有 `noneMatch`。

除此之外，还常常使用 `forEach`，或者在并行流上可能需要 `forEachOrdered`。这时是为了使用代码的 "副作用"。

此外，Java 提供了强大的 `reduce` 方法。简单地说，它接收一个二元操作，并对所有元素连续进行这个操作。操作必须满足结合律。这类操作包括最大值最小值、加法、拼接等。

```java
// sum of all elements, empty if stream is empty
Optional<Integer> sum = values.stream().reduce((x, y) -> x + y);
Optional<Integer> sum = values.stream().reduce(Integer::sum);
// or offer an "start point", 0 for add, 1 for multiply, etc. act as default value.
int sum = values.stream().reduce(0, Integer::sum);
```

这里，0充当的是运算的起点，即单位元的作用。对于累加是 0，对于累乘就应当是 1。不过，`reduce` 只能接收一个 `(T, T) -> T` 类型的方法，即二元操作。如果要进行更复杂的操作：

```java
Optional<Integer> result = words.reduce((total, word) -> total + word.length(),
                                        (total1 ,total2) -> total1 + total2);
```

这是说，我们需要进行两类累加操作，并分别提供函数。当然，这种情况下将其 map 到一个 `IntStream` 再处理要简单得多。

Java 中提供了三种基本类型流 `IntStream` `LongStream` `DoubleStream`，它们的 `toArray` 得到的是基本类型数组，具有 `max` `sum` `average` 等方法，并可能返回 `OptinalInt` 等类型。

### 分组，分片，下游收集器

```java
Map<String, List<Locale>> contryToLocales = locales.collect(
    Collectors.groupingBy(Locale::getCountry));  // group by country
// while group by boolean, faster method
Map<Boolean, List<Locale>> englishAndOthers = locales.collect(
    Collectors.partitioningBy(l -> l.getLanguage().equals("en")));
// concurrent map
Collectors.groupingByConcurrent(Locale::getCountry);
```

上面都是默认使用了 `List` 作为 Map 的 value。如果要使用其他的，比如 Set，可以提供一个下游收集器（downstream collector）。

```java
Map<String, List<Locale>> contryToLocales = locales.collect(
    Collectors.groupingBy(Locale::getCountry, Collectors.toSet()));
```

类似这里 `toSet` 用法的还包括：`counting` `summing` `maxBy(Comparator)` `minBy(Comparator)`，以及比较复杂的 `mapping`。也有相应的 `groupingByConcurrent` 等。

```java
import java.util.stream.Collectors.*
Map<String, Set<String>> counttryToLangs = locales.collect(
    groupingBy(Locale::getCountry, mapping(Locale::getDisplayLanguage, toSet())));
```

如果返回的值是 `int` `long` `double`，就可以用上面出现的 `summarizingInt` 系列方法来替代 `toSet` 进行统计。

当然，下游收集器这种方法只适合在使用 `groupingBy` 和 `partitioningBy` 时使用，否则只需要直接对流使用归约 `max` `count` `reduce` 等即可。

## 并行流

获得并行流：

```java
words.parallelStream();
Stream.of(wordArr).parallel();
```

显然，并行流中的操作不应该使用共享的内容。也就是说，所有操作都应当是无状态的，可以以任意顺序执行。例如，对字符串中的单词长度进行计数：

```java
// WRONG way
int[] shortWords = new in[12];
words.parallelStream().foreach(s -> {
    if (s.length() < 12) {
        shortWords[s.length()]++;  // competing condition
    }
});
//Right Way
Map<Integer, Long> shortWordCounts = words.parallelStream()
    .filter(s -> s.length < 12)
    .collect(Collectors.groupingBy(String::length, counting()));
```

默认，来自有序集合、range、生成器、迭代器和 `sorted` 得到的流都是有序的。有序不影响并行。但如果顺序不重要，可以使用 `unordered` 来提高性能。例如使用 `limit` 来取出几个元素但并不在乎是哪几个时。又比如，使用 `distinct` 来让流所有元素保持唯一时。

又比如使用上面的 `groupingBy` 操作时，代价相当高。但如果使用 `groupingByConcurrent`，虽然使元素成为无序的，但可以进行并行操作，提高性能。如果你不是用与顺序有关的下游收集器，就无需考虑顺序问题。
