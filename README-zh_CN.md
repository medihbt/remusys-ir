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

- 操作数定义 -- 操作数、常量
- 指令+数据流定义
- 基本块+控制流定义

### 优化器 (Opt)

已经实现了部分分析工具. 完整的优化器等待实现中...

### 后端 (MIR)

使用 [Remusys InstGen DSL (RIG)](https://codeberg.org/medihbt/remusys-instgen) 定义指令结构, 实现了大部分 AArch64 基础部分指令.

## 功能列表

参见 [TODO](TODOLIST.md).
