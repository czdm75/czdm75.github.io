+++
title = 'Spark RDD Programming'
+++

# Spark RDD 编程

## 使用数据集

### 并行化集合

可以使用 Spark Context 的方法对数组进行并行化，以在其上并行地进行操作。

``` scala
val data = Array(1, 2 ,3, 4, 5)
val distData = sc.parallelize(data)
distData.reduce((a, b) => a + b)
sc.parallelize(data, 10)  // 10 partitions
```

进行并行化的一个重要参数就是将集合进行切分的分区（Partition）数量。通常，比较好的数字是每个逻辑核心 2 ~ 4 个分区，Spark 会自动进行划分。如上，也可以手动指定分区的数量。

### 外部数据集：文本文件

Spark 可以为任何被 Hadoop 支持的存储方式上创建分布式的数据集，包括 HDFS，Cassandra，Hbase，Amazon S3，以及本地存储等等。它也支持文本文件或 Hadoop 的各种 InputFormat。

文本文件的数据集可以使用 Spark Context 的 `textFile` 方法来读取。这个方法接收一个文件的 URI，并作为行的集合读取进来。

``` scala
val distFile = sc.textFile("data.txt")
// distFile: org.apache.spark.rdd.RDD[String] = data.txt MapPartitionsRDD[10] at textFile at <console>:26
distFile.map(line => line.length).reduce((a, b) => a + b)
// full length of file
```

-   如果读取本地文件系统的文件，那么集群的所有结点都必须拥有读取这个文件的权限。可以把这个文件拷贝到所有结点的同一位置，或者将文件挂载到网络上来共享。
-   Spark 的所有基于文件的输入方法，包括这里的 `textFile`，都支持目录、通配符和 gz 压缩文件。
-   `textFile` 方法也可以接收一个参数，来控制文本文件的分区数量。Spark 默认为文件的每个 Block 创建一个分区，在 HDFS 上默认是 128 MB。你可以传入一个更大的值使分区的数量更多，但不能少于 block 的数量。

### 其他数据格式

-   `sc.wholeTextFiles` 可以读取一个包含许多小文本文件的目录，将每一个以 `filename, content` 元组返回。相比之下，`textFile` 是以行为单位进行读取。这个方法也提供了第二个参数来控制分区的最少数量。
-   对于 Hadoop 生成的二进制键值对形式的 Sequence 文件，使用 `sc.sequenceFile[K, V]` 来读取。其中，K 和 V 都需要是 Hadoop 的 `Writable` 的子类，例如 `IntWritable` 和 `Text`。Spark 允许使用原生类型来代替几种常见的 `Writable`，例如对于一个 `IntWritable` 和 `Text` 的序列文件，也可以使用 `sequenceFile[Int, String]`，Spark 会进行自动转换。
-   对于其他的 Hadoop InputFormat，可以使用 `sc.hadoopRDD` 方法来读取。它接收一个任意的 `JobConf` 和 InputFormat 类，key 类和 value 类。对于新的 MapReduce API（org.apache.hadoop.mapreduce），使用 `sc.newAPIHadoopRDD`。
-   `rdd.saveAsObjectFile` 和 `sc.objectFile` 可以保存和读取一个 RDD，使用的是简单的序列化 Java 对象。虽然这样不算高效，但足够简单。

## RDD 操作

### Transformations

转换操作将一个数据集转换为另一个数据集，主要包括各种 `map` `filter`，针对键值对的 `reduceByKey` `groupByKey` `sortByKey` `aggregateByKey`，对集合进行转换的 `union` `join`，以及处理分区数量的 `repartition` `coalesce` 等方法。这些操作都会是懒加载的。也就是说，在遇到 Action 之前，Spark 都只记录操作而并不真正计算。

这样的缺点是，对于不同的 action，可能需要重新计算转换过程。因此，可以使用 `persist` 和 `cache` 方法将 RDD 持久化到内存中。当然，也可以持久化到磁盘，或复制到多个节点等。

