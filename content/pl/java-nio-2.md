+++
title = 'Java NIO Internal'
+++

# Buffer 内部实现与包装

## 更多形式的 get 与 put

也存在一些其他形式的 `get()` 和 `put()` 方法。

``` java
byte get();
ByteBuffer get(byte dst[]);
ByteBuffer get(byte dst[], int offset, int length);
byte get(int index);

ByteBuffer put(byte b );
ByteBuffer put(byte src[]);
ByteBuffer put(byte src[], int offset, int length);
ByteBuffer put(ByteBuffer src);
ByteBuffer put(int index, byte b);
```

可以认为，这里接收 `index` 参数的方法是"绝对的"，它们直接对 buffer 中某个位置进行操作，而不受 buffer 目前状态的影响，绕过了下面我们将提到的 buffer 的统计方法。

## Buffer 的统计方式

Buffer 的行为相当于一个简单的数组，它有三个重要的属性：

-   capacity 数组的大小。

-   limit 在写入时，就是 capacity。在读取时，就是最多能够读取的字节数。

-   position 当前的指针位置。

当 position 到达 limit 时，继续 `get` 会抛出异常。这时，可以使用 `rewind()` 将 position 重新指回开头，使用 `clear()` 将 limit 和 position 一起重置，或者使用 `compact()` 保留未读的数据。

``` java
buf            // HeapByteBuffer[pos=4 lim=4 cap=8]
buf.rewind()   // HeapByteBuffer[pos=0 lim=4 cap=8]
buf.clear()    // HeapByteBuffer[pos=0 lim=8 cap=8]
// some operation
buf            // HeapByteBuffer[pos=1 lim=2 cap=8]
buf.compact()  // HeapByteBuffer[pos=1 lim=8 cap=8]
```

可以看到，`clear()` 同时也将 buffer 设置回了写入模式。

如果我们并不想直接回到 buffer 的最开始位置，也可以通过 `mark()` 和 `reset()` 来手动标记。

``` java
buf.mark()   // HeapByteBuffer[pos=1 lim=4 cap=8]
buf.get()
buf          // HeapByteBuffer[pos=2 lim=4 cap=8]
buf.reset()  // HeapByteBuffer[pos=1 lim=4 cap=8]
```

需要注意的是，buffer 的**读取模式和写入模式并没有严格的区分**，而是通过 position 与 limit 的相对位置来控制。例如，在读取模式下调用 `get()` 将会同样返回当前位置的值（这个值是上一次读写遗留下来的），并将 position 加一。应当避免这类情况。

总的来看，写入模式下，limit 与 capacity 相等，pos 指向要写入的位置，pos 以下的是"有效数据"。读取模式下，limit 为有效数据的范围，pos 为当前读取的位置。limit 以下的是"有效数据"。在 buffer 的 `equals()` 方法中，只比较有效数据。当然，如果有效数据的长度不相等，`equals()` 一定为假。

## 类型化、包装

``` java
byte[] array = new byte[8]                  // { 0, 0, 0, 0, 0, 0, 0, 0 }
ByteBuffer buffer = ByteBuffer.wrap(array)  // HeapByteBuffer[pos=0 lim=8 cap=8]
buffer.putInt(2147004567)                   // HeapByteBuffer[pos=4 lim=8 cap=8]
array                                       // { 127, -8, -80, -105, 0, 0, 0, 0 }
buffer.flip()                               // HeapByteBuffer[pos=0 lim=4 cap=8]
buffer.getInt()                             // 2147004567
```

在这里我们做了两件事。首先，我们将一个数组包装成了 buffer。并且可以看到，对 buffer 的操作可以立刻体现在数组里。其次，我们使用了 `ByteBuffer` 的类型化 getter 和 setter。这类方法可以将 `long`、`double` 等类型的变量存入 `ByteBuffer` 并原样读取出来，对复杂数据的操作，例如 Unicode 文本和图片都有利。

