use crate::{
    base::{APInt, SlabRef},
    typing::id::ValTypeID,
};
use std::{
    fmt::Debug,
    hash::Hash,
    num::NonZero,
    rc::{Rc, Weak},
};

mod block;
mod cmp_cond;
mod constant;
mod global;
mod module;
mod opcode;
mod utils;

pub mod checking;
pub mod graph_traits;
pub mod inst;

pub use self::{
    block::{
        BlockData, BlockDataInner, BlockRef,
        jump_target::{
            ITerminatorInst, ITerminatorRef, JumpTarget, JumpTargetKind, JumpTargetSplitter,
            PredList, TerminatorDataRef, TerminatorRef,
        },
    },
    cmp_cond::CmpCond,
    constant::{
        data::ConstData,
        expr::{Array, ConstExprData, ConstExprRef, Struct},
    },
    global::{
        GlobalData, GlobalDataCommon, GlobalKind, GlobalRef, ISubGlobal,
        func::{Func, FuncArg, FuncArgRef, FuncRef, FuncStorage, FuncUser},
        var::{Var, VarInner},
    },
    inst::{
        ISubInst, InstCommon, InstData, InstInner, InstRef,
        usedef::{ITraceableValue, Use, UseKind, UserIter, UserList},
    },
    module::{
        IRAllocs, IRAllocsRef, Module,
        gc::{IRLiveValueSet, IRValueMarker},
    },
    opcode::{InstKind, Opcode},
    utils::{
        builder::{
            IRBuilder, IRBuilderError, IRBuilderExpandedFocus, IRBuilderFocus,
            IRBuilderFocusCheckOption,
        },
        numbering::{IRValueNumberMap, NumberOption},
        writer::{IRWriter, write_ir_module},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueSSA {
    None,

    /// 常量数据, 包括整数、浮点数、零值等
    ConstData(ConstData),

    /// 常量表达式, 包括数组、结构体
    ConstExpr(ConstExprRef),

    /// 函数参数, 包含函数引用和参数索引
    FuncArg(GlobalRef, u32),

    /// 基本块引用, 用于控制流图中的块
    Block(BlockRef),

    /// 指令引用, 包括终结指令和其他指令
    Inst(InstRef),

    /// 全局变量引用, 包括函数和全局变量
    Global(GlobalRef),
}

impl ISubValueSSA for ValueSSA {
    fn try_from_ir(value: &ValueSSA) -> Option<&Self> {
        Some(value)
    }
    fn into_ir(self) -> ValueSSA {
        self
    }

    fn is_zero(&self, allocs: &IRAllocs) -> bool {
        match self {
            ValueSSA::ConstData(data) => data.is_zero(),
            ValueSSA::ConstExpr(expr) => expr.is_zero(allocs),
            _ => false,
        }
    }

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID {
        match self {
            ValueSSA::None => ValTypeID::Void,
            ValueSSA::ConstData(data) => data.get_valtype(allocs),
            ValueSSA::ConstExpr(expr) => expr.get_valtype(allocs),
            ValueSSA::FuncArg(func, id) => {
                FuncArgRef(func, id as usize).get_valtype(&allocs.globals)
            }
            ValueSSA::Block(_) => ValTypeID::Void,
            ValueSSA::Inst(inst_ref) => inst_ref.get_valtype(allocs),
            ValueSSA::Global(_) => ValTypeID::Ptr, // Global references are treated as pointers
        }
    }

    fn try_gettype_noalloc(self) -> Option<ValTypeID> {
        match self {
            ValueSSA::ConstData(data) => data.try_gettype_noalloc(),
            ValueSSA::Global(_) => Some(ValTypeID::Ptr),
            ValueSSA::Block(_) => Some(ValTypeID::Void),
            _ => None,
        }
    }

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        match self {
            ValueSSA::None => writer.write_str("none"),
            ValueSSA::ConstData(data) => data.fmt_ir(writer),
            ValueSSA::ConstExpr(expr) => expr.fmt_ir(writer),
            ValueSSA::FuncArg(_, id) => write!(writer.output.borrow_mut(), "%{id}"),
            ValueSSA::Block(block) => block.fmt_ir(writer),
            ValueSSA::Inst(inst) => inst.fmt_ir(writer),
            ValueSSA::Global(global) => global.fmt_ir(writer),
        }
    }
}

impl From<APInt> for ValueSSA {
    fn from(value: APInt) -> Self {
        ValueSSA::ConstData(ConstData::Int(value.bits(), value.as_unsigned()))
    }
}

impl TryInto<APInt> for ValueSSA {
    type Error = ValueSSAError;

    fn try_into(self) -> Result<APInt, Self::Error> {
        match self {
            ValueSSA::ConstData(x) => match x.as_apint() {
                Some(apint) => Ok(apint),
                None => Err(ValueSSAError::KindNotMatch(
                    self,
                    ValueSSA::ConstData(ConstData::Int(0, 0)),
                )),
            },
            _ => Err(ValueSSAError::KindNotMatch(
                self,
                ValueSSA::ConstData(ConstData::Int(0, 0)),
            )),
        }
    }
}

impl ValueSSA {
    /// 检查该 Value 是否能被追踪
    pub fn partial_traceable(&self) -> bool {
        !matches!(self, Self::None | Self::ConstData(_))
    }

    /// 检查该 Value 是否可以获得完整的使用者信息.
    pub fn traceable(&self) -> bool {
        use ValueSSA::*;
        matches!(self, FuncArg(..) | Block(_) | Inst(_) | Global(_))
    }

    /// 获取该 Value 的使用者列表
    pub fn users(self, allocs: &IRAllocs) -> Option<&UserList> {
        match self {
            ValueSSA::FuncArg(func, id) => {
                let users = FuncArgRef(func, id as usize).get_users(&allocs.globals);
                Some(users)
            }
            ValueSSA::Block(block) => Some(&block.to_data(&allocs.blocks).users()),
            ValueSSA::Inst(inst_ref) => Some(inst_ref.to_data(&allocs.insts).users()),
            ValueSSA::Global(global) => Some(global.to_data(&allocs.globals).users()),
            _ => None,
        }
    }

    pub(crate) fn add_user_rc(self, allocs: &IRAllocs, user: &Rc<Use>) -> bool {
        self.add_user(allocs, Rc::downgrade(user))
    }
    pub(crate) fn add_user(self, allocs: &IRAllocs, user: Weak<Use>) -> bool {
        match self {
            ValueSSA::FuncArg(func, id) => {
                FuncArgRef(func, id as usize)
                    .to_data(&allocs.globals)
                    .add_user(user);
                true
            }
            ValueSSA::Block(block) => {
                block.to_data(&allocs.blocks).add_user(user);
                true
            }
            ValueSSA::Inst(inst_ref) => {
                inst_ref.to_data(&allocs.insts).add_user(user);
                true
            }
            ValueSSA::Global(global) => {
                global.to_data(&allocs.globals).add_user(user);
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ValueSSAError {
    IDNotEqual(ValueSSA, ValueSSA),
    KindNotMatch(ValueSSA, ValueSSA),
    NotFunction(ValueSSA),
}

pub trait ISubValueSSA: Debug + Clone + PartialEq + Eq + Hash {
    fn try_from_ir(value: &ValueSSA) -> Option<&Self>;
    fn from_ir(value: &ValueSSA) -> &Self {
        Self::try_from_ir(value).expect("ValueSSA type mismatch")
    }
    fn into_ir(self) -> ValueSSA;

    fn is_zero(&self, allocs: &IRAllocs) -> bool;

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID;
    fn try_gettype_noalloc(self) -> Option<ValTypeID>;

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()>;
}

/// Trait for types that store pointer information.
/// Implementors of this trait can provide information about the type pointed to.
pub trait PtrStorage {
    /// Gets the type of the value being pointed to.
    ///
    /// # Returns
    /// The value type ID of the pointee type.
    fn get_stored_pointee_type(&self) -> ValTypeID;

    /// Gets the align of the value begin pointed to.
    fn get_stored_pointee_align(&self) -> Option<NonZero<usize>>;
}

/// Trait for types that use pointers as operands.
/// Implementors of this trait can retrieve type information about the pointee.
pub trait PtrUser {
    /// Gets the type of the value pointed to by an operand.
    ///
    /// # Returns
    /// The value type ID of the pointee.
    fn get_operand_pointee_type(&self) -> ValTypeID;

    /// Gets the align of this value user.
    fn get_operand_align(&self) -> Option<NonZero<usize>>;
}
