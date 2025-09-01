//! Compact IR 紧凑 IR 版本
//!
//! 为多线程传输适配的 IR. Remusys IR 的定义中到处都是 `Cell RefCell Rc` 这种见不得多线程的东西,
//! 因此专门设计适合多线程传输的版本.
//!
//! 本来想让它适配 Bump Allocator 等内存池的, 可惜 Rust Allocator 还不是 stable feature,
//! 所以就只能先做 global allocator 版本的了。
//!
//! ## 未完成提醒
//!
//! 现在类型系统还在沿用主机的, 但主机的 TypeContext 没有并行能力。现在要么改造 TypeContext, 要么
//! 多做一套适合传输的类型映射.

use crate::{
    ir::{ConstData, Linkage},
    typing::{AggrType, ArrayTypeRef, FuncTypeRef, ValTypeID},
};
use std::{marker::PhantomData, ops::Range};

mod inst;

pub use inst::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    /// 空值，表示无效或未初始化的值
    None,
    /// 常量数据，包括整数、浮点数、零值等基础常量
    Data(ConstData),
    /// 常量表达式引用，表示复合常量（数组、结构体等）的索引
    Expr(ExprID),
    /// 聚合类型的零值表达式，用于数组和结构体的零初始化
    AggrZero(AggrType),
    /// 函数参数 `(func_id, arg_id)`
    FuncArg(GlobalID, u32),
    /// 基本块引用，表示控制流图中基本块的索引
    Block(BlockID),
    /// 指令引用，表示 IR 指令的索引
    Inst(InstID),
    /// 全局对象引用，表示全局变量或函数的索引
    Global(GlobalID),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockID(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalID(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExprID(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstID(u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Array(ArrayTypeRef, Box<[Value]>),
    Struct(AggrType, Box<[Value]>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Global {
    Var(Variable),
    Func(Func),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub ty: ValTypeID,
    pub linkage: Linkage,
    pub initval: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexRange<T>(Range<u32>, PhantomData<T>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Func {
    pub name: String,
    pub func_ty: FuncTypeRef,
    pub blocks: IndexRange<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub insts: IndexRange<inst::Inst>,
}

#[derive(Debug, Clone)]
pub struct CompactIR {
    pub globals: Vec<Global>,
    pub exprs: Vec<Expr>,
    pub blocks: Vec<Block>,
    pub insts: Vec<inst::Inst>,
}
