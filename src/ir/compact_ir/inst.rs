use super::{BlockID, Value};
use crate::{
    ir::{AmoOrdering, CmpCond, Opcode, SyncScope},
    typing::ValTypeID,
};

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
    Switch(Value, BlockID, Box<[(i128, BlockID)]>),

    /// 在栈上分配一段固定大小的内存
    Alloca(ValTypeID, u8),

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
    Load(ValTypeID, Value, u8),

    /// 存储寄存器中的值到内存
    Store(ValTypeID, Value, Value, u8),

    /// 原子读-修改-写指令
    AmoRmw(CAmoRmw),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CAmoRmw {
    pub valtype: ValTypeID,
    pub opcode: Opcode,
    pub ptr: Value,
    pub val: Value,
    pub scope: SyncScope,
    pub ordering: AmoOrdering,
}
