//! ## Remusys IR subsystem
//!
//! Core IR structures and utilities.

mod attributes;
mod block;
mod cmp_cond;
mod constant;
mod global;
mod jumping;
mod managed;
mod module;
mod opcode;
mod usedef;
mod utils;

pub mod checking {
    //! IR checking utilities.
}
pub mod inst;

use crate::{
    base::{APInt, INullableValue},
    typing::{AggrType, IValType, ScalarType, TypeMismatchError, ValTypeClass, ValTypeID},
};

pub use self::{
    block::{BlockID, BlockObj},
    cmp_cond::CmpCond,
    constant::{
        array::{ArrayExpr, ArrayExprID},
        data::ConstData,
        expr::{ExprCommon, ExprID, ExprObj, ISubExpr, ISubExprID},
        structure::{StructExpr, StructExprID},
        vec::{FixVec, FixVecID},
    },
    global::{
        GlobalCommon, GlobalID, GlobalKind, GlobalObj, ISubGlobal, ISubGlobalID,
        func::{
            FuncArg, FuncArgID, FuncBody, FuncBuilder, FuncID, FuncObj, FuncTerminateMode,
            IFuncUniqueUser, IFuncValue,
        },
        var::{GlobalVar, GlobalVarBuilder, GlobalVarID, IGlobalVarBuildable},
    },
    inst::{AmoOrdering, ISubInst, ISubInstID, InstCommon, InstID, InstObj, SyncScope},
    jumping::{
        ITerminatorID, ITerminatorInst, JumpTarget, JumpTargetID, JumpTargetKind, JumpTargets,
        JumpTargetsBlockIter, PredList, TerminatorID, TerminatorObj,
    },
    managed::{ManagedBlock, ManagedExpr, ManagedGlobal, ManagedInst, ManagedJT, ManagedUse},
    module::{
        Module,
        allocs::{
            IRAllocs, PoolAllocatedClass, PoolAllocatedDisposeErr, PoolAllocatedDisposeRes,
            PoolAllocatedID,
        },
        gc::{IRLiveSet, IRMarker},
    },
    opcode::{InstKind, Opcode},
    usedef::{
        ITraceableValue, IUser, OperandSet, OperandUseIter, Use, UseID, UseIter, UseKind, UserID,
        UserList,
    },
    utils::{
        builder::*,
        numbering::{IRNumberValueMap, NumberOption},
        writer::{IRWriteOption, IRWriter, IRWriterStat},
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

pub trait ISubValueSSA: Copy {
    fn get_class(self) -> ValueClass;
    fn try_from_ir(ir: ValueSSA) -> Option<Self>;
    fn into_ir(self) -> ValueSSA;
    fn from_ir(ir: ValueSSA) -> Self {
        match Self::try_from_ir(ir) {
            Some(v) => v,
            None => panic!(
                "Invalid ValueSSA type for {}",
                std::any::type_name::<Self>()
            ),
        }
    }

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID;

    fn can_trace(self) -> bool;
    fn try_get_users(self, allocs: &IRAllocs) -> Option<&UserList>;
    fn try_add_user(self, allocs: &IRAllocs, user_use: UseID) -> bool {
        let Some(users) = self.try_get_users(allocs) else {
            return false;
        };
        users
            .push_back_id(user_use.inner(), &allocs.uses)
            .expect("Failed to add User to ValueSSA users");
        true
    }

    fn is_zero_const(self, allocs: &IRAllocs) -> bool;
}
pub trait IPtrValue {
    fn get_ptr_pointee_type(&self) -> ValTypeID;
    fn get_ptr_pointee_align(&self) -> u32;
}
pub trait IPtrUniqueUser: IUser {
    fn get_operand_pointee_type(&self) -> ValTypeID;
    fn get_operand_pointee_align(&self) -> u32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueSSA {
    None,
    ConstData(ConstData),
    ConstExpr(ExprID),
    AggrZero(AggrType),
    FuncArg(FuncID, u32),
    Block(BlockID),
    Inst(InstID),
    Global(GlobalID),
}
impl From<APInt> for ValueSSA {
    fn from(value: APInt) -> Self {
        ValueSSA::ConstData(ConstData::Int(value))
    }
}
impl INullableValue for ValueSSA {
    fn new_null() -> Self {
        Self::None
    }
    fn is_null(&self) -> bool {
        matches!(self, Self::None)
    }
}
impl ISubValueSSA for ValueSSA {
    fn get_class(self) -> ValueClass {
        match self {
            Self::None => ValueClass::None,
            Self::ConstData(_) => ValueClass::ConstData,
            Self::ConstExpr(_) => ValueClass::ConstExpr,
            Self::AggrZero(_) => ValueClass::AggrZero,
            Self::FuncArg(..) => ValueClass::FuncArg,
            Self::Block(_) => ValueClass::Block,
            Self::Inst(_) => ValueClass::Inst,
            Self::Global(_) => ValueClass::Global,
        }
    }
    fn try_from_ir(ir: ValueSSA) -> Option<Self> {
        Some(ir)
    }
    fn into_ir(self) -> ValueSSA {
        self
    }

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID {
        use ValueSSA::*;
        match self {
            None => ValTypeID::Void,
            ConstData(data) => data.get_valtype(allocs),
            ConstExpr(expr) => expr.get_valtype(allocs),
            AggrZero(aggr) => aggr.into_ir(),
            FuncArg(func, id) => FuncArgID(func, id).get_valtype(allocs),
            Block(_) => ValTypeID::Void,
            Inst(inst) => inst.get_valtype(allocs),
            Global(_) => ValTypeID::Ptr,
        }
    }
    fn is_zero_const(self, allocs: &IRAllocs) -> bool {
        use ValueSSA::*;
        match self {
            ConstData(data) => data.is_zero_const(allocs),
            ConstExpr(expr) => expr.is_zero_const(allocs),
            AggrZero(_) => true,
            _ => false,
        }
    }

    fn can_trace(self) -> bool {
        use ValueSSA::*;
        matches!(self, ConstExpr(_) | Block(_) | Inst(_) | Global(_))
    }
    fn try_get_users(self, allocs: &IRAllocs) -> Option<&UserList> {
        use ValueSSA::*;
        match self {
            ConstExpr(expr) => expr.try_get_users(allocs),
            FuncArg(func, id) => FuncArgID(func, id).try_get_users(allocs),
            Block(block) => block.try_get_users(allocs),
            Inst(inst) => inst.try_get_users(allocs),
            Global(global) => global.try_get_users(allocs),
            _ => Option::None,
        }
    }
}
impl ValueSSA {
    pub fn new_zero(ty: ValTypeID) -> Result<Self, TypeMismatchError> {
        let val = match ty {
            ValTypeID::Void => ValueSSA::None,
            ValTypeID::Ptr => ValueSSA::ConstData(ConstData::PtrNull(ValTypeID::Void)),
            ValTypeID::Int(bits) => ValueSSA::ConstData(ConstData::Int(APInt::new(0, bits))),
            ValTypeID::Float(fpkind) => ValueSSA::ConstData(ConstData::Float(fpkind, 0.0)),
            ValTypeID::FixVec(v) => ValueSSA::AggrZero(AggrType::FixVec(v)),
            ValTypeID::Array(a) => ValueSSA::AggrZero(AggrType::Array(a)),
            ValTypeID::Struct(s) => ValueSSA::AggrZero(AggrType::Struct(s)),
            _ => return Err(TypeMismatchError::NotClass(ty, ValTypeClass::Compound)),
        };
        Ok(val)
    }

    pub fn as_dyn_traceable<'ir>(&self, allocs: &'ir IRAllocs) -> Option<&'ir dyn ITraceableValue> {
        match self {
            ValueSSA::ConstExpr(expr) => Some(expr.deref_ir(allocs)),
            ValueSSA::FuncArg(func, id) => Some(FuncArgID(*func, *id).deref_ir(allocs)),
            ValueSSA::Block(block) => Some(block.deref_ir(allocs)),
            ValueSSA::Inst(inst) => Some(inst.deref_ir(allocs)),
            ValueSSA::Global(global) => Some(global.deref_ir(allocs)),
            _ => None,
        }
    }
    pub fn as_dyn_user<'ir>(&self, allocs: &'ir IRAllocs) -> Option<&'ir dyn IUser> {
        match self {
            ValueSSA::Inst(inst) => Some(inst.deref_ir(allocs)),
            ValueSSA::Global(global) => Some(global.deref_ir(allocs)),
            ValueSSA::ConstExpr(expr) => Some(expr.deref_ir(allocs)),
            _ => None,
        }
    }
    pub fn as_dyn_ptrvalue<'ir>(&self, allocs: &'ir IRAllocs) -> Option<&'ir dyn IPtrValue> {
        match self {
            ValueSSA::Global(global) => Some(global.deref_ir(allocs)),
            ValueSSA::Inst(inst) => match inst.deref_ir(allocs) {
                InstObj::Alloca(i) => Some(i),
                InstObj::GEP(i) => Some(i),
                _ => None,
            },
            _ => None,
        }
    }
    pub fn as_dyn_ptruser<'ir>(&self, allocs: &'ir IRAllocs) -> Option<&'ir dyn IPtrUniqueUser> {
        match self {
            ValueSSA::Inst(inst) => match inst.deref_ir(allocs) {
                InstObj::Store(i) => Some(i),
                InstObj::Load(i) => Some(i),
                InstObj::AmoRmw(i) => Some(i),
                InstObj::Call(i) => Some(i),
                _ => None,
            },
            _ => None,
        }
    }
    pub fn as_apint(&self) -> Option<APInt> {
        match self {
            ValueSSA::ConstData(ConstData::Int(v)) => Some(*v),
            ValueSSA::ConstData(ConstData::Zero(ScalarType::Int(bits))) => {
                Some(APInt::new(0, *bits))
            }
            _ => None,
        }
    }
}