除了对数组进行包装，也可以将一个 buffer 的一部分包装成另一个 buffer。与上面的情况类似，这两个 buffer 共享同一个底层数组。通过这种方法，可以更好地分离功能，让我们处理数据的代码的耦合度得到降低。

``` java
buffer.putLong(465132465415L)        // HeapByteBuffer[pos=8 lim=8 cap=8]
buffer.position(3)                   // HeapByteBuffer[pos=3 lim=8 cap=8]
buffer.limit(7)                      // HeapByteBuffer[pos=3 lim=7 cap=8]
ByteBuffer another = buffer.slice()  // HeapByteBuffer[pos=0 lim=4 cap=4]
```

## 只读、直接缓冲区、内存映射

Java NIO 还提供了只读缓冲区包装和速度更快的直接缓冲区。对于直接缓冲区，JVM 会尽量从系统 IO 调用中直接读取或写入系统调用，避免中间更多的缓冲区开销。

``` java
buffer.asReadOnlyBuffer()     // HeapByteBufferR[pos=3 lim=7 cap=8]
ByteBuffer.allocateDirect(8)  // DirectByteBuffer[pos=0 lim=8 cap=8]
```

最后，Java 还提供了将文件直接映射到内存里的方式。

``` java
MappedByteBuffer mbb = fc.map(FileChannel.MapMode.READ_WRITE, start, size);

mbb.put(0, (byte)97);
mbb.put(1023, (byte)122);
```

# NIO 中的各种 Channel

## FileChannel

前面我们已经见到 `FileChannel` 的使用，它是一种只有阻塞模式的 Channel。`FileChannel` 操作的是随机读写的文件。因此，还有一些相应的属性和方法。`truncate` 代表从当前位置截取一定长度的文件，`force` 代表强制将系统缓存写入到磁盘。接收的参数代表是否写入元数据。

``` java
long pos = channel.position();
channel.position(pos + 123);
long fileSize = channel.size();
channel.truncate(1024);
channel.force(true);
```

出于网络数据与本地文件之间的转换非常常见，NIO 提供了一些便利的转换方法：

``` java
RandomAccessFile toFile = new RandomAccessFile("toFile.txt", "rw");
FileChannel      toChannel = toFile.getChannel();

long position = 0;
long count    = fromChannel.size();

toChannel.transferFrom(fromChannel, position, count);
```

`count` 只能代表传输量的最大值，如果通道的数据量小于 `count`，就只传递通道内实际有的数据。类似地，也可以使用 `FileChannel.transferTo(position, count, channel)` 来将文件内容传输到其他位置。

## AsynchronousFileChannel 异步文件通道

异步文件通道的读写返回的是一系列的 `Future` 包装的对象，并且不会阻塞，可以立即返回。其创建和其他的通道类似：

``` java
AsynchromousFileChannel fileChannel = AsynchronousFileChannel.open(
    Paths.get("data/test.xml"), StandardOpenOption.READ);

Future<Integer> operation = fileChannel.read(buffer, 0);
while(!operation.isDone());
```

这里 `read()` 的第二个参数是读取文件的起始位置。然后，我们可以通过 `isDone()` 来判断读取的完成。

``` java
fileChannel.read(buffer, position, attachment,
    new CompletionHandler<Integer, ByteBuffer>() {
    @Override
    public void completed(Integer result, Attatchment attachment) {
        ...;
    }

    @Override
    public void failed(Throwable exc, Attachment attachment) {
        ...;
    }
});
```

这里的第三个参数是 IO 操作的 Attachment，和前面 `SelectionKey` 的 Attachment 类似，可以为 null。然后，我们继承一个 `CompletionHandler` , 其第二个泛型类型变量是 Attachment 的类型。如果 `read` 成功，随即就会调用 `completed` 方法。通常，对于一般的网络应用，我们可以把同一个 buffer 作为 Attachment 传入，并在 `completed` 方法中使用 buffer 回应。相应地，如果读取失败，会将对应的 `Throwable` 传到 `failed` 方法。

