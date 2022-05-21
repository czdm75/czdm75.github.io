+++
title = 'Hadoop Basic Concepts'
+++

# Hadoop 基本概念

## HDFS

### 功能特性

向各种计算框架开放文件读写端口，适合大文件、一次写入多次读取的存储。

仅支持单线程 append 写，不支持随机多线程访问。

### 结构

#### HDFS Client

-   与 NameNode 交互，获取文件 Block 所在结点等信息

-   切分文件为 Block

-   与 DataNode 交互，实际传输文件

-   其他管理 HDFS 的工作

#### NameNode

-   作为集群的 Master，管理名称空间

-   管理 Block 的映射信息

-   配置副本策略

-   处理 Client 的读写请求

NameNode 是整个 HDFS 最重要的部分，在 Hadoop 2.0 之后和 YARN 一起引入了 HA 架构，使得 Hadoop 可以用于在线应用。

#### DataNode

-   执行 NameNode 下达的操作

-   存储数据

-   执行读写

#### SecondaryNameNode

-   定期将 `edits` 和 `fsimage` 合并

-   起到辅助恢复的作用，但并不是热备

### SecondaryNameNode 的作用

SecondaryNameNode 能够保存 HDFS 的变化信息，在集群故障时辅助进行恢复工作。同时，也为 NameNode 分担了一小部分工作。其工作流程是基于变化的：

1.  NameNode 维护两部分信息：`edits` 和 `fsimage`。一切新的修改都执行在 `edits` 上。因此，访问文件也是优先访问 `edits` 部分。

2.  当到达一定时间或 `edits` 达到一定大小（默认为 3600s 和 64MB）时，将二者共同传送给 SecondaryNameNode。然后，建立一个新的 `edits.new`，存储接下来文件系统的变化。

3.  SecondaryNameNode 接收到 `edits` 和 `fsimage` 后，将二者合并，成为一个新的文件系统快照。这个快照在集群故障时就可作为恢复依据。将合并后的新的文件快照 `fsimage.ckpt` 发回给 NameNode。

4.  NameNode 收到合并后的 `fsimage.ckpt` 后，将其作为新的 `fsimage` 使用，而在这段时间内存储文件系统变化的 `edits.new` 既可作为新的 `edits` 使用。

通过这样的流程，SecondaryNameNode 将合并文件系统变化的工作接了过来，NameNode 只需要负责及时响应客户端的变化即可。

### HDFS 的读写流程

在读取文件时，客户端获取到 DFS 实例后，DFS 实例从 NameNode 通过 RPC 获取到一批 block 的位置，但并不一定是全部 block。这些 block 按照其所在的 DataNode 距离客户端从近到远的顺序排序。Client 依次串行地读取每一个 block，直到全部读完，再从 NameNode 获取下一批 block 的位置。这样，避免了文件较大时读取队列过长的情况。

在写入文件时，调用 DFS 实例的 `create` 方法，要求 NameNode 创建一个大小为 0 的文件。也就是说，HDFS 写入新文件的方式是先 touch 再 append。在创建时，NameNode 会检查是否有同名文件、目录权限等问题。

随后，Client 的 `DFSOutputStream` 将数据切分成 block 进入 `data queue` 队列，并向 NameNode 请求 3 个最合适的 DataNode（取决于副本策略）。此外，`DFSOutputStream` 还包含一个 `ack queue` 队列，用于等待 DataNode 的写入完成回复。只有在收到了来自 DataNode 的确认信息之后才会将 block 从 `data queue` 中移除。写入全部完成后，关闭流，通知元数据节点。

### HDFS 的副本策略

决定副本策略的权衡因素包括集群的可靠性以及读写带宽。默认副本因子为 3，要求同机架（rack）两份，其他机架一份。在 Client 需要读取文件时，选择距离最近的结点进行读取。

在 Hadoop 的安全模式下，会对所有的文件检查副本数量。

## MapReduce

适用于离线处理的，高容错（在一个节点上失败将会移到其他节点）的分布式计算框架

### 代码示例

