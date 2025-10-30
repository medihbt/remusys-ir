mod attributes {}
mod block;
mod cmp_cond;
mod constant;
mod global;
mod managed {}
mod module;
mod opcode;
mod usedef;
mod utils {}

pub mod checking {}
pub mod inst;
pub mod snap_ir {}

use crate::typing::AggrType;

pub use self::{
    block::{BlockID, BlockObj},
    cmp_cond::CmpCond,
    constant::{
        array::ArrayExpr,
        data::ConstData,
        expr::{ExprID, ExprObj, ISubExpr, ISubExprID},
        structure::StructExpr,
    },
    global::{GlobalID, GlobalObj, ISubGlobal, ISubGlobalID, var::GlobalVar},
    inst::{ISubInst, ISubInstID, InstID, InstObj},
    module::{IPoolAllocated, IRAllocs},
    opcode::{InstKind, Opcode},
    usedef::{
        ITraceableValue, IUser, OperandSet, OperandUseIter, Use, UseID, UseIter, UseKind, UserID,
        UserList,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueClass {
    None,
    ConstData,
    ConstExpr,
    AggrZero,
    FuncArg,
    Block,
    Inst,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueSSA {
    None,
    ConstData(ConstData),
    ConstExpr(ExprID),
    AggrZero(AggrType),
    FuncArg(GlobalID, u32),
    Block(BlockID),
    Inst(InstID),
    Global(GlobalID),
}
