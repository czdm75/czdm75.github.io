# 2. 面向对象编程

## 包

当使用 `javac` 程序时，class 文件将会被放在与源代码相同的位置。但在运行类文件时，必须将其放置在对应包的文件夹里。

```sh
# 指定class文件的位置
javac -d <path>
# jar程序的参数与tar程序类似
jar cvf xxx.jar com/*.class
# 指定jar文件的主类
jar cvfe xxx.jar com.MainClass com/*.class
# 运行jar文件
java -jar xxx.jar
```

### Class Path

class path 可以包含：

- 匹配包名的包含 class 文件的子目录
- jar 文件
- 包含 jar 文件的子目录

```sh
java -cp .:../libs/lib1.jar com.MainClass.class
# 或者：
export CLASSPATH=.:../libs/lib1.jar
```

这意味着使用当前目录和 libs 目录中的 `lib1.jar` 作为 class path。对于 Windows：

```powershell
java -cp .;..\libs\lib1.jar com.MainClass.class
# 或者：
SET CLASSPATH=.;..\libs\lib1.jar
```

class path 默认为当前目录。但如果额外设置了 class path，则需要包含 `.`。

### 包密封

在 Java 中，所有以 java 开始的包都不会被加载，这是为了防止 Java 本身被篡改。在封装 jar 包时，也可以使用类似的办法。提供一个 jar 包的 `manifest` 文件：

```manifest
Name: com/
Sealed: true
Name: net/
Sealed: true
```

或者，仅包含第二行的内容，这意味着整个 jar 中的包都被封闭。

### JavaDoc

JavaDoc 工具会将符合格式的所有包，`public` 类和 `public` 及 `protected`成员的注释转化为 HTML 文档。注释使用 `/**...*/` 格式，放在描述对象的前面，其中可以使用 HTML 的标签。只推荐使用 `<em>` `<code>` `<strong>` `<img>` 等标签，避免使用标题，以免打乱文档。

当使用图片时，将图片保存在 `doc-files` 目录下，并使用 HTML 标签：`<img src="doc-files/uml.jpg" alt="UML">`。最后：

```sh
javadoc -d path package1 packege2 ...
# 添加作者和版本信息
javadoc -version -author
# 将标准库链接到Oracle文档
javadoc -link http://docs.oracle.com/javase/8/docs/api *.java
# 将源代码生成为HTML并链接
javadoc -linksource package
```

#### 类注释

```java
/**
 * Something
 * @author someone
 * @version 1.1
 */
```

#### 方法注释

```java
/**
 * Something
 * @param param1 something
 * @return something
 * @throws something
 */
```

#### 变量注释

通常用在静态常量处

```java
/**
 * something
 */
```

#### 通用标记

```java
/**
 * @since version 1.1
 * @deprecated
 */
```

另外，还可以使用 `@Deprecated` 注解在编译中发出警告。

#### 链接

```java
/**
 * see 中可以省去包名或类名，这样会在当前包或类中查找
 * @see com.test.Clazz#method(double)
 * @see <a href="http://xxx">Link</a>
 * @see something
 * @link link的用法类似
 */
```

#### 包和概述注释

对于包，使用一个 `package-info.java` ：

```java
/**
 * something
 */
package com.test;
```

对于整个项目的 Overview 注释，提供一个 `overview.html` 文件，放在所有源文件的父目录里。JavaDoc 会将其 `<body>` 标签的内容放在 Overview 里。

### 静态导入

```java
import static java.lang.Math.*;
```

此时可以使用 Math 类的静态方法和静态变量。或：

```java
import static java.lang.Math.sqrt;
import static java.lang.Math.PI;
```

## 嵌套类

### 静态嵌套类

#### 私有静态嵌套类

```java
public class Invoice {
    private static class Item {
        //此处无需访问控制，因为Item类已经是private
        String decription;
        double price() { return quantity * unitiPrice; }
    }
    private ArrayList<Item> items = new Arraylist<>();
    public void addItem(String description) {
        ...
    }
}
```

这时，只有 Invoice 类中的方法能够访问 Item 类。

#### 公有静态嵌套类

```java
 public class Invoice {
    public static class Item {}
}
```

此时任何人都可以直接访问 `Item` 类

```java
public class Other {
    Invoice.Item item = new Item();  //其他位置可直接访问Item类
}
```

在静态嵌套类中，两个类除了访问权限和方法之外并没有实际的关系，嵌套类的使用和正常的类没有任何区别。

### 内部类

```java
public class Network {
    public class Member {
        //Member对象将会知道自己从属的Network对象
        private String name;
        public void leave() {
            members.remove(this);  //从内部类中访问外部类的变量
            //等同于 Network.this.members.remove(this)
        }
    }
    private ArrayList<Member> members;
    public Member enroll(String name) {
        members.add(new Member(name);  //等同于this.new Member()
    }
}
```

此时可以在其他位置引用：

```java
Network myFace = new Network();
//在Network类外使用 Member 类
Network.Member fred = myFace.enroll("fred");
Network.Member f = myFace.new Member();
fred.leave();
```

### 内部类的特殊情况

内部类的一个对象从属于外部类的一个对象，这会带来一些复杂的情况。

首先，内部类除了编译时常量 `static final Class variable = xxx;` 之外，并不能拥有静态成员，因为我们无法确定，这个成员是对于外部对象静态，还是对于 JVM 静态。

通常，我们可以直接在内部类中访问外部类的对象。当我们需要外部对象本身的时候，可以使用 `OuterClass.this`。

无论是嵌套类还是内部类，都将被编译成一个 `Outer$Inner.class` 文件。实现上，内部类的每个对象持有一个指向自己所从属的外部对象的引用。

### 局部类

局部类用于产生一个实现某个接口的对象，在 Java 8 之后逐渐被 lambda 表达式取代一部分。这样定义的类被控制在方法内。由于这个类不能被外部访问，所以不需要访问控制。另外，也可以声明匿名的局部类。

```java
public static IntSequence randomInts(int low, int high) {
    class RandomSequence implements IntSequence {
        // 此处可以随意访问类的变量和randomInts方法的参数等
    }
    return new RandomSequence();
}

public static InySequence randomDoubles(double low, double high) {
    return new IntSequence {
        ...
    }
}
```
