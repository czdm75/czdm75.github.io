<!-- markdownlint-disable MD056 -->

# 1. 基本的编程结构

## 基本类型和变量

### 数字

4 字节整型 `int` 约正负 21 亿，8字节整型 `long` 约正负 9×10^19，约 900 千亿。4 字节浮点数 `float` 约 6 位有效数字，范围至正负 10^38 ；8 字节浮点数 `double` 约 15 位有效数字，范围至正负 10^308。

`Integer` `Byte` `Short` `Long` `Double` `Float` 均有 `MAX_VALUE` 和 `MIN_VALUE` 成员变量，指示其边界范围。`Float` 和 `Double` 还有 `NaN`、`POSITIVE_INFINITY` 和 `NEGATIVE_INFINITY` 三个静态变量。`NaN` 之间互不相等。

Java 还提供了 `BigInteger` 和 `BigDemical` 两个类，尤其适用于金融。

在部分机器上，如 Intel x86平台，使用 80 bit 的浮点单元来提高浮点运算的精度。如果需要严格的 64 bit 浮点运算，可以在方法前加上 `strictfp` 修饰符。另外几个不常见的关键字是与多线程有关的 `volatile`，与序列化有关的 `transient` 和与跨语言调用有关的 `native`。此外，`StrictMath` 类也提供了类似的功能。

### 字符

`char` 描述的是 Unicode 中的**编码单元**，即一个 16 bit 整数。可以直接使用 Unicode 编码如 `\u004A`，甚至直接用数字 `0x004A` 来声明 `char`。

此外，还有转义符：`\n` `\r` `\t` `\b` `\\` 以及单引号。

## 操作符

| 运算符       | 操作      | 运算符   | 操作     | 运算符 | 操作     |
| ------------ | --------- | -------- | -------- | ------ | -------- |
| `~`          | 按位取反  | `(cast)` | 强制转换 | `new`  | 构造对象 |
| `<<`         | 左移，补0 | `>>`     | 逻辑右移 | `>>>`  | 算数右移 |
| `instanceof` | 判断实例  | `&`      | 按位与   | `^`    | 按位异或 |
| `|`          | 按位或    | `? :`    | 条件语句 | `&&`   | 逻辑与   |

- 求模操作符 `%` 对于负数是不安全的，如 `-7%5`结果为 -2。最好使用 `Math.floorMod(-7,5)` 得到 3。
- 如果整数运算中需要在溢出等特殊情况时抛出异常，需要使用 `Math.xxxExact()` 系列的方法。
- 在算术操作时，基本数字类型之间会发生**转型**。转型的 方向为：`int->long->float->double`。注意，这意味着 `short` 和 `byte` 在运算时也被作为 `int` 处理，并需要重新强制转换回来。
- 四舍五入使用 `Math.round` 方法，返回 `long` 型。如果在 cast 中担心丢失数字，可以使用 `Math.toIntExact()` 方法，溢出时会报异常。
- Java 的逻辑操作是短路的。但是，为了代码的可读性，不建议在判断语句中使用带有副作用的语句。
- `>>` 在高位补 0，`>>>` 在高位补符号位。另外，`1<<35 == 1<<3`。如果是 `long` 类型，则模 64。

## 字符串

连接与加号混合可能出现问题。如，`""+42+1` 可能得到 `421`。使用圆括号来避免。连接的实现实际上是 `StringBuilder` 类。

`String.equalsIgnoreCase` 方法忽略大小写。`String.compareTo` 方法使用 UTF-16 顺序，结果可能和直觉不一样。要使用自然语言顺序，使用 `Collator` 对象。这是一个单例的 `Comparator` 对象，其排序与 Locale 和其他设置有关。

`Integer.parseInt(String)` 返回一个 `int`，`Integer.valueOf(String)` 返回一个 `Integer`。`parseInt(String, int)` 指定数字字符串的进制。`Double` 的情况类似。

Java 的 `String` 是 Immutable 的。

### Unicode 编码问题

Java 基于 UTF-16，但 Unicode 的范围已经超过了 16 bit。于是，超出的字符使用连续两个 16 bit 的整数，即两个编码单元（Code Unit），组成一个编码点（Code Point）。这个过程与反斜杠转义类似。如果不使用高位区的字符，则可以直接用 `String.charAt(int)` 方法。这样得到的返回值是 `char`，即 16 bit 整数。如果这个字符是高位的，就会面临得到半个字符的情况。这时应该使用 `String.codePointAt` 以得到两个编码单元，其返回值是一个 `int`，即 32 bit 整数。如果字符是低位的，那么返回值的一半是全 0。否则，返回值内就是两个编码单元。还可以使用：

```java
int[] codepoints = "abc".codePoints().toArray()
```

## 控制台 IO

`Scanner` 类的 `nextLine()` 方法读取一行，`next()` 方法读取以空格为分割的单词。

读取密码时，使用：

```java
Console terminal = System.console();
char[] password = terminal.readPassword("hint info");
```

由于 `String` 是 Immutable 的，在 GC 之前，它都会留在内存里，不够安全。相比之下，`char[]` 可以在使用之后就覆盖掉。

`System.out.printf()` 和 `String.format()` 接受类似的参数，进行格式化。

## 控制流和数组

`switch` 语句可以接受的标签包括：`char` `byte` `short` `int` 及其包装类，`String` 和枚举类型。需要注意的是，由于 fall through 特性，`switch` 的各个分支之间的声明变量是共用的。

`break` 和 `continue` 语句可以使用标签，以方便跳出循环：

```java
outer:
while(){
    while(){
        break outer;
    }
}
```

对于基本类型的数组，默认使用 0 和 `false` 来填充。对于对象数组，默认使用 `null` 来填充。

数组和 `ArrayList` 之间 常常互相转换和进行深拷贝：

```java
String[] arr = list.toArray(new String[]);
ArrayList<String> list = new ArrayList<>(Arrays.asList(arr));
ArrayList<String> newList = new ArrayList<>(list);
String[] newArr = Arrays.copyOf(arr);
```

`Arrays` 和 `Collections` 类提供了数组的常见算法。

```java
Arrays.fill(arr, value);
Arrays.sort(arr);
// 扩展一个数组
arr = Arrays.copyOf(arr, arr.length * 2);
// 多线程的排序，Collections 里没有这个方法
Array.parallelSort(arr);
// Arrays 里没有这两个方法
Collections.shuffle(list);
Collections.reverse(list);
```

在使用 `toString` 时，数组和 `ArrayList` 都能够打印出元素内容。但对于多维数组，则要使用 `deepToString`。

## 可变参数

可变参数只能是最后一个参数。

```java
void method(double... nums) {
    //nums is a double[]
}
//call this method
method(1.0, 2.0);
double[] nums = {1.0, 2.0}
method(nums)
```
