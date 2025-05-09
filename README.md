# Remusys-IR: Recrafted LLVM-like IR system

-- by Medi H.B.T.

使用 Rust 编写的类 LLVM 中间代码系统, 主要为编译器竞赛服务.

只能说这玩意除了思想和 LLVM-IR 沾点边外, 从头到尾一点都不像 LLVM.

## 开发进度

- [x] 类型系统
- [ ] 指令系统
    - [x] 通用指令初始化设计
    - [ ] 实现所有指令 (?/?)
    - [ ] 实现基本块
    - [ ] 实现函数体、函数体的切换
    - [ ] 实现模块、模块内的遍历
- [ ] CFG 和 DFG
    - [x] 实现 use-def 关系
    - [x] 实现 jump from-to 关系
    - [ ] 实现控制流图
