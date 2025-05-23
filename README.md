# Remusys-IR: Recrafted LLVM-like IR system

-- by Medi H.B.T.

使用 Rust 编写的类 LLVM 中间代码系统, 主要为编译器竞赛服务.

只能说这玩意除了思想和 LLVM-IR 沾点边外, 从头到尾一点都不像 LLVM.

## 开发进度

- [x] 类型系统
- [ ] 指令系统
    - [x] 通用指令系统、基本块
    - [ ] 实现 Intrinsic 机制
        - [ ] 怎么在 Module 中定义 Intrinsic 函数
        - [ ] 怎么调用 Intrinsic 函数
        - [ ] 支持常见的 Intrinsic: `memcpy` `memset`
    - [ ] 支持向量
        - [ ] 向量类型 (Fixed Vector)
        - [ ] 向量运算
        - [ ] 向量元素的插入和提取
        - [ ] 与向量有关的基本检查
- [x] DFG
    - [x] 实现 use-def 关系
    - [x] 实现 Use-Def 反图, 并可以按需启用
    - [ ] 数据流上的基础优化
        - [ ] 常量传播
        - [ ] 指令合并
        - [ ] 死指令消除
- [ ] CFG
    - [x] 实现 jump from-to 关系
    - [x] 实现控制流图反图，并可以按需启用
    - [ ] 实现控制流图导出关系
        - [x] 控制流图快照
        - [x] DFS 树
        - [x] 支配树, 后向支配树（Semi-NCA算法）
        - [ ] 循环检测
        - [ ] 实现导出关系增量更新
    - [ ] 控制流上的基础优化
        - [ ] 死基本块消除
        - [ ] 函数体排序
        - [ ] Mem2Reg 可变操作消除
- [ ] Remusys-MIR 非 SSA 中层代码
    - [ ] 设计
    - [ ] Phi 消除
    - [ ] 寄存器分配