写数据的方法和读取相似：

``` java
AsynchromousFileChannel fileChannel = AsynchronousFileChannel.open(
    Paths.get("data/test.xml"), StandardOpenOption.WRITE);

// possible throws java.nio.file.NoSuchFileException
Future<Integer> operation = fileChannel.write(buffer, position);
while(!operation.isDone());

fileChannel.write(buffer, position, buffer,
    new CompletionHandler<Integer, ByteBuffer>() {

    @Override
    public void completed(Integer result, ByteBuffer attachment) {
        ...;
    }

    @Override
    public void failed(Throwable exc, ByteBuffer attachment) {
        System.out.println("Write failed");
        exc.printStackTrace();
    }
});
```

## SocketChannel

前面见到了服务端开启 `SocketChannel` 的方法，并将其和传统方式比较了一下。客户端的情况还要更简单一些：

``` java
SocketChannel socketChannel = SocketChannel.open();
socketChannel.connect(new InetSocketAddress("http://jenkov.com", 80));
```

阻塞模式下的操作我们已经比较熟悉，和传统 IO 基本相同。在非阻塞模式下，就要使用更多的方法来检查 Channel 的情况：

``` java
socketChannel.configureBlocking(false);
socketChannel.connect(new InetSocketAddress("http://jenkov.com", 80));

while(!socketChannel.finishConnect()){
    // wait
}
```

类似地，在非阻塞模式下的 `read` 方法总是能被调用，同时也意味着它有可能并不读出任何数据。因此，必须要检查方法的返回值，即读入的字节数。非阻塞模式和 Selector 的运行方式也更加吻合。

## ServerSocketChannel

阻塞模式的运行方式我们同样已经比较熟悉。在非阻塞模式下，调用 `accept()` 方法可能会返回 null，代表没有连接建立。

## DataGramChannel

UDP 本身就是无连接的，因此数据包通道的操作和传统 IO 没有太大的区别：

``` java
DatagramChannel channel = DatagramChannel.open();
channel.socket().bind(new InetSocketAddress(9999));
channel.receive(buf);

int byteSent = channel.send(buf, new InetSocketAddress("example.com", 80));

// connecting to a specific address
channel.connect(new InetSocketAddress("example.com"), 80));
```

如果在 `recieve` 时实际数据量超过了 buffer 的大小，多出的数据会被直接抛弃。

在调用了 `connect` 方法之后，就可以像 TCP 一样调用 `read` 和 `write` 方法。只不过，背后实际调用的仍然是 TCP 模式，数据的读写也无法得到保证。

## Pipe 管道

管道能够提供将一个通道的数据发送到另一个的功能。也可以通过方法将数据取出来。这是一个抽象类，具体的实现需要继承编写。

``` java
Pipe pipe = Pipe.open();
Pipe.SinkChannel sinkChannel = pipe.sink();
Pipe.SourceChannel sourceChannel = pipe.source();
```

# 提供其他相关功能的类

## java.nio.file.Path

`Path` 是在 Java 7 的 NIO 2 中引入的，其伴随类 `Paths` 同样位于 `java.nio.file` 包。这个类和原来的 `java.io.File` 类的功能类似，在 NIO 中，专用来处理路径。而关于具体文件的操作大多交给 `Files`，以 `Path` 作为其参数。

``` java
Path path = Paths.get("/home/user/a.txt");
// use base path and relativepath to create absolute path
Path file = Paths.get("/home/user", "a.txt");

Paths.get("/home/./../opt").normalize();
// equals /opt
```

如果将 UNIX 绝对路径用在 Windows 上，得到的将会是相对于当前磁盘的路径。

## java.nio.file.Files

`Files` 用来和 `Path` 类共同操作文件，将其和传统的 `java.io.File` 类区分开。

### 对文件的操作

在操作文件时，需要注意，并不支持递归创建。也就是说，在创建一个文件之前，必须首先创建其父文件夹。

