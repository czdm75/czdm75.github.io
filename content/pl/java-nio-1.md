+++
title = 'Java NIO Usage'
+++

# Java NIO: 应用

## 基本概念

Java NIO 是一种非阻塞的、面向块而非字节的 IO 方式。虽然 Java 的传统 IO 也进行了一些基于 NIO 的改造，NIO 仍然能够带来许多优势。

面向流的 IO 方便我们一个字节一个字节地处理数据，有利于实现过滤等功能，更加优雅和简单。相应地，其速度通常比较慢。

Java NIO 的模型由三部分组成。

-   Channel 通道，类似于传统 IO 中的流，用来实际传输数据。
-   Buffer 缓冲，我们用来读取和发送数据的位置。
-   Selector 选择器，可以在一个线程上绑定多个 Channel 和对应的 Buffer。

### Channel

Channel 和流非常相似。区别是，通道支持异步读写，支持双向读写，而且基于缓冲区。相比之下，流通常是单向同步读写的。

常用的 Channel 主要包括：

-   `FileChannel` 文件
-   `DatagramChannel` UDP数据报
-   `SocketChannel` TCP 套接字
-   `ServerSocketChannel` TCP 服务端套接字

### Buffer

Java 中的各种基本类型都有其对应的 Buffer，最常用的是 `ByteBuffer`。可以通过 Channel 或者手动写入数据。

``` java
ByteBuffer buffer = ByteBuffer.allocate(1024);
// through a channel
int bytesRead = inChannel.read(buf);
// manually input
buf.put((byte)127);
```

然后，要从 buffer 中读取数据，需要首先 `flip()` 它，变成读取模式。

``` java
buf.flip();
// write from buffer into channel.
int bytesWritten = inChannel.write(buf);
// manually get
byte aByte = buf.get();
```

需要注意的是，很多 Channel ,如 `FileChannel` 和非阻塞模式下的 `SocketChannel` 的 `write` 方法**并不能**保证将 buffer 全部写入文件。因此，要使用循环来处理：

``` java
while(buf.hasRemaining()) {
    channel.write(buf);
}
```

下面是一个简单的输出文件内容的示例：

``` java
RandomAccessFile aFile = new RandomAccessFile("data/nio-data.txt", "rw");
FileChannel inChannel = aFile.getChannel();
ByteBuffer buf = ByteBuffer.allocate(48);

int bytesRead = inChannel.read(buf);
while (bytesRead != -1) {
    buf.flip();
    while(buf.hasRemaining()){
        System.out.print((char) buf.get());
    }
    buf.clear();
    bytesRead = inChannel.read(buf);
}
aFile.close();
```

`InputStream` 和 `OutputStream` 类也有类似的 `getChannel` 方法。当然，这样开启的通道只能是单向的。下面是一个将输入内容传输到输出的可复用的代码片段：

``` java
while (true) {
    buffer.clear();
    int r = fcin.read( buffer );

    if (r==-1) {
        break;
    }

    buffer.flip();
    fcout.write( buffer );
}
```

### Scatter / Gather

Scatter 和 Gather 可以译为分散和聚集，指的是向同一个通道写入和读出多个 Buffer 的过程。在处理复杂结构的数据，如 Header-Content 时，有利于代码整洁。Scatter Read 指从一个 Channel 读取到多个 Buffer，Gather Write 指从多个 Buffer 写入到一个 Channel。关于网络的内容还会在后面进一步解释。

``` java
ServerSocketChannel ssc = ServerSocketChannel.open();
ssc.socket().bind( address );
SocketChannel sc = ssc.accept();

int messageLength = firstHeaderLength + secondHeaderLength + bodyLength;

ByteBuffer buffers[] = new ByteBuffer[3];
buffers[0] = ByteBuffer.allocate( firstHeaderLength );
buffers[1] = ByteBuffer.allocate( secondHeaderLength );
buffers[2] = ByteBuffer.allocate( bodyLength );

int bytesRead = 0;
while (bytesRead < messageLength) {
    long r = sc.read( buffers );
    bytesRead += r;
}

// flip all buffers

long bytesWritten = 0;
while (bytesWritten<messageLength) {
    long r = sc.write( buffers );
    bytesWritten += r;
}
```

## 网络和异步 IO

异步 IO 的模式实际上来自于操作系统，如 Linux 的 IO 复用和 Windows 的 IOCP。因此，类似的编程模式很可能也适用于其他语言。

### TCP 异步 IO 的例子

异步 IO 不会阻塞，这也使得它可以处理许多的 IO 连接。在传统 IO 下，这通常需要通过轮询或多线程来实现。

