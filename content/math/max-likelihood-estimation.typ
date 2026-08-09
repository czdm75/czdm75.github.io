#let meta = (
  title: "最大似然估计",
  lang: "zh",
)

= 最大似然估计 Maximum Likelihood Estimation, MLE

训练集是

$
D={(x_i, y_i)_(i=1)^n}
$

其中对第 i 个样本 $(x_i, y_i)$，x 是其 feature，y是其label。

一个条件概率模型，其参数为 $theta$。

$
p_theta(y|x)
$

== 单个样本的 likelihood

对于单个样本，$x_i$ 和 $y_i$ 都是已经确定的常数。将其代入模型：

$
L_i(theta) = p_theta (y_i|x_i)
$

是这个样本的似然函数（likelihood contribution）。它表示，*对于特定的参数和输入，模型给 y 多大的概率分配*。这个值越大，说明模型对这个样本越准；这个值越小，说明对这个样本越不准。

== 整个数据集的 likelihood

对于整个数据集，因为每个 $y_i$ 独立，所以连乘

$
L(theta) = product_(i=1)^n L_i(theta)
$

所以对其取 $arg max$：

$
hat(theta)_("MLE")=arg max product_(i=1)^n p_theta (y_i|x_i)
$

也就是取一个 $theta$ 使得整个数据集的可解释性最大。

== Loss Function

为了方便计算，对其取负倒数

$
cal(L)(theta) &= -sum_i log p_theta (y_i|x_i) \
hat(theta)_("MLE") &= arg min cal(L)(theta)
$
