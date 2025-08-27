//! # 中间代码子系统
//!
//! 中间代码子系统提供了类似 LLVM IR 的中间表示形式、数据流与控制流分析系统, 用于程序的分析和优化.

use crate::{
    base::{APInt, INullableValue, SlabRef},
    typing::{AggrType, FPKind, IValType, ScalarType, ValTypeID},
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
mod managed;
mod module;
mod opcode;
mod user;
mod utils;

pub mod checking;
pub mod compact_ir;
pub mod inst;

pub use self::{
    block::{
        BlockData, BlockDataInner, BlockRef,
        jump_target::{
            ITerminatorInst, ITerminatorRef, JumpTarget, JumpTargetKind, JumpTargetSplitter,
            JumpTargets, PredList, TerminatorDataRef, TerminatorRef,
        },
    },
    checking::{ValueCheckError, inst_check::InstCheckCtx},
    cmp_cond::CmpCond,
    constant::{
        array::Array,
        data::ConstData,
        expr::{ConstExprData, ExprCommon, ExprRef, ISubExpr},
        structure::Struct,
        vec::FixVec,
    },
    global::{
        GlobalData, GlobalDataCommon, GlobalKind, GlobalRef, ISubGlobal, Linkage,
        func::{Func, FuncArg, FuncArgRef, FuncRef, FuncStorage, FuncUser},
        var::{Var, VarInner},
    },
    inst::{
        AmoOrdering, ISubInst, ISubInstRef, InstCommon, InstData, InstInner, InstRef, SyncScope,
        usedef::{ITraceableValue, Use, UseKind, UserIter, UserList},
    },
    managed::{IManagedIRValue, IRManaged, ManagedInst},
    module::{
        IModuleEditable, IModuleReadable, Module, IRModuleCleaner,
        allocs::*,
        gc::{IRLiveValueSet, IRValueMarker, IRValueCompactMap},
        view::*,
    },
    opcode::{InstKind, Opcode},
    user::{IUser, IUserRef, OperandSet, UserID},
    utils::{
        builder::{
            IRBuilder, IRBuilderError, IRBuilderExpandedFocus, IRBuilderFocus,
            IRBuilderFocusCheckOption, inst_builders::*,
        },
        numbering::{IRValueNumberMap, NumberOption},
        writer::{IRWriter, IRWriterOption, write_ir_module, write_ir_module_quiet},
    },
};

/// # 操作数定义
///
/// 和 LLVM IR 一样, Remusys IR 一切可追踪对象皆 Value. ValueSSA 可以是
/// 常量、指令、基本块、函数参数、全局变量等, 其中全局量、指令等引用类型采用
/// 实体内存池方式存储, 这里仅仅存储它们的引用, 其他值类型直接内联在 ValueSSA 里.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueSSA {
    None,

    /// 常量数据, 包括整数、浮点数、零值等
    ConstData(ConstData),

    /// 常量表达式, 包括数组、结构体
    ConstExpr(ExprRef),

    /// 常量 0 表达式
    AggrZero(AggrType),

    /// 函数参数, 包含函数引用和参数索引
    FuncArg(GlobalRef, u32),

    /// 基本块引用, 用于控制流图中的块
    Block(BlockRef),

    /// 指令引用, 包括终结指令和其他指令
    Inst(InstRef),

    /// 全局变量引用, 包括函数和全局变量
    Global(GlobalRef),
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueSSAClass {
    None,
    ConstData, ConstExpr, AggrZero,
    FuncArg, Block, Inst, Global,
}

impl INullableValue for ValueSSA {
    fn new_null() -> Self {
        ValueSSA::None
    }

    fn is_null(&self) -> bool {
        match self {
            ValueSSA::None => true,
            ValueSSA::ConstData(_) => false,
            ValueSSA::ConstExpr(x) => x.is_null(),
            ValueSSA::AggrZero(x) => x.is_null(),
            ValueSSA::FuncArg(x, _) => x.is_null(),
            ValueSSA::Block(x) => x.is_null(),
            ValueSSA::Inst(x) => x.is_null(),
            ValueSSA::Global(x) => x.is_null(),
        }
    }
}

impl ISubValueSSA for ValueSSA {
    fn try_from_ir(value: ValueSSA) -> Option<Self> {
        Some(value)
    }
    fn into_ir(self) -> ValueSSA {
        self
    }

    fn is_zero(&self, allocs: &IRAllocs) -> bool {
        match self {
            ValueSSA::ConstData(data) => data.is_zero(),
            ValueSSA::ConstExpr(expr) => expr.is_zero(allocs),
            ValueSSA::AggrZero(_) => true,
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
            ValueSSA::AggrZero(aggr) => aggr.into_ir(),
        }
    }

    fn try_gettype_noalloc(self) -> Option<ValTypeID> {
        match self {
            ValueSSA::ConstData(data) => data.try_gettype_noalloc(),
            ValueSSA::Global(_) => Some(ValTypeID::Ptr),
            ValueSSA::Block(_) => Some(ValTypeID::Void),
            ValueSSA::AggrZero(aggr) => Some(aggr.into_ir()),
            _ => None,
        }
    }

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        match self {
            ValueSSA::None => writer.write_str("none"),
            ValueSSA::ConstData(data) => data.fmt_ir(writer),
            ValueSSA::ConstExpr(expr) => expr.fmt_ir(writer),
            ValueSSA::AggrZero(_) => writer.write_str("zeroinitializer"),
            ValueSSA::FuncArg(_, id) => write!(writer.output.borrow_mut(), "%{id}"),
            ValueSSA::Block(block) => block.fmt_ir(writer),
            ValueSSA::Inst(inst) => inst.fmt_ir(writer),
            ValueSSA::Global(global) => global.fmt_ir(writer),
        }
    }
}

