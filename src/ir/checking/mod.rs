use std::collections::BTreeSet;

use crate::{
    ir::{
        BlockRef, IRAllocs, ISubValueSSA, InstKind, JumpTargetKind, Opcode, UseKind, ValueSSA,
        ValueSSAClass, inst::*,
    },
    typing::{IValType, ValTypeClass, ValTypeID},
};

pub(super) mod inst_check;

/// 生成操作数为空的错误

#[derive(Debug, Clone)]
pub enum ValueCheckError {
    TypeMismatch(ValTypeID, ValTypeID),
    TypeNotClass(ValTypeID, ValTypeClass),
    TypeNotSized(ValTypeID),

    InvalidValue(ValueSSA, String),
    InvalidZeroOP(ValueSSA, Opcode, UseKind),
    ValueNotClass(ValueSSA, ValueSSAClass),

    FalseOpcodeKind(InstKind, Opcode),
    OperandPosNone(InstRef, UseKind),
    JumpTargetNone(InstRef, JumpTargetKind),

    DuplicatedSwitchCase(InstRef, JumpTargetKind),
    PhiIncomeSetUnmatch(InstRef, BTreeSet<BlockRef>, BTreeSet<BlockRef>),
    CallArgCountUnmatch(InstRef, u32 /* require */, u32 /* real */),
    CastUnmatch(InstRef, Opcode, ValTypeID, ValTypeID),
    CmpOpcodeErr(InstRef, Opcode, ValTypeID),

    Other(&'static str),
    OtherFull(String),
}

impl std::fmt::Display for ValueCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{:?}", self)
        match self {
            ValueCheckError::TypeMismatch(req, real) => {
                write!(f, "TypeMismatch {{ required: {req:?}, real: {real:?} }}")
            }
            ValueCheckError::TypeNotClass(real, req) => {
                write!(f, "TypeNotClass {{ klass: {req:?}, real: {real:?} }}")
            }
            ValueCheckError::TypeNotSized(ty) => {
                write!(f, "TypeNotSized {{ ty: {ty:?} }}")
            }
            ValueCheckError::InvalidValue(val, msg) => {
                write!(f, "InvalidValue {{ value: {val:?}, msg: {msg} }}")
            }
            ValueCheckError::InvalidZeroOP(operand, opcode, use_kind) => {
                write!(
                    f,
                    "InvalidZeroOP {{ operand: {operand:?}, opcode: {opcode:?}, use_kind: {use_kind:?} }}"
                )
            }
            ValueCheckError::ValueNotClass(operand, klass) => {
                write!(
                    f,
                    "ValueNotClass {{ operand: {operand:?}, klass: {klass:?} }}"
                )
            }
            ValueCheckError::FalseOpcodeKind(inst_kind, opcode) => {
                write!(
                    f,
                    "FalseOpcodeKind {{ requires: {inst_kind:?}, opcode: {opcode:?} }}"
                )
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

pub(super) fn type_matches(
    ty: ValTypeID,
    val: ValueSSA,
    allocs: &IRAllocs,
) -> Result<(), ValueCheckError> {
    let valty: ValTypeID = val.get_valtype(allocs);
    if valty != ty { Err(ValueCheckError::TypeMismatch(ty, valty)) } else { Ok(()) }
}

pub(super) fn type_isclass(
    klass: ValTypeClass,
    val: ValueSSA,
    allocs: &IRAllocs,
) -> Result<(), ValueCheckError> {
    let val_class = val.get_valtype(allocs).class_id();
    if val_class != klass {
        Err(ValueCheckError::TypeNotClass(
            val.get_valtype(allocs),
            klass,
        ))
    } else {
        Ok(())
    }
}
