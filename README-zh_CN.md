# Remusys-IR: Recrafted LLVM-like IR system

**Languages:** [English](README.md) | [中文](README-zh_CN.md)

CSCC 2025 参赛项目, 使用 Rust 编写的类 LLVM 中间代码系统, Musys IR 的重新设计. 附带一个参赛性质的 AArch64 RIG 后端.

## ⚠️ 警告

**该项目是实验性项目, 仅可用于学习研究, 请勿用于实际生产环境!**

**项目处于初期开发 / 测试阶段, 没有任何 API 稳定性, 代码架构、接口、实现等随时发生破坏性变化, 倘若造成后果, 则由使用者自负.**

## 构建指南

由于是实验性项目, 该项目并未上传至 crates.io——等哪天 cargo 能像 GitHub 那样按用户分类项目了再说. 要在自己的项目中添加 `remusys-ir` 作为依赖, 请在 `Cargo.toml` 中添加下面这一段:

```toml
[dependencies]
remusys-ir = { git = "https://github.com/medihbt/remusys-ir" }

# 或者指定特定分支
remusys-ir = { git = "https://github.com/medihbt/remusys-ir", branch = "master" }

# 或者指定特定标签/版本
remusys-ir = { git = "https://github.com/medihbt/remusys-ir", tag = "v0.1.0" }

# 或者指定特定 commit
remusys-ir = { git = "https://github.com/medihbt/remusys-ir", rev = "commit-hash" }
```

该项目目前没有任何 FFI 绑定, 将来若有精力, 等 API 稳定后可能会实现一个 GObject 绑定.

## 技术栈说明

### 中间代码 (IR)

类 LLVM 中间代码, 提供完整的数据流与控制流分析框架. IR 模块包括如下部分:

- 类型系统: 定义了整个 IR 系统会使用的类型、类型关系、类型存储等.
- 操作数定义: 以 `ValueSSA` 枚举为核心, 定义了标量常量等值语义不可追踪 Value 和指令等可追踪 Value.
- 数据流定义: 以 `Use | UseID | IUser | ITraceableValue` 为核心定义了整套 `def-use` 链的范式, 规范了指令、全局量等操作的发出者和接收者
- 控制流定义: 以 `BlockID | JumpTarget | ITerminatorInst` 等为核心, 定义了与 `def-use` 相似且平行的控制流系统.

### 优化器 (Opt)

Remusys-IR 至今仍然没有完整的优化管理器, `opt` 模块仅仅是少数 IR 变换规则的罗列而已.

已经实现的分析规则有:

- 支配树

已经实现的变换规则有:

- Mem2Reg
- 保守的 DCE

### 后端 (MIR)

Remusys-IR 在竞赛阶段有 MIR (参见 `old-with-slab` 分支) 用于后端表示、后端优化, 但代码质量不佳, 已经被移除。目前没有合适的 MIR 构建思路，故不实现之.

现在的 Remusys-IR 是一门没有后端的中间代码, 当前做验证的方式是利用 Remusys-IR Text 与 LLVM IR Text 的交集, 把 IR 转换成 LLVM-compatible 的文本交给 LLVM 做验证.

## 功能列表

参见 [TODO](TODOLIST.md).