``` java
// check if exist
if (!Files.exists(path)) {
    Files.createFile(path);
}

if (!Files.exist(path, new LinkOption[]{LinkOption.NOFOLLOW_LINKS})) {
    ...;
}

// create, copy, throws FileAlreadyExistsException, IOException
Path newdir = Files.createDirectory(path);
Files.copy(sourcePath, destinationPath);
Files.copy(sourcePath, destinationPath,
    StandardCopyOption.REPLACE_EXISTING);

// move, delete, throws IOException
Files.move(sourcePath, destinationPath,
    StandardCopyOption.REPLACE_EXISTING);
Files.delete(path);
```

### FileVisitor 遍历目录结构

`walkFileTree()` 方法可以查看目录结构。这个方法需要接受一个自己实现的 `FileVisitor` 对象。Java 只提供了一个全空实现的 `SimpleFileVisitor`。

``` java
enum FileVisitResult {
    CONTINUE, TERMINATE,
    SKIP_SIBLINGS, // dont visit brother files;
    SKIP_SUBTREE;  // dont visit subdir and subfile
                   // only effective in preVisitDirectory,
                   // or continue;
}

Files.walkFileTree(path, new FileVisitor<Path>() {
    @Override
    public FileVisitResult preVisitDirectory(Path dir,
        BasicFileAttributes attrs) throws IOException {
        //...;
        return FileVisitResult.CONTINUE;
    }
    @Override
    public FileVisitResult visitFile(Path file,
        BasicFileAttributes attrs) throws IOException {
        //...;
        return FileVisitResult.CONTINUE;
    }
    @Override
    public FileVisitResult visitFileFailed(Path file,
        IOException exc) throws IOException {
        //...;
        return FileVisitResult.CONTINUE;
    }
    @Override
    public FileVisitResult postVisitDirectory(Path dir,
        IOException exc) throws IOException {
        //...;
        return FileVisitResult.CONTINUE;
    }
});
```

`FileVisitor` 的这种定义是为了解决递归问题。例如，我们要删除一个目录，必须先删除其所有的子目录和文件。我们不继承 `FileVisitor`，而是继承 `SimpleFileVisitor`，以继承类内的空方法。

``` java
Files.walkFileTree(rootPath, new SimpleFileVisitor<Path>() {
    @Override
    public FileVisitResult visitFile(Path file, BasicFileAttributes attrs) throws IOException {
        System.out.println("delete file: " + file.toString());
        Files.delete(file);
        return FileVisitResult.CONTINUE;
    }

    @Override
    public FileVisitResult postVisitDirectory(Path dir, IOException exc) throws IOException {
        Files.delete(dir);
        System.out.println("delete dir: " + dir.toString());
        return FileVisitResult.CONTINUE;
    }
});
```

### Files 与 Stream

在 Java 8 中，`Files` 增加了新的便于逐行处理文本文件的方法。

``` java
public static Stream<String> lines(Path path,Charset cs) throws IOException {}

public static Stream<Path> walk(Path start, int maxDepth,
    FileVisitOption... options) throws IOException {}

public static Stream<Path> find(Path start, int maxDepth,
    BiPredicate<PathBasicFileAttributes> matcher,
    FileVisitOption... options) throws IOException {}
```

在 `BufferedReader` 中也有类似的方法。

## 文件锁

Linux 的文件锁可以从两个维度来区分：**共享锁-排他锁**，和**劝告锁-强制锁**。通常，读锁是共享的，写锁是排他的。Java 在这里使用的是劝告锁，这意味着内核并不真正禁止对文件的访问，而只是为应用程序提供一个已经上锁的状态指示。

``` java
// open with write mode, gets exclusive lock
RandomAccessFile raf = new RandomAccessFile("usefilelocks.txt", "rw");
FileChannel fc = raf.getChannel();
FileLock lock = fc.lock(start, end, false);
lock.release();
```

考虑到不同的 OS 可能提供不同种类的锁，为了保证安全，最好的办法是将所有锁都视为排他、劝告式的。
