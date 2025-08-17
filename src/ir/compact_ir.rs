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

use std::{marker::PhantomData, ops::Range};

use crate::{
    ir::{CmpCond, ConstData, Linkage, Opcode},
    typing::{AggrType, ArrayTypeRef, FuncTypeRef, ValTypeID},
};

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
pub enum Inst {
    /// Phi 指令，根据前驱基本块选择一个值
    Phi(ValTypeID, Box<[(BlockID, Value)]>),

    /// 终止函数控制流并返回一个值
    Ret(ValTypeID, Value),

    /// 终止函数控制流，不返回值 (void 函数)
    RetVoid,

    /// 表示 "所在基本块不可达"，封死整个基本块的控制流
    Unreachable,

    /// 无条件跳转到指定基本块
    Jump(BlockID),

    /// 条件分支指令，根据条件跳转到不同的基本块
    Br(Value, BlockID, BlockID),

    /// Switch 语句，根据条件跳转到不同的 case 分支
    Switch(Value, Box<[(i128, BlockID)]>),

    /// 在栈上分配一段固定大小的内存
    Alloca(ValTypeID),

    /// 二元操作指令
    BinOp(ValTypeID, Opcode, Value, Value),

    /// 函数调用指令
    Call(ValTypeID, Value, Box<[Value]>),

    /// 类型转换指令
    Cast(ValTypeID, Opcode, Value),

    /// 比较两个值的关系，产生一个布尔值
    Cmp(CmpCond, Value, Value),

    /// 根据索引计算指针偏移，用于数组或结构体访问
    GEP(ValTypeID, Value, Box<[Value]>),

    /// 选择指令，根据条件选择两个值中的一个
    Select(ValTypeID, Value, Value, Value),

    /// 加载内存中的值到寄存器
    Load(ValTypeID, Value),

    /// 存储寄存器中的值到内存
    Store(Value, Value),
}

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
    pub insts: IndexRange<Inst>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactIR {
    pub globals: Vec<Global>,
    pub exprs: Vec<Expr>,
    pub blocks: Vec<Block>,
    pub insts: Vec<Inst>,
}