``` scala
val lines = sc.textFile("data.txt")
val lineLengths = lines.map(s => s.length).persist()
val totalLength = lineLengths.reduce((a, b) => a + b)
```

### Action

Action 通常会将集合转换为一个非集合的值，或者将集合收集回 Driver。包括 `reduce` `collect` `first` `take` `countByKey` 等取得元素或计算得到值的方法，以及各种 `saveAsTextFile` 类似的保存方法。

同时，Spark 也提供了一些方法的异步版本。例如 `foreachAsync` 返回一个 `FutureAction`，而不会阻塞在运算过程中。

### 传递函数

向 `map` 这一类方法传递函数的方式有几种。可以和上面一样使用匿名函数，也可以传递已有的方法，例如单例对象中的方法。

``` scala
object MyFunctions {
    def func1(s: String): String = { ... }
}
rdd.map(MyFunctions.func1)
```

也可以传递对象中的实例方法。这时，这个对象就会被发送到整个集群。

``` scala
class MyClass {
    val field = "Hello"
    def doStuff(rdd: RDD[String]): RDD[String] = rdd.map(x => field + x)
}
```

显然，将这个 `MyClass` 对象发送给整个集群很可能是不太合适的。因此，更好的方法是把这一类变量保存下来。

``` scala
def doStuff(rdd: RDD[String]): RDD[String] = {
    val field_ = this.field
    rdd.map(x => field_ + x)
}
```

这样，就只有 `field_` 需要被发送到整个集群。

### 理解闭包

``` scala
var counter = 0
sc.parallelize(data).foreach(x => counter += x)  // WRONG
```

显然，这样的代码是存在冲突的，其结果是不确定的。在每一个 task 执行之前，Spark 都要计算这个任务的**闭包**（Closure），也就是这个过程所需要访问的外部变量和函数。闭包会被序列化并发送到集群的每一个 executor。

因此，所有的 executor 处理的都是其内部自己的 counter 变量的副本，而不是我们定义的 Driver 里的 counter。当然，在 Local 模式下，如果 executor 和 Driver 使用同一个 JVM，这个值是有可能变化的。但无论如何都不应该使用这种方法。

这种时候，适合使用累加器 `Accumulator`。Spark 会为这个累加器提供安全更新变量的机制。具体做法我们会在后面讨论。

类似的情况还出现在打印 RDD 的元素时。由于同样的原因，直接使用 `foreach` 进行打印，会在集群的每一台节点上分别打印其正在处理的元素，这显然不是我们想要的。正常的方法是使用 `collect` 把 RDD 收集回到 Driver 上再进行打印。如果数据集很大，不能放在一台机器上，更好的方法是使用 `take` 取出一部分来检查。

``` scala
rdd.collect().foreach(println)
rdd.take(100).foreach(println)
```

### 使用键值对

如果 RDD 的泛型类型是一个二元元组（Tuple），那么 RDD 会将其作为键值对来处理。例如，对于简单的 LineCount 程序：

``` scala
sc.textFile("data.txt").map(s => (s, 1))
                       .reduceByKey((a, b) => a + b)
                       .sortByKey().collect()
```

与 Java 一样，由于键值对的操作中需要对元素进行比较，key 必须要提供匹配的 `hashCode` 和 `equals` 方法。

## Shuffle

### Background

Shuffle 是 Spark 重新分配数据的一种方式。这个过程通常需要在结点之间互相交换数据，因此这个过程相对代价较高。

以 `reduceByKey(func)` 为例。这个方法返回一个 RDD，其内容为一些形如 `(key, reducedValue)` 的 Tuple。虽然操作完成后各个分区的内容是确定的，分区之间的顺序也是确定的，但分区内部元素的顺序并不确定。可以使用几种方法：