``` java
public class WordCount {

  public static class TokenizerMapper extends Mapper<Object, Text, Text, IntWritable> {
    private final static IntWritable one = new IntWritable(1);
    private Text word = new Text();

    public void map(Object key, Text value, Context context
                    ) throws IOException, InterruptedException {
      StringTokenizer itr = new StringTokenizer(value.toString());
      while (itr.hasMoreTokens()) {
        word.set(itr.nextToken());
        context.write(word, one);
      }
    }
  }

  public static class IntSumReducer extends Reducer<Text,IntWritable,Text,IntWritable> {
    private IntWritable result = new IntWritable();

    public void reduce(Text key, Iterable<IntWritable> values,
                       Context context
                       ) throws IOException, InterruptedException {
      int sum = 0;
      for (IntWritable val : values) {
        sum += val.get();
      }
      result.set(sum);
      context.write(key, result);
    }
  }

  public static void main(String[] args) throws Exception {
    Configuration conf = new Configuration();
    Job job = Job.getInstance(conf, "word count");
    job.setJarByClass(WordCount.class);
    job.setMapperClass(TokenizerMapper.class);
    job.setCombinerClass(IntSumReducer.class);
    job.setReducerClass(IntSumReducer.class);
    job.setOutputKeyClass(Text.class);
    job.setOutputValueClass(IntWritable.class);
    FileInputFormat.addInputPath(job, new Path(args[0]));
    FileOutputFormat.setOutputPath(job, new Path(args[1]));
    System.exit(job.waitForCompletion(true) ? 0 : 1);
  }
}
```

### MR 内部逻辑

#### Map 阶段

Map 阶段由 InputFormat、Mapper 和 Partitioner 三部分构成。

输入的每个 Split 交给一个 Mapper 进行处理。Split 与 HDFS 中 Block 的对应关系取决于 InputFormat 和压缩格式，通常为一对一。默认使用的是 `TextInputFormat`，其传递给 Mapper 的格式中，Key 为行的偏移量，Value 为行内容。

经过用户编写的 Map 逻辑后，数据根据 Key 由 Partitioner 分组选择不同的 Reduce 节点。默认方法是进行 Hash 取模，模数即为 Reduce 节点的数量。因此，相同的 Key 将会被分配到同一个 Reduce 节点。

在 Partitioner 之前，可以使用 Combiner 将数据进行合并。例如，在 wordcount 程序中，可以先使用 Combiner 将相同 Key 的数据计算出来。那么，Partitioner 输出的数据就将不再形如 `<k1,1> <k1,1> <k2,1>`，而是经过 Combiner 计算变成 `<k1,2> <k2,1>`。相当于进行了局部的 Reduce 操作。因此，这个参数在很多时候（比如上面的例子）可以直接使用 Reducer，即使不传入也不会对结果有影响。

#### Reduce 阶段

Reduce 阶段由 Shuffle、Sort、Reducer、OutputFormat 四个部分构成。

在 Shuffle 部分，数据从 Mapper 拷贝到 Reduce 节点，具体拷贝哪一个由前面的 Partitioner 决定。拷贝之后，在 Reduce 节点本地进行 Sort，将相同的 Key 排列到一起以供 Reduce 使用。

Reducer 将数据处理完成后，根据 OutputFormat 输出到磁盘。默认为 `TextOutputFormat`。

#### Shuffle 阶段的具体细节

在 Mapper 处理完成后，数据并不是直接进入磁盘，而是先储存在内存中的 buffer 中。buffer 是一个环形缓冲区，其大小由 `io.sort.mb` 属性控制，阈值由 `io.sort.spill.percent` 控制，默认为 100M 和 80%。缓冲区达到阈值后，将缓冲区内的内容进行 Partition，合并写入磁盘。

这样，一次缓冲区满的 80 MB 数据将被按 key 排列写入，每一片最终进入一个 Reducer。

### Hadoop 序列化

Java 的序列化比较复杂，而我们在 Hadoop 中需要的序列化只要把必须的数据依次序列化即可，可以更加紧凑。在集群间进行通讯和 RPC 时，需要进行大量的序列化和反序列化操作。

Hadoop 为 Java 的八种基本类型准备了对应的 `Writable` 类。其中，Boolean 的实现使用了一个字节来存储。另一种常用的序列化类型是 `Text`。默认使用 UTF-8 编码。

``` java
Text text = new Text("xxx");
text.set("xxx");
text.set("".toByteArray());
```

所有的序列化类型都要实现 `WritableComparable` 接口。

``` java
public interface Writable {
  void write(DataOutput out) throws IOException;
  void readFields(DataInput in) throws IOException;
}

public class IntWritable implements WritableComparable<IntWritable> {
  private int value;
  @Override
  public void readFields(DataInput in) throws IOException {
    value = in.readInt();
  }
  @Override
  public void write(DataOutput out) throws IOException {
    out.writeInt(value);
  }
  @Override
  public int compareTo(IntWritable o) {
    int thisValue = this.value;
    int thatValue = o.value;
    return (thisValue<thatValue ? -1 :
            (thisValue==thatValue ? 0 : 1));
  }
}
```

