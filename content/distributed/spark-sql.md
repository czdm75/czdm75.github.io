+++
title = 'Spark SQL Programming'
+++

# Spark SQL Programming

## Basic

### DataFrame

``` python
# standalone
from pyspark.sql import SparkSession
spark = SparkSession \
    .builder \
    .appName("Python Spark SQL basic example") \
    .config("spark.some.config.option", "some-value") \
    .getOrCreate()

# in pyspark repl
spark = SQLContext(sc)

# json file content:
# {"name":"Michael"}
# {"name":"Andy", "age":30}
# {"name":"Justin", "age":19}
df = spark.read.json("examples/src/main/resources/people.json")
# missing value is null
df.show()
df.printSchema()

df.select("name").show() # prints a column of data
df.select(df['name'], df['age'] + 1).show() # prints two columns, one of'em is operated

df.filter(df['age'] > 21).show
df.groupBy("age").count().show()
```

### SQL Queries

``` python
# temp view
df.createOrReplaceTempView("people")
spark.sql("SELCT * FROM people").show()

# global temp view
df.createGlobalTempView("people")
spark.newSession().sql("SELECT * FROM global_temp.people").show()
```

## Schema

### Infering Schema

``` python
from pyspark.sql import Row

# reading text file as RDD
# text file content:
# Michael, 29
# Andy, 30
# Justin, 19

lines = sc.textFile("examples/src/main/resources/people.txt")
parts = lines.map(lambda l: l.split(","))
people = parts.map(lambda p: Row(name=p[0], age=int([[1]])))  # RDD[Row]

schemaPeople = sql.createDataFrame(people) # transform into sql dataframe
schemaPeople.createOrReplaceTempView("people")
teenNames = spark.sql("SELECT name FROM people WHERE age >= 13 AND age <= 19")

teenNames.rdd.map(lambda p: "name: " + p.name).collect()  # RDD[String]
```

### Programatically Specifying the Schema

1.  Create an RDD of tuple or lists. (like `RDD[Tuple2]`)
2.  Create the schema as a `StructType` matching these tuples
3.  Apply the schema via `createDataFrame`

``` python
from pyspark.sql.types import *

people = sc.textFile("examples/src/main/resources/people.txt") \
    .map(lambda l: l.split(",")) \
    .map(lambda p: (p[0], p[1].strip()))  # RDD[(String, String)]

schemaString = "name age"  # cloumn names string
fields = [
    StructField(field_name, StringType(), True)
    for field_name in schemaString.split()
]  # One StructField is One field in structType

# class StructField(name, dataType, nullable=True, metadata=None)
# DataType is base class of datatypes, e.g. StringType, BinaryType, etc.

schema = StructType(fields)
schemaPeople = sql.createDataFrame(people, schema)
```

### User-defined Functions (UDFs)

``` python
spark.registerFunction(
    "CTOF",
    lambda degreesCelsius: ((degreesCelsius * 9.0 / 5.0) + 32.0)
)
spark.sql(
    "SELECT city, CTOF(avgLow) AS avgLowF, CTOF(avgHigh) AS avgHighF FROM citytemps"
).show()
```

### User-defined Aggregate Functions

``` scala
import org.apache.spark.sql.{Row, SparkSession}
import org.apache.spark.sql.expressions.MutableAggregationBuffer
import org.apache.spark.sql.expressions.UserDefinedAggregateFunction
import org.apache.spark.sql.types._

object MyAverage extends UserDefinedAggregateFunction {
  // The data type of the input value
  def inputSchema: StructType = StructType(
      StructField("inputColumn", LongType) :: Nil)
  // The data type of the aggregation buffer
  def bufferSchema: StructType = StructType(
      StructField("sum", LongType) :: StructField("count", LongType) :: Nil)
  // The data type of the returned value
  def dataType: DataType = DoubleType
  // Whether this function always returns the same output on the identical input
  def deterministic: Boolean = true


  // Initializes the given aggregation buffer. The buffer itself is a `Row` that
  // in addition to standard methods like retrieving a value at an index
  // (e.g., get(), getBoolean()), provides the opportunity to update its values.
  // Note that arrays and maps inside the buffer are still immutable.
  def initialize(buffer: MutableAggregationBuffer): Unit = {
    buffer(0) = 0L  // sum
    buffer(1) = 0L  // count
  }

  // Updates the given aggregation buffer `buffer` with new input data from `input`
  def update(buffer: MutableAggregationBuffer, input: Row): Unit = {
    if (!input.isNullAt(0)) {
      buffer(0) = buffer.getLong(0) + input.getLong(0)
      buffer(1) = buffer.getLong(1) + 1
    }
  }

  // Merges two aggregation buffers and stores the updated buffer values back to `buffer1`
  def merge(buffer1: MutableAggregationBuffer, buffer2: Row): Unit = {
    buffer1(0) = buffer1.getLong(0) + buffer2.getLong(0)
    buffer1(1) = buffer1.getLong(1) + buffer2.getLong(1)
  }

  // Calculates the final result
  def evaluate(buffer: Row): Double = buffer.getLong(0).toDouble / buffer.getLong(1)
}

val df = spark.read.json("examples/src/main/resources/employees.json")
df.createOrReplaceTempView("employees")
spark.udf.register("myAverage", MyAverage)
spark.sql("SELECT myAverage(salary) as average_salary FROM employees")
```