impl From<APInt> for ValueSSA {
    fn from(value: APInt) -> Self {
        ValueSSA::ConstData(ConstData::Int(value))
    }
}

impl From<f32> for ValueSSA {
    fn from(value: f32) -> Self {
        ValueSSA::ConstData(ConstData::Float(FPKind::Ieee32, value as f64))
    }
}

impl From<f64> for ValueSSA {
    fn from(value: f64) -> Self {
        ValueSSA::ConstData(ConstData::Float(FPKind::Ieee64, value))
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
                    ValueSSA::ConstData(ConstData::Int(0.into())),
                )),
            },
            _ => Err(ValueSSAError::KindNotMatch(
                self,
                ValueSSA::ConstData(ConstData::Int(0.into())),
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

    /// 从整数值创建一个 ValueSSA
    pub fn from_int<T: Into<APInt>>(value: T) -> Self {
        ValueSSA::ConstData(ConstData::Int(value.into()))
    }

    pub fn as_int(self) -> Option<APInt> {
        self.try_into().ok()
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

    pub fn new_zero(ty: ValTypeID) -> ValueSSA {
        match ty {
            ValTypeID::Ptr => Self::ConstData(ConstData::Zero(ScalarType::Ptr)),
            ValTypeID::Int(bits) => Self::ConstData(ConstData::Int(APInt::new(0, bits))),
            ValTypeID::Float(fpkind) => Self::ConstData(ConstData::Float(fpkind, 0.0)),
            ValTypeID::Array(aggr) => Self::AggrZero(aggr.into()),
            ValTypeID::Struct(aggr) => Self::AggrZero(aggr.into()),
            ValTypeID::StructAlias(aggr) => Self::AggrZero(aggr.into()),
            ValTypeID::FixVec(fv) => Self::AggrZero(fv.into()),
            ValTypeID::Void | ValTypeID::Func(_) => {
                panic!("Cannot create zero value for void or function types")
            }
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
    fn try_from_ir(value: ValueSSA) -> Option<Self>;
    fn from_ir(value: ValueSSA) -> Self {
        if let Some(val) = Self::try_from_ir(value) {
            val
        } else {
            let selfty = std::any::type_name::<Self>();
            panic!("ValueSSA kind mismatch: {value:?} but requires {selfty}",);
        }
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

pub trait IReferenceValue {
    type ValueDataT;

    fn to_value_data<'a>(self, allocs: &'a IRAllocs) -> &'a Self::ValueDataT
    where
        Self::ValueDataT: 'a;

    fn to_value_data_mut<'a>(self, allocs: &'a mut IRAllocs) -> &'a mut Self::ValueDataT
    where
        Self::ValueDataT: 'a;
}