## MR 的可插拔（pluggable）组件

### IO Format

|   IO Format                |   Key      |   Value   |   备注                                      |
| -------------------------- | ---------- | --------- | ------------------------------------------- |
| `TextInputFormat`          | 字节偏移量 | 行内容    |                                             |
| `KeyvalueInputFormat`      | 行前半     | 行后半    | 默认以制表符分割，可自定义                  |
| `NLineInputFormat`         | 字节偏移量 | 行内容    | 每个Mapper 固定收到 N 行                    |
| `SequenceInputFormat`      | 自定义     | 自定义    | 读取 Hadoop Sequence 文件                   |
| `TextOutputFormat`         | key        | value     | 输出结果类似 `KeyValueInputFormat` 输入格式 |
| `SequenceFileOutputFormat` |            |           |                                             |

### 多输入 / 输出

``` java
MultipleInputs.addInputPath(
    job,
    aInputPath,
    TextInputFormat.class,
    ADataMapper.class
);
MultipleInputs.addInputPath(
    job,
    bInputPath,
    TextInputFormat.class,
    BDataMapper.class
);
MultipleOutputs.addNamedOutput(
    job,
    "text",
    TextOutputFormat.class,
    LongWritable.class,
    Text.class
);
```

### Combiner

Combiner 可以减少 Map Task 的磁盘 IO 与 Shuffle 的网络 IO。通常可以和 Reducer 使用同一个类。

``` java
job.setCombinerClass(IntSumReducer.class);
```

### Partitioner

Partitioner 的作用是为相同的 key 分配相同的 Reducer。默认的实现是 `hash(key) mod R`，R 为 Reducer 的数量。自定义 Partitioner，需要继承 Partitioner 类并重写 `getPartition` 方法：

``` java
class MyPartitioner<K, V> extends Partitioner<K, V> {
    @Override
    public int getPartition(K key, V value, int numPartitions) {
        return key.hashCode() % numPartitions;
    }
}
```

需要注意的是，如果只有一个 Reduce Task，MR 会直接进行分配，不会执行我们自定义的 Partitioner。可以使用 `job.setNumReduceTasks(number);` 设置。

考虑一个全局排序的场景。默认的 `HashPartitioner` 不能保证 Reducer 之间的有序性，因此我们需要自定义 Partitioner。Hadoop 库中的 `TotalOrderPartitioner` 使用了前缀树来处理这个问题。

## YARN

YARN 将原本 JobTracker 的任务划分为 ResourceManager 和 ApplicationManager 两部分，再统一到 YARN 中，将资源分配和任务调配统一起来。MapReduce 任务将直接部署到 YARN 上，资源的表示方法也从 slot 变成了直接的 RAM 和 CPU。

### YARN 的组成

-   ResourceManager 处理请求，启动 AM，监控 NM 与 AM，进行资源调度。

-   NodeManager 管理单个结点的资源，处理 RM 和 AM 发来的命令。

-   ApplicationMaster 负责单个任务的监控、调度以及向 RM 申请资源。

-   Container 封装运算资源

总的来看，用户将任务提交给 ResourceManager，RM 为每个具体的应用程序在某个结点上启动一个 ApplicationMaster。这时 RM 只分配了 AM 所需的资源。AM 启动后，再根据任务的情况向 RM 请求任务执行所需的资源，并与 NodeManager 通信进行计算。计算完成后，AM 向 RM 注销，表示完成任务。

Container 的作用出现在 AM 向 RM 需求分配资源时。AM 向 RM 发送的数据包括优先级、需要的 RAM、CPU 以及是否接受跨机架等信息。当然，YARN 默认会尽量在同一个机架上完成任务。然后，RM 向 AM 返回一个 Container指示了分配到的资源所在位置等信息。也就是说，一个 Container 封装了一个节点上一定量计算资源的信息。然后，AM 将计算任务与 Container 的信息一同发送给 NodeManager 进行计算。

### YARN 的容错

在整个任务过程中，AM 和 NM 都要不停地发送心跳信息。如果 AM 出现问题，RM 可以直接启动另一个 AM。如果 NM 出现问题或者任务失败频率较高，则AM 可以将任务转移到其他结点。