-   `mapPartitions` 来对每个分区分别进行排序
-   `repartitionAndSortWithinPartitions` 直接完成重分区和排序
-   `sortBy` 来使整个 RDD 有序。

能够触发 Shuffle 的操作包括 `rePartition` `coalesce`，除 `counting` 之外的各种 `ByKey` 操作，以及 `join` 和 `cogroup`。

### 性能影响

Shuffle 操作主要的代价涉及到磁盘 IO、数据序列化、网络 IO 三个部分。这时，Spark 系统会建立一批 map task 和 reduce task。这两个名词来自 MapReduce，和 Spark 的两个方法没有关系。

在系统内部，每个独立的 map task产生的结果都被保存在内存里，直到放不下为止。然后，这些数据会根据目标分区进行排序并写到磁盘上的一个文件。最后，reduce task 读取磁盘，将数据恢复。

一些 Shuffle 方法会消耗大量的堆内存，因为它们需要采用一些数据结构来组织这些数据。例如，`reduceByKey` 和 `aggregateByKey` 会在 map task 过程中创建这些数据结构，而另一些其他 `ByKey` 操作则会在 Reduce 端创建这些数据结构（如 `sortByKey`）。当内存不足时，就会需要大量的磁盘 IO 和 GC 操作。

此外，这个过程还会在磁盘上产生大量的中间文件，有些类似于 Hadoop 的形式。在 Spark 1.3 之后，这些文件会一直保留到 RDD 被 GC，无法再使用为止。这是为了能够方便地计算 RDD 的"血统"（lineage）。那么，如果 RDD 被使用的时间很长，或者 GC 不频繁，这些文件就会占据相当多的磁盘空间。临时存储路径定义在 `SparkContext` 的 `spark.local.dir` 属性。

## RDD 持久化

首先看 `RDD.scala`源码与持久化相关的部分：

``` scala
private def persist(newLevel: StorageLevel, allowOverride: Boolean):       this.type = {
  // TODO: Handle changes of StorageLevel
  if (storageLevel != StorageLevel.NONE &&
      newLevel != storageLevel &&
      !allowOverride) {
    throw new UnsupportedOperationException("...")
  }
  // If this is the first time this RDD is marked for persisting
  // register it
  // with the SparkContext for cleanups and accounting. Do this only once.
  if (storageLevel == StorageLevel.NONE) {
    sc.cleaner.foreach(_.registerRDDForCleanup(this))
    sc.persistRDD(this)
  }
  storageLevel = newLevel
  this
}


def persist(newLevel: StorageLevel): this.type = {
  if (isLocallyCheckpointed) {
    persist(LocalRDDCheckpointData.transformStorageLevel(newLevel),
            allowOverride = true)
  } else {
    persist(newLevel, allowOverride = false)
  }
}

def persist(): this.type = persist(StorageLevel.MEMORY_ONLY)
def cache(): this.type = persist()
```

可以看到，`persist` 和 `cache` 实际上调用的是同一个过程。调用这个方法之后，RDD 并不会被立刻计算，它仍然需要 Action 才能真正触发计算。只不过，计算完成后并不会被清除，而是保留在内存中以备未来使用。

Spark 的缓存是容错的：如果某个分区丢失了，仍然可以再次计算出来。这种情况会出现在内存不足的时候，我们在上面使用 `sc.cleaner.foreach(_.registerRDDForCleanup)` 就是为了允许 Spark 清除这个缓存。Spark 会以 LRU 的方式管理这些缓存。当然，也可以手动调用 `unpersist`。

每个 RDD 都有不同的 Storage Level。默认使用的是 `MEMORY_ONLY`，这意味着缓存只以 Java 对象形式存在于内存中。这意味着，我们认为重新计算数据的代价比磁盘 IO 和反序列化更小（通常并非如此）。