首先回顾一下普通的 IO 编程中处理 TCP 连接的方法：`ServerSocket` 类监听端口，客户端的 `Socket` 类构造时发出连接请求。这时，`ServerSocket.accept()` 从阻塞中脱离，返回服务端的 `Socket` 对象。

然后，我们来看异步处理的方法：

``` java
int[] ports = ...;
Selector selector = Selector.open();
for (int i : ports) {
    // new channel
    ServerSockertChannel channel = ServerSocketChannel.open();
    // config
    channel.configureBlocking(false);
    channel.socket().bind(new InetSocketAddress(i));
    // register to selector
    SelectionKey key = channel.register(selector, SelectionKey.OP_ACCEPT);
}
```

这里的 `SelectionKey.OP_ACCEPT` 是适用于 `ServerSocketChannel` 的唯一事件，即 TCP 连接建立的事件。

``` java
int num = selector.select();
Set<SelectionKey> keys = selector.selectedKeys()
```

`select()` 方法会阻塞直到有任何一个连接建立。`selectedKeys` 返回一个 `Set<SelectionKey>` 对象。在异步 IO 的类似处理过程中，由于我们已经通过 `select` 得到了这个连接信息，就不必再担心 `accept` 会阻塞：

``` java
for (SelectKey key : keys) {
    if ((key.readyOps() & SelectionKey.OP_ACCEPT) == SelectionKey.OP_ACCEPT){

        // Accept the new connection
        // accept() the new socket and register to selector
        ServerSocketChannel channel = (ServerSocketChannel)key.channel();
        SocketChannel sc = ssc.accept();
        sc.configureBlocking( false );
        SelectionKey newKey = sc.register(selector, SelectionKey.OP_READ);

    } else if ((key.readyOps() & SelectionKey.OP_READ) == SelectionKey.OP_READ) {

        // Read the data from socket
        SocketChannel sc = (SocketChannel)key.channel();
        // use buffer to proccess
    }
    keys.remove(key);
}
```

可以看到，我们将 `accept` 得到的结果重新放回了 `selector` 的监听列表，并且将监听事件修改为了 `SelectionKey.OP_READ`，即有数据到达的事件。这个过程和传统 IO 中从 `ServerSocket.accept()` 获得 `Socket` 的过程类似。

然后，在 `if` 语句的另一个分支，我们来处理接收数据的过程。使用 `channel()` 方法得到双向读写的通道对象。随后，我们就可以使用之前熟悉的 buffer 来处理这个连接了。

最后，我们需要把处理结束的连接从 `keys` 中删除，以免重复处理。在实际的应用场景中，我们还需要把关闭的连接从 `Selector` 中去除，并且很有可能使用多线程。

### SelectionKey

上面我们见到了 `OP_READ` 和 `OP_ACCEPT`。除此之外，NIO 还有 `OP_WRITE` 和 `OP_CONNECT` 两种事件。可以认为每个事件代表"就绪"：例如当连接另一方传来数据时，连接处于"读就绪"状态，对应事件 `OP_READ`。因此，写就绪和连接就绪这两种事件并不常用。

四种事件的值分别为 1、2、8、16，因此可以使用位操作来处理这些事件。例如：

``` java
int intreastSet = OP_READ | OP_ACCEPT;
if ((interest & OP_READ) == OP_READ) {}
```

相应地，`SelecttionKey` 也提供了一些处理这些信息的方法。

``` java
boolean isAcceptable();
int interestOps();  // all interest ops
int readyOps();     // ops that already triggered
```

还可以为每个 `SectionKey` 附加一个对象，以方便识别类似的对象。

``` java
selectionKey.attach(theObject);
// or
SelectionKey key = channel.register(selector, SelectionKey.OP_READ, theObject);

Object attachedObj = selectionKey.attachment();
```

### Selector

除了 `select()` 方法外，`Selector` 类同样提供了带有超时的阻塞方法和非阻塞，允许返回 0 的方法。

``` java
int select(int timeout);
int selectNow();
```

如果在阻塞期间调用 `Selector` 的 `wakeUp()` 方法（当然，是在另一个线程里），线程会立刻放弃阻塞。在操作结束之后，需要使用 `Selector.close()` 方法。这将会使所有的 `SelectionKey` 都无效，但并不会关闭 Channel。

## 异步 IO 设计

### 概述

