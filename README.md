# Remusys-IR: Recrafted LLVM-like IR system

**Languages:** [English](README.md) | [中文](README-zh_CN.md)

A CSCC 2025 competition project: an LLVM-like intermediate representation system written in Rust, redesigned from Musys IR. Includes a competitive AArch64 RIG backend.

## ⚠️ Warning

**This project is experimental and intended for learning and research purposes only. Do not use it in production environments!**

**The project is in early development/testing stage with no API stability guarantees. Code architecture, interfaces, and implementations may undergo breaking changes at any time. Users are responsible for any consequences.**

## Build Guide

As an experimental project, this library is not published to crates.io—maybe when Cargo supports user-based project categorization like GitHub. To add `remusys-ir` as a dependency in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
remusys-ir = { git = "https://github.com/medihbt/remusys-ir" }

# Or specify a specific branch
remusys-ir = { git = "https://github.com/medihbt/remusys-ir", branch = "master" }

# Or specify a specific tag/version
remusys-ir = { git = "https://github.com/medihbt/remusys-ir", tag = "v0.1.0" }

# Or specify a specific commit
remusys-ir = { git = "https://github.com/medihbt/remusys-ir", rev = "commit-hash" }
```

This project currently has no FFI bindings. In the future, if time permits and the API stabilizes, I may implement GObject bindings.

## Technical Architecture

### Intermediate Representation (IR)

LLVM-like intermediate representation providing a complete framework for data flow and control flow analysis. The IR module includes the following components:

- Type system: Defines the types, type relationships, and type storage used throughout the entire IR system.
- Operand definitions: Centered around the `ValueSSA` enum, defining scalar constants and other value semantics as untraceable Values, and instructions and other traceable Values.
- Data flow definitions: Using `Use | UseID | IUser | ITraceableValue` as the core, defines a complete `def-use` chain paradigm, standardizing the producers and consumers of instructions, globals, and other operations.
- Control flow definitions: Centered around `BlockID | JumpTarget | ITerminatorInst`, etc., defines a control flow system similar to and parallel to `def-use`.

### Optimizer (Opt)

Remusys-IR still lacks a complete optimization manager; the `opt` module is merely a collection of a few IR transformation rules.

Implemented analysis rules include:

- Dominator tree

Implemented transformation rules include:

- Mem2Reg
- Conservative DCE

### Backend (MIR)

During the competition phase, Remusys-IR had MIR (see the `old-with-slab` branch) for backend representation and backend optimization, but the code quality was poor and has been removed. Currently, there is no suitable MIR construction approach, so it is not implemented.

The current Remusys-IR is an intermediate representation without a backend. The current validation method is to utilize the intersection of Remusys-IR Text and LLVM IR Text, converting the IR to LLVM-compatible text to be validated by LLVM.

## Feature List

See [TODO](TODOLIST.md).