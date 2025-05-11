# Remusys-IR: Recrafted LLVM-like IR system

-- by Medi H.B.T.

使用 Rust 编写的类 LLVM 中间代码系统, 主要为编译器竞赛服务.

只能说这玩意除了思想和 LLVM-IR 沾点边外, 从头到尾一点都不像 LLVM.

## 开发进度

- [x] 类型系统
- [ ] 指令系统
    - [x] 通用指令初始化设计
    - [ ] 实现所有指令 (?/?)
    - [x] 实现基本块
    - [x] 实现函数体、函数体的切换
    - [x] 实现模块、模块内的遍历
- [ ] DFG
    - [x] 实现 use-def 关系
    - [ ] 实现 Use-Def 反图, 并可以按需启用
- [ ] CFG
    - [x] 实现 jump from-to 关系
    - [ ] 实现控制流图

