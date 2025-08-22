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

- Operand definitions -- operands and constants
- Instruction + data flow definitions
- Basic block + control flow definitions

### Optimizer (Opt)

Partial analysis tools have been implemented. A complete optimizer is waiting to be implemented...

### Backend (MIR)

Uses [Remusys InstGen DSL (RIG)](https://codeberg.org/medihbt/remusys-instgen) to define instruction structures, implementing most AArch64 basic instruction sets.

## Feature List

See [TODO](TODOLIST.md).
