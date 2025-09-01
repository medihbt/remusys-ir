use std::collections::BTreeSet;

use thiserror::Error;

use crate::{
    ir::{
        BlockRef, FuncRef, IRAllocs, ISubValueSSA, InstKind, JumpTargetKind, Opcode, UseKind,
        ValueSSA, ValueSSAClass, inst::*,
    },
    typing::{IValType, ValTypeClass, ValTypeID},
};

pub(super) mod inst_check;

#[derive(Debug, Clone, Error)]
pub enum BlockLayoutError {
    #[error("Phi instruction {0:?} not in head of basic block")]
    PhiNotInHead(PhiRef),
    #[error("Dirty phi section with non-phi instruction {0:?}")]
    DirtyPhiSection(InstRef),
    #[error("Multiple terminator instructions, found {0:?}")]
    MultipleTerminator(InstRef),
}

#[derive(Debug, Clone, Error)]
pub enum FuncLayoutError {
    #[error("Entry block not in front for function {0:?}")]
    EntryNotInFront(FuncRef),
}

#[derive(Debug, Clone, Error)]
pub enum ValueCheckError {
    #[error("Type mismatch: requires {0:?} but got {1:?}")]
    TypeMismatch(ValTypeID, ValTypeID),
    #[error("Type {0:?} not in class {1:?}")]
    TypeNotClass(ValTypeID, ValTypeClass),
    #[error("Type {0:?} is not sized")]
    TypeNotSized(ValTypeID),

    #[error("Operand type mismatch in instruction {0:?} at {1:?}: requires {2:?} but got {3:?}")]
    OpTypeMismatch(InstRef, UseKind, ValTypeID, ValTypeID),
    #[error("Operand type {2:?} not in class {3:?} for instruction {0:?} at {1:?}")]
    OpTypeNotClass(InstRef, UseKind, ValTypeID, ValTypeClass),
    #[error("Operand type {2:?} is not sized for instruction {0:?} at {1:?}")]
    OpTypeNotSized(InstRef, UseKind, ValTypeID),

    #[error("Instruction type mismatch in {0:?}: requires {1:?} but got {2:?}")]
    InstTypeMismatch(InstRef, ValTypeID, ValTypeID),
    #[error("Instruction type {1:?} not in class {2:?} for {0:?}")]
    InstTypeNotClass(InstRef, ValTypeID, ValTypeClass),
    #[error("Instruction type {1:?} is not sized for {0:?}")]
    InstTypeNotSized(InstRef, ValTypeID),

    #[error("Invalid value {0:?}: {1}")]
    InvalidValue(ValueSSA, String),
    #[error("Invalid zero operand {0:?} for opcode {1:?} at {2:?}")]
    InvalidZeroOP(ValueSSA, Opcode, UseKind),
    #[error("Value {0:?} is not of class {1:?}")]
    ValueNotClass(ValueSSA, ValueSSAClass),

    #[error("Wrong opcode {1:?} for instruction kind {0:?}")]
    FalseOpcodeKind(InstKind, Opcode),
    #[error("Missing operand for instruction {0:?} at position {1:?}")]
    OperandPosNone(InstRef, UseKind),
    #[error("Missing jump target for instruction {0:?} of kind {1:?}")]
    JumpTargetNone(InstRef, JumpTargetKind),

    #[error("Duplicated switch case {1:?} in instruction {0:?}")]
    DuplicatedSwitchCase(InstRef, JumpTargetKind),
    #[error("Phi incoming set mismatch in {0:?}: expected {1:?} but got {2:?}")]
    PhiIncomeSetUnmatch(InstRef, BTreeSet<BlockRef>, BTreeSet<BlockRef>),
    #[error("Call argument count mismatch in {0:?}: requires {1} but got {2}")]
    CallArgCountUnmatch(InstRef, u32 /* require */, u32 /* real */),
    #[error("Cast operation mismatch in {0:?}: cannot cast {2:?} to {3:?} with opcode {1:?}")]
    CastUnmatch(InstRef, Opcode, ValTypeID, ValTypeID),
    #[error("Compare opcode error in {0:?}: opcode {1:?} not supported for type {2:?}")]
    CmpOpcodeErr(InstRef, Opcode, ValTypeID),

    #[error("Block layout error: {0:?}")]
    BlockLayoutError(#[from] BlockLayoutError),
    #[error("Function layout error: {0:?}")]
    FuncLayoutError(#[from] FuncLayoutError),

    #[error("{0}")]
    Other(&'static str),
    #[error("{0}")]
    OtherFull(String),
}

pub(super) fn type_matches(
    ty: ValTypeID,
    val: ValueSSA,
    allocs: &IRAllocs,
) -> Result<(), ValueCheckError> {
    let valty: ValTypeID = val.get_valtype(allocs);
    if valty != ty { Err(ValueCheckError::TypeMismatch(ty, valty)) } else { Ok(()) }
}

pub(super) fn optype_matches(
    inst: InstRef,
    ukind: UseKind,
    ty: ValTypeID,
    val: ValueSSA,
    allocs: &IRAllocs,
) -> Result<(), ValueCheckError> {
    let valty: ValTypeID = val.get_valtype(allocs);
    if valty != ty { Err(ValueCheckError::OpTypeMismatch(inst, ukind, ty, valty)) } else { Ok(()) }
}

pub(super) fn _type_isclass(
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

pub(super) fn optype_isclass(
    inst: InstRef,
    ukind: UseKind,
    klass: ValTypeClass,
    val: ValueSSA,
    allocs: &IRAllocs,
) -> Result<(), ValueCheckError> {
    let val_class = val.get_valtype(allocs).class_id();
    if val_class != klass {
        Err(ValueCheckError::OpTypeNotClass(
            inst,
            ukind,
            val.get_valtype(allocs),
            val_class,
        ))
    } else {
        Ok(())
    }
}