|   Storage Level          |   Meaning                                  |
| ------------------------ | ------------------------------------------ |
| `MEMORY_ONLY`            | 仅内存                                     |
| `MEMORY_AND_DISK`        | 内存和磁盘                                 |
| `MEMORY_ONLY_SER`        | 在内存中以序列化后的 `byte[]` 形式存在     |
| `MEMORY_AND_DISK_SER`    |                                            |
| `DISK_ONLY`              |                                            |
| `MEMORY_ONLY_2` and etc. | 为每个分区在集群上建立两个副本             |
| `OFF_HEAP`               | 仅内存序列化，使用 off-heap 内存（需启用）|

Python 中能够使用的 Level 有所不同。

在 Shuffle 操作中，Spark 会自动使用持久化，即使我们不去主动指定。这样做的目的是，在某个节点运行失败时，不需要计算所有的输入数据。总之，只要我们可能重复使用同一个 RDD，最好就对其进行持久化。

## 共享变量

上面我们已经提到，在 `map` 等方法中使用的变量，需要被复制到整个集群的所有节点中去。并且，这些变量的更新并不会被传回 Driver。为了解决这个问题，Spark 提供了广播变量和累加器两种解决办法。

### 广播变量

广播变量允许我们将一个只读的变量缓存到每一台机器上去。例如以一种比较高效的方式将一份相对稍大的数据集给每一个节点一个副本。Spark 会尝试使用相对高效的算法来降低传输代价。

Spark 的操作会被 Action 和 Shuffle 划分为很多个 Stage。Spark 会自动将每个 Stage 内任务所需要的公共数据进行广播，方式是通过序列化。这意味着，在许多个 Stage 都需要使用同一份数据时，或者使用这些反序列化的数据非常重要时，显式创建广播变量的效果才会比较好。

``` scala
val broadcastVar = sc.broadcast(Array(1, 2, 3))
// broadcastVar: org.apache.spark.broadcast.Broadcast[Array[Int]] = Broadcast(0)
broadcastVar.value
// res0: Array[Int] = Array(1, 2, 3)
```

创建之后，变量 `v` 只需要被传输一次就足够了。另外，为了保证所有结点都能取到相同的值，变量 `v` 在广播后不应该再被修改。

### 累加器 Accumulator

累加器常常用来实现计数器或求和。Spark 原生支持数值型 long 和 double 的累加，而且我们可以实现自己的累加器类型。建立之后，每个任务都可以对累加器进行加操作，但只有 Driver 能够读取值。

累加器甚至会出现在 Web UI 当中（因为其作为计数器的作用）。Tasks 表中将会显示每个任务对累加器的修改，而其最终值将会出现在 Web UI 上面。

``` scala
val accum = sc.longAccumulator("My Accumulator")
// accum: org.apache.spark.util.LongAccumulator = LongAccumulator(id: 0, name: Some(My Accumulator), value: 0)
c.parallelize(Array(1, 2, 3, 4)).foreach(x => accum.add(x))
// 10/09/29 18:41:08 INFO SparkContext: Tasks finished in 0.317106 s
accum.value
// res2: Long = 10
data.map { x => accum.add(x); x }
// now accum is still 0
```

我们可以通过继承 `AccumulatorV2` 类来实现自己的累加器，只需要实现一些方法。

``` scala
class VectorAccumulatorV2 extends AccumulatorV2[MyVector, MyVectot] {
  private val myVector: MyVector = MyVector.createZeroVector

  override def isZero: Boolean = ???
  override def copy(): AccumulatorV2[MyVector, MyVector] = ???
  override def reset(): Unit = ???
  override def add(v: MyVector): Unit = ???
  override def merge(other: AccumulatorV2[MyVector, MyVector]): Unit = ???
  override def value: MyVector = ???
}
```

累加器也允许返回值类型与添加的元素类型不一致。

累加器的更新只发生在 Action 操作中。或者说，对累加器的更新只会在 Action 处被同步到累加器对象。因此，带有累加器的操作仍然是懒加载的。