阻塞 IO 和异步 IO 的区别是显而易见的。阻塞 IO 是一种成功的设计，它能够保证 IO 的可靠和简单。但在这种模式下，每个 IO 都需要单独的一个线程来处理。在 JVM 的默认参数下，32 bit 系统的一个栈为 320kB，64 bit 下更达到了 1MB，在高并发情况下这是完全无法接受的。线程池是解决这个问题的一种途径，但当我们面临大量低速长链接的时候，问题仍然没有被彻底解决。而这正是大规模互联网应用的常态。因此，异步 IO 成为了必然的选择。异步 IO 的最典型特征是，每一次检查不再是*阻塞或获得整块数据*，而是*0或数据*。这虽然解决了多线程的问题，却带来了另外一些需要解决的问题。

异步 IO 首先要解决的问题是，怎样用一个线程处理许多连接。于是，我们有了 Selector，使用一个 Selector 来处理许多连接，以实现"阻塞直到有一个"的效果，而不需要去处理那些尚未读到数据的连接。于是，线程资源被充分地利用起来。

第二个问题是，由于所有的操作都被立即返回，异步 IO 下读到的数据不总是完整的。甚至，在连续传输的情况下，几乎总是不完整的。于是，我们需要做两件事：

-   判断当前的数据是否是完整的

-   将不完整的数据暂存起来，以备下一次传输时拼合起来。

于是，我们在 Selector 与 Channel 之间加入一个 Message Reader，用来处理这些工作。在工程化的实践中，我们可能希望这套系统能够处理各种不同的协议。因此，可能会接收一个 Message Reader 的工厂作为参数，以进行依赖注入。

### Message Reader 的实现

前面我们看到，Message Reader 需要能够在内部的一个 Buffer 中存储不完整的 Message。显而易见，这个 buffer 的大小应该等于消息的最大值。但这时我们又遇到了之前说的内存不足的问题：百万级别的 1 MB buffer 意味着 1 TB 的 RAM 空间。因此，我们需要在这里使用可伸缩的（flexible）buffer。

#### 拷贝扩容

一种常见的方法是熟悉的拷贝扩容，也就是当 buffer 已满后将所有内容复制到一个更大的数组中去。在这种方式下，threshold 的 选取就是一个重要的问题。例如，假设一个系统的请求消息不大于 4 kB，传输的文件通常不大于 128 kB，更大的文件则没有规律性。那么，我们就会将 threshold 设置为 4kB 和 128 kB，将最终的内存占用控制在 GB 级别。

#### 追加扩容

另一种常见的方式是追加（append）扩容，方法是用一个列表将所有小的 buffer 片段集合起来，或者将一个大的数组分片，再用列表来管理分片。后者在内存模型上会更有利一些，但需要对并发量的准确判断。追加扩容的缺点也很明显，维护和读取都比较复杂。

#### 使用 TLV 消息

许多协议，包括 HTTP/2 在内，开始使用 TLV 格式的消息。TLV 指的是 Type-Length-Value 的元组。对于这类消息，我们可以在一开始就知道消息的长度，并为其开辟好内存空间，避免了上面的方式中对内存资源的浪费。

当然，TLV 格式也有其缺点。对于很长的 TLV 消息，我们就需要很大的内存空间的预开辟，这也为 DoS 攻击提供了空间。一种解决方案是使用分段 TLV 的消息格式，但这并不能彻底解决问题。另一种方式是为消息设置超时时间。这样，服务器至少能够在一段时间的无响应后恢复。

### 写不完整的消息

前面已经提到，非阻塞模式的通道并不能对一次 `write()` 实际写入的数据量做出保证，而是将写入的数据的字节数返回给调用者。于是，为了进一步解耦和提高效率，我们还需要在数据处理者和 Channel 同样准备一个 Message Writer，用来处理这个不稳定的输出过程。

回过头来想，我们在这里并不想为每个连接都维护一个线程。因此，我们只希望对有消息可写的 Writer 进行处理。因此，我们使用这样一个过程：

当 Message Writer 有消息可写时，才将其对应的 Channel 注册到 Selector。然后，服务器在空闲时检查 Selector 来获取可写的 Channel，并寻找其对应的 Writer 以写入数据。在 Writer 已经没有数据可写时，将 Channel 从 Selector 上解绑。

### 集成

现在我们已经理清了输入和输出两个部分，现在我们从整个服务器的角度来思考。总的来说，一个服务器会执行这样一个循环：

从 ServerSocket 中获取 Socket =\> Select 读事件 =\> 将接受的数据交给 Reader 来处理 =\> 在核心部分处理 Reader 传来的完整数据 =\> 将处理后的数据交给 Writer =\> Select 写事件

当然，这些功能还可以在多个线程内完成。
