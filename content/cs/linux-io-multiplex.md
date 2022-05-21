+++
title = 'Linux IO Multiplexing'
+++

# Linux IO 多路复用

## 同步/异步 阻塞/非阻塞

### 概念

《UNIX Network Programming》中给出了五种 IO 模型：

-   同步阻塞 IO

-   同步非阻塞 IO

-   IO 多路复用

-   信号驱动 IO

-   异步 IO

在考虑 IO 模式之前，要先回顾一些关于操作系统的知识。在程序调用**系统调用**（System Call）时，程序转而进入内核态，在内核态中完成 IO 操作，并将 IO 的数据从内核空间的 buffer 拷贝到用户空间的 buffer 中。回到用户态之后，程序将可以从 buffer 中取得数据。

### 同步阻塞 IO

对于同步阻塞 IO，在调用系统的 `recvfrom` 调用之后，进入内核态，线程阻塞。直到系统内核接收到数据并把数据拷贝到用户空间 buffer 全部完成后，才能从阻塞中恢复。

### 同步非阻塞 IO

在非阻塞模式的 IO 中，如果调用 `recvfrom` 时系统尚未收到对应的数据，则不会阻塞，而是直接返回 `EWOULDBLOCK` 错误。如果调用时已经收到了数据，才会在拷贝 buffer 的过程中阻塞。通常来说这个时间是短暂而稳定的，因此这种 IO 模式完全可以认为是非阻塞的。

### 多路 IO 复用

`select` `poll` `epoll` 都属于这一类。这种 IO 模式并没有实现真正的异步 IO，却能够获得异步 IO 的一些优点。

例如，假设我们现在需要监听多个端口的 UDP 数据。在同步阻塞 IO 下，我们需要为每个端口的监听准备一个线程，这些线程全部阻塞住，有数据时再进行处理。这在高并发场景下会带来海量的 RAM 需求。

然后我们考虑在同步非阻塞 IO 下应该怎样处理。一个比较可行的方法是，在一个线程内对所有的 IO 进行轮询。如果返回错误，就访问下一个监听端口。如果有数据，则处理数据。这种方法十分可行，但编码复杂，也存在一定程度的性能浪费。

多路 IO 复用可以认为是替我们完成了上述的过程（当然不完全一样）。例如 `select`，就是阻塞直到所有被监听的事件中有一个获得数据，就立刻返回。这样，我们就能在一个线程之内处理许多个 IO，这就叫做多路 IO 复用。

### 信号驱动 IO

信号驱动的 IO 并不需要用户程序去查询是否有 IO 数据到达，而是让用户程序在系统中注册。当数据到达时，内核反过来通知用户程序，用户程序阻塞并开始进行 buffer 拷贝。这种形式的 IO 比较少见。

### 异步 IO

异步 IO 在调用时立刻返回用户态，但并不得到结果。直到系统得到数据并拷贝到 buffer 之后，系统通知客户程序，客户程序可以开始处理数据。

遗憾的是，Linux 尚未实现真正异步的网络 IO。实际情况下，程序需要阻塞在拷贝 buffer 过程。在 Windows、Solaris 等系统的 IOCP 模型下可以实现真正的异步。

![](../io-multiplexing.png)

## select 与 poll 的问题

### Linux 的 wakeup callback 机制

在介绍 IO 复用之前需要首先介绍 wakeup callback 机制。Linux 为每个 socket 维护一个 `sleep_list`，列表中存储 `wait_entry`，存储监听该 socket 的进程信息。

-   调用 `select` `poll` `epoll_wait`，陷入内核，查看监听的 socket 是否有事件。如果有，就返回处理。

-   如果没有，为当前进程建立一个 `wait_entry` 节点，插入到所监控 socket 的 `sleep_list` 中。

-   进入循环，阻塞

-   当 socket 事件发生时，遍历 socket 对应的 `sleep_list`，调用其 callback 函数。通常会唤醒对应的进程进入 CPU 就绪队列。

-   队列遍历完成或遇到排他的 `wait_entry`，停止遍历。

### select

一个进程可以通过 `select` 监听多个 socket，并阻塞。在所有监听的事件中有任何一个发生时，进程从阻塞中恢复，我们可以遍历所有监听的事件来查看到底是哪一个可用。

回到上面的 wakeup callback 机制。要达到这样的效果，我们需要把这个进程插入到所有监听的 socket 的 `wait_list` 中去。当其中任何一个有事件发生时，进程就会被唤醒。

``` c
int select(
    int nfds,
    fd_set *restrict readfds,
    fd_set *restrict writefds,
    fd_set *restrict exceptfds,
    struct timeval *restrict timeout
);
```

这里的 `fd_set` 的实质内容是一个 `long int`。可以看到，select 将三类事件分开传入。

调用时，程序将会把要监听的 `fd_set` 文件标识符拷贝到内核态。可以想到，这时系统会首先先检查所有的监听事件是否有发生的。如果有，就可以立刻进行返回。如果所有事件都没有，就进入阻塞状态。当然，还可以设置超时时间。

随后，当事件发生时，我们需要重新遍历一遍所有的监听事件，因为我们不知道是否还有其他事件同时发生。收集到所有的事件之后，我们才可以返回用户态。

`select` 存在的问题：

-   将 `fd_set` 拷贝到内核态的开销。

