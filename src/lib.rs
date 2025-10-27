//! # Remusys-IR 类 LLVM 中间代码系统.
//!
//! Copyright (c) 2025 Medi H.B.T.

pub use {slab, thiserror};

pub mod base;
pub mod ir;
pub mod mir;
pub mod opt;
pub mod testing;
pub mod typing;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

pub const MTBLIB_PKG_NAME: &str = "io.medihbt.Remusys.IR";