Or, a type-safe version, using pre-defined case classes as schema:

``` scala
import org.apache.spark.sql.{Encoder, Encoders, SparkSession}
import org.apache.spark.sql.expressions.Aggregator

case class Employee(name: String, salary: Long)
case class Average(var sum: Long, var count: Long)

object MyAverage extends Aggregator[Employee, Average, Double] {
  // Should satisfy the property that any b + zero = b
  def zero: Average = Average(0L, 0L)
  // Combine two values to produce a new value. For performance, the function may
  // modify `buffer` and return it instead of constructing a new object
  def reduce(buffer: Average, employee: Employee): Average = {
    buffer.sum += employee.salary
    buffer.count += 1
    buffer
  }
  // Merge two intermediate values
  def merge(b1: Average, b2: Average): Average = {
    b1.sum += b2.sum
    b1.count += b2.count
    b1
  }
  // Transform the output of the reduction
  def finish(reduction: Average): Double = reduction.sum.toDouble / reduction.count
  // Specifies the Encoder for the intermediate value type
  def bufferEncoder: Encoder[Average] = Encoders.product
  // Specifies the Encoder for the final output value type
  def outputEncoder: Encoder[Double] = Encoders.scalaDouble
}

// a DataSet[Employee]
val ds = spark.read.json("examples/src/main/resources/employees.json").as[Employee]
val averageSalary = MyAverage.toColumn.name("average_salary")
ds.select(averageSalary)
```

## Data Sources

``` python
df = spark.read.load("examples/src/main/resources/users.parquet")
df.select("name", "favorite_color").write.save("namesAndFavColors.parquet")

df = spark.read.load("examples/src/main/resources/people.json", format="json")
df.select("name", "age").write.save("namesAndAges.parquet", format="parquet")

df = spark.read.load(
    "examples/src/main/resources/people.csv",
    format="csv",
    sep=":",
    inferSchema="true",
    header="true"
)
df = spark.sql("SELECT * FROM parquet.`examples/src/main/resources/users.parquet`")
```

### SaveMode

-   `ErrorIfExists`
-   `Append`
-   `Overwrite`
-   `Ignore`

### Save to Persistent Table

``` python
df.write.option("path", "/some/path").saveAsTable("t")
df.write.bucketBy(42, "name").sortBy("age").saveAsTable("people_bucketed")
df.write.partitionBy("favorite_color").format("parquet").save("namesPartByColor.parquet")
```

### Hive

For Spark SQL to operate Hive, you need hive jars in classpath of every node and `hive-site.xml` `core-site.xml` `hdfs-site.xml` file in `conf/` . When not configured by the `hive-site.xml`, the context automatically creates `metastore_db` in the current directory and creates a directory configured by `spark.sql.warehouse.dir`, which defaults to the directory `spark-warehouse` in the directory the Spark application starts.

``` python
warehouse_location = abspath('spark-warehouse')

# text file content:
# 238val_238
# 86val_86
# 311val_311

spark.sql("CREATE TABLE IF NOT EXISTS src (key INT, value STRING) USING hive")
spark.sql("LOAD DATA LOCAL INPATH 'examples/src/main/resources/kv1.txt' INTO TABLE src")

spark.sql("SELECT * FROM src").show()
spark.sql("SELECT COUNT(*) FROM src").show()

sqlDF = spark.sql("SELECT key, value FROM src WHERE key < 10 ORDER BY key")  # RDD[Row]
stringsDS = sqlDF.rdd.map(lambda row: "Key: %d, Value: %s" % (row.key, row.value)).collect()  # RDD[String]

# Create a Spark dataframe
Record = Row("key", "value")
recordsDF = spark.createDataFrame([Record(i, "val_" + str(i)) for i in range(1, 101)])
recordsDF.createOrReplaceTempView("records")

# Can also join DataFrame data with data stored in Hive.
spark.sql("SELECT * FROM records r JOIN src s ON r.key = s.key").show()
```