-   为了减少拷贝 `fd_set` 的影响，select 的集合大小在内核中限制为 1024。对于高并发的情形，仍然需要做很多额外的工作。

-   得到事件通知后，我们仍然需要遍历整个列表来寻找究竟是哪一个发生了事件。

### poll

`poll` 的出现解决了 `select` 的 1024 限制问题。函数原型如下：

``` c
struct pollfd {
    int fd;                        /* File descriptor to poll.  */
    short int events;              /* Types of events poller cares about.  */
    short int revents;             /* Types of events that actually occurred.  */
};

int poll(struct pollfd *fds, nfds_t nfds, int timeout);
```

`poll` 不再使用 `fd_set`，而是把要监听的事件封装到 `pollfd` 中去，减少了参数的数量。

然而，所有的文件描述符仍然需要拷贝到内核态，并且当描述符超过 1024 个之后，性能仍然会下降。也就是说，`poll` 并没有解决 `select` 的根本问题。

## epoll

### epoll 的使用示例

``` c
int epoll_create(int size);

int epoll_ctl(
    int epfd,
    int op,
    int fd,
    struct epoll_event *event
);

int epoll_wait(
    int epfd,
    struct epoll_event *events,
    int maxevents,
    int timeout
);
```

``` c
struct epoll_event ev;

int main() {
    // these functions return -1 while error occurs
    epfd = epoll_create(argc - 1);
    fd = open(argv[j], O_RDONLY);
    epoll_ctl(epfd, EPOLL_CTL_ADD, fd, &ev)

    ready = epoll_wait(epfd, evlist, MAX_EVENTS, -1);
    for (int j = 0; j < ready; j++) {
        if (evlist[j].events & EPOLLIN) {
            // process event
        }
    }
}
```

### 解决集合拷贝问题

回顾 `select` 和 `poll` 我们会发现，每次调用都需要重新将整个文件描述符集合传入内核态。然而实际情况下，这个集合变化并不频繁，我们的服务器通常固定监听少数几个端口。即使变化，整个集合也很少会全部变化，而是少数的增加和删除。因此，我们首先可以将这个过程剥离成为另一个函数：`epoll_ctl`。也就是说，一次拷贝，多次使用。然后，我们设立几个操作：`EPOLL_CTL_ADD` `EPOLL_CTL_MOD` `EPOLL_CTL_DEL`。这样，我们在拷贝上的开销就减少到了最低。

不过，这个过程还涉及到对描述符集合的 CRUD。由于我们已经突破了 1024 的限制，在这里使用一个高效的数据结构就存在必要。在 Linux 2.6.8 之前，这里使用的是 Hash。因此，`epoll_create` 的原型里包含了初始容量。其后，Linux 改用了红黑树来处理这个问题。因此，这个参数现在没有任何作用。

其次，在事件就绪后，我们还需要回头再次把文件描述符拷贝出来。对于这个问题，`epoll` 的解决办法是将用户空间和内核空间的地址映射到同一块物理内存上去。这样，用户程序就无需拷贝可以直接访问。

### 遍历 fd 集合

`select` 的另一个问题是经常需要遍历整个集合。面对这个问题，`epoll` 拿出的解决办法是使用一个中间层。

在 `select` 中，我们需要在所有 socket 的睡眠队列中插入对应的进程的标识。相比之下，`epoll` 将进程睡眠在 `ready_list` 上，而将操作 `ready_list` 的操作绑定在 socket 的睡眠队列上。这样，当 socket 发生事件时，所对应的事件就被加入到 `ready_list` 中。于是，在从阻塞中返回之前，我们只要看遍历 `ready_list`，而不再需要遍历所有监听的事件。总的来看，`epoll` 把对于整个兴趣列表的遍历变成了对所有已经有事件就绪的兴趣的列表的遍历。

## epoll 的边缘触发与水平触发

### 概念

在边缘触发情况下，信号仅在 buffer 状态变化时发出。而在水平触发的情况下，在整个 buffer 的状态，信号都会持续发出。

以 `epoll` 的读取就绪事件为具体例子，在 ET 模式下，仅当数据到达 buffer 时，该 socket 对应的事件被加入到 `ready_list` 以供读取。而在 LT 模式下，只要 buffer 不为空，这个 socket 对应的事件就会保持在 `ready_list`。

### 区别

显而易见，LT 模式下的触发和遍历都要更多一些，看起来性能会更弱一些。不过，在良好网络状况的状态下，如果一个 socket 紧接着就得到了另一组数据，那么相比 ET 模式，我们就省去了一次重复的复制过程。因此，两种方式的性能区别实际上并不明显。

考虑我们希望多次读取一个 socket 的情况。显然，我们不能在一次循环中直接调用两次 `recv`，因为可能会在这里阻塞。然而，如果在 ET 模式下再次调用 `epoll_wait`，由于 ET 模式没有被触发，无法再次得到这个 socket。总之，在阻塞模式下 ET 模式会遇到很多问题。因此，阻塞模式下不能使用 ET。如果要使用 ET，必须使用非阻塞模式，在一次循环中将所有数据全部读取出来。

相比之下，在 LT 模式下，我们可以在一次循环中只处理一个请求，因为即使没有处理，下一次 `epoll_wait` 也会将其取出，不会发生饿死的情况。

