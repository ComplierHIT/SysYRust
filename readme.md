# SySy编译器-compilerhit

### 项目介绍

sysy:sysy是一种图灵完备的类c语言

该项目把sysy语言编译成面向visionfive2开发板的rv64gc汇编,并实现了各种常规优化和部分进步优化

本身是作为华为毕昇杯2023的比赛项目开发,在最后的性能排名中占第18.

有少数用例超过gcc -O2优化性能,大部分用例接近gcc -O2性能

### 实现的优化

1. 前中端常规优化:

   函数内联,循环展开,不可达代码删除,gvn, pre(有bug), 常量传播,常量折叠,块重排

2. 后端常规优化:

   1. 乘除法优化

   2. 寄存器分配的分配(线性扫描,图着色),指派,溢出(启发式),接合

   3. 栈重排

   4. 块重排,块合并,部分指令上提

3. 其他前中端优化；

   1. 循环表达式归纳(比如加法变乘法)

   2. 循环不变量外提

   3. 循环消去

4. 其他后端优化:

   1. 指令调度(块内):

      使用列表调度法,只是针对两种模式进行了调度:邻def-use,邻def-\[store\]

5. 自动并行化 (对部分隐藏性能用例存在bug,初步怀疑可能线程库有漏洞)

   采用保守的方式进行自动并行化.

   针对自身IR设计简化版scev，分析循环是否存在读写冲突和写写冲突,挑选出能够并行的循环,针对IR进行修改，

   使用后端提供的线程库接口实现并行

### 可进步空间

1. 使用ILP/PBQB统一寄存器分配和溢出处理中的价值衡量,以得到更好的分配方案

   但是并没有找到rust独立实现的线性规划/PBQP求解器，都是依赖需要单独下载的求解器

2. 后端针对更多开发板特性实现更完善的调度

3. 实现局部调度,接合每个指令处寄存器压力考虑调度方案

4. 实现更完善的SCEV,能够优化更多循环以及发现更多并行机会

### 与CMMC对比

cmmc:2023毕昇杯冠军项目,许多用例性能超过gcc -O2,部分用例超过gcc -O3,甚至个别用例性能达到了gcc -O3的10倍.

* 项目组织上

  1. 我们编译器中源程序的各种中间形式之间缺乏严谨的中间检查机制,大大增加了调试的消耗

  2. 准备的项目测试用例过少,没有采用专业的编译器测试技术,比如csmith fuzzing.

* 架构无关优化上

  1. 我们的scev不够完善,对于一些能够优化到O(1)的用例还缺乏一些基础功能建设来分析

  2. 未实现循环转换,对于一些用例生成的汇编影响prefectcher的良好工作。

* 架构有关优化上

  1. 未深入研究u74文档,只实现了简单的指令调度,并且没有采用专业工具来测试IPC.而cmmc项目的IPC甚至达到了2！