### Hive Settings

``` sql
CREATE TABLE src(id int) USING hive OPTIONS(fileFormat 'parquet')
```

-   `fileFormat`: A fileFormat is kind of a package of storage format specifications, including "serde", "input format" and "output format". Currently we support 6 fileFormats: 'sequencefile', 'rcfile', 'orc', 'parquet', 'textfile' and 'avro'.
-   `inputFormat` `outputFormat`: These 2 options specify the name of a corresponding InputFormat and OutputFormat class as a string literal, e.g. org.apache.hadoop.hive.ql.io.orc.OrcInputFormat. These 2 options must be appeared in pair, and you can not specify them if you already specified the fileFormat option.
-   `serde`: This option specifies the name of a serde class. When the fileFormat option is specified, do not specify this option if the given fileFormat already include the information of serde. Currently "sequencefile", "textfile" and "rcfile" don't include the serde information and you can use this option with these 3 fileFormats.
-   `fieldDelim` `escapeDelim` `collectionDelim` `mapkeyDelim` `lineDelim`: These options can only be used with "textfile" fileFormat. They define how to read delimited files into rows.

`broadcast` hints Spark to broadcast the table to the cluster when join with others. **Broadcast Hash Join** is preferred but not guaranteed. When both side is broadcasted, Spark chooses the one with lower statistics.

``` python
from pyspark.sql.functions import broadcast
broadcast(spark.table("src")).join(spark.table("records"), "key").show()
```

## Pandas

### Transforming

``` python
import numpy as np
import pandas as pd

# Enable Arrow-based columnar data transfers
spark.conf.set("spark.sql.execution.arrow.enabled", "true")

# Create a Spark DataFrame from a Pandas DataFrame using Arrow

pdf = pd.DataFrame(np.random.rand(100, 3))
df = spark.createDataFrame(pdf)
result_pdf = df.select("*").toPandas()
```

### Pandas Scalar (纯量) UDFs

``` python
import pandas as pd
from pyspark.sql.functions import col, pandas_udf
from pyspark.sql.types import LongType

# Declare the function and create the UDF
def multiply_func(a, b):
    return a * b

multiply = pandas_udf(multiply_func, returnType=LongType())

x = pd.Series([1, 2, 3])
print(multiply_func(x, x))
# 0    1
# 1    4
# 2    9
# dtype: int64

# Execute function as a Spark vectorized UDF
df = spark.createDataFrame(pd.DataFrame(x, columns=["x"]))
df.select(multiply(col("x"), col("x"))).show()
# +-------------------+
# |multiply_func(x, x)|
# +-------------------+
# |                  1|
# |                  4|
# |                  9|
# +-------------------+
```

### Pandas Grouped Map UDFs

Grouped Map UDFs are used along with `groupBy()`

``` python
from pyspark.sql.functions import pandas_udf, PandasUDFType

df = spark.createDataFrame(
    [(1, 1.0), (1, 2.0), (2, 3.0), (2, 5.0), (2, 10.0)],
    ("id", "v"))

@pandas_udf("id long, v double", PandasUDFType.GROUPED_MAP)
def substract_mean(pdf):
    # pdf is a pandas.DataFrame
    v = pdf.v
    return pdf.assign(v=v - v.mean())

df.groupby("id").apply(substract_mean).show()
# +---+----+
# | id|   v|
# +---+----+
# |  1|-0.5|
# |  1| 0.5|
# |  2|-3.0|
# |  2|-1.0|
# |  2| 4.0|
# +---+----+
```

## Shuffle Hash Join / Broadcast Hash Join / Sort Merge Join

Hash Join 的思想是，首先将两个表分为小表 BuildTable 和大表 ProbeTable。将 BuildTable 以 Join Key 为 Key 构建为 HashMap，就可以将 ProbeTable 中的每条记录的 Key 在 HashMap 中进行 O(1) 的搜索，命中后再具体比较 Join Key 的实际值。整个算法的复杂度为 O(a+b)。

Broadcast Hash Join 非常好理解：对于大小表相 join 的情况，将小表的 HashMap 广播至所有 Executor，然后在所有的节点上执行 Hash Join。

当内存不足以存储小表时，使用 Shuffle Hash Join。通过 Spark 的 Shuffle 机制，将 Join Key 相同的数据发送到同一 Executor。

Sort Merge Join 目前用于两张大表互相 Join 的情况。先利用 Spark 的 Partition 将两张表重新分区，并在表内部进行排序。现在，每个节点上 A 表和 B 表数据的 Join Key 的范围相同，且都按照 Join Key 排序。然后，使用类似于归并排序的扫描方式来寻找相同的 Join Key。

