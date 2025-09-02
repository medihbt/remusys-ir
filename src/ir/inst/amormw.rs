use crate::{
    ir::{
        IRAllocs, IRWriter, ISubInst, ISubInstRef, ISubValueSSA, IUser, InstCommon, InstData,
        InstKind, InstRef, Opcode, OperandSet, Use, UseKind, ValueSSA,
    },
    typing::ValTypeID,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmoOrdering {
    NonAtomic,
    Relaxed,
    Monotonic,
    Release,
    Acquire,
    /// Acquire and release
    AcqRel,
    /// Sequentially consistent
    SeqCst,
}

impl AmoOrdering {
    pub fn as_str(self) -> &'static str {
        match self {
            AmoOrdering::NonAtomic => "not_atomic",
            AmoOrdering::Relaxed => "relaxed",
            AmoOrdering::Monotonic => "monotonic",
            AmoOrdering::Release => "release",
            AmoOrdering::Acquire => "acquire",
            AmoOrdering::AcqRel => "acq_rel",
            AmoOrdering::SeqCst => "seq_cst",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncScope {
    SingleThread,
    System,
}

impl SyncScope {
    pub fn as_str(self) -> &'static str {
        match self {
            SyncScope::SingleThread => "singlethread",
            SyncScope::System => "system",
        }
    }
}

/// 原子操作: 读取-修改-写入
///
/// ### IR 语法
///
/// ```llvm
/// %id = atomicrmw [volatile] <operation> ptr <pointer>, <ty> <value> [syncscope("<target-scope>")] <ordering>[, align <alignment>]  ; yields ty
/// ```
#[derive(Debug, Clone)]
pub struct AmoRmw {
    common: InstCommon,
    operands: [Rc<Use>; 2],
    pub ordering: AmoOrdering,
    pub is_volatile: bool,
    pub align_log2: u8,
    pub scope: SyncScope,
}

impl IUser for AmoRmw {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }

    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.operands
    }
}

impl ISubInst for AmoRmw {
    fn new_empty(opcode: Opcode) -> Self {
        Self {
            common: InstCommon::new(opcode, ValTypeID::Void),
            operands: [Use::new(UseKind::AmoRmwPtr), Use::new(UseKind::AmoRmwVal)],
            ordering: AmoOrdering::NonAtomic,
            is_volatile: false,
            align_log2: 0,
            scope: SyncScope::System,
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        if let InstData::AmoRmw(x) = inst { Some(x) } else { None }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        if let InstData::AmoRmw(x) = inst { Some(x) } else { None }
    }
    fn into_ir(self) -> InstData {
        InstData::AmoRmw(self)
    }
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn is_terminator(&self) -> bool {
        false
    }
    /// ```llvm
    /// %id = atomicrmw [volatile] <operation> ptr <pointer>, <ty> <value> [syncscope("<target-scope>")] <ordering>[, align <alignment>]  ; yields ty
    /// ```
    fn fmt_ir(&self, id: Option<usize>, writer: &IRWriter) -> std::io::Result<()> {
        let Some(id) = id else {
            use std::io::{Error, ErrorKind::*};
            return Err(Error::new(Other, "Inst must have an ID to be printed"));
        };
        write!(writer, "%{id} = atomicrmw ")?;
        if self.is_volatile {
            writer.write_str("volatile ")?;
        }

        write!(writer, "{} ", Self::subop_get_name(self.common.opcode))?;

        writer.write_str("ptr ")?;
        writer.write_operand(self.get_pointer())?;

        let value = self.get_value();
        let valuety = self.get_valtype();
        debug_assert_eq!(value.get_valtype(&writer.allocs), valuety);
        writer.write_str(", ")?;
        writer.write_type(valuety)?;
        writer.write_operand(value)?;

        if self.scope != SyncScope::System {
            write!(writer, " syncscope(\"{}\") ", self.scope.as_str())?;
        }
        write!(
            writer,
            "{}, align {}",
            self.ordering.as_str(),
            1 << self.align_log2
        )?;

        Ok(())
    }
}

impl AmoRmw {
    fn subop_get_name(opcode: Opcode) -> &'static str {
        match opcode {
            Opcode::AmoXchg => "xchg",
            Opcode::AmoAdd => "add",
            Opcode::AmoSub => "sub",
            Opcode::AmoAnd => "and",
            Opcode::AmoNand => "nand",
            Opcode::AmoOr => "or",
            Opcode::AmoXor => "xor",
            Opcode::AmoSMax => "max",
            Opcode::AmoSMin => "min",
            Opcode::AmoUMax => "umax",
            Opcode::AmoUMin => "umin",
            Opcode::AmoFAdd => "fadd",
            Opcode::AmoFSub => "fsub",
            Opcode::AmoFMax => "fmax",
            Opcode::AmoFMin => "fmin",
            Opcode::AmoUIncWrap => "uinc_wrap",
            Opcode::AmoUDecWrap => "udec_wrap",
            Opcode::AmoUSubCond => "usub_cond",
            Opcode::AmoUSubStat => "usub_stat",
            _ => panic!("Invalid opcode for AmoRmw: {opcode:?}"),
        }
    }

    pub fn pointer_use(&self) -> &Rc<Use> {
        &self.operands[0]
    }
    pub fn get_pointer(&self) -> ValueSSA {
        self.pointer_use().get_operand()
    }
    pub fn set_pointer(&self, allocs: &IRAllocs, value: impl ISubValueSSA) {
        self.pointer_use().set_operand(allocs, value);
    }

    pub fn value_use(&self) -> &Rc<Use> {
        &self.operands[1]
    }
    pub fn get_value(&self) -> ValueSSA {
        self.value_use().get_operand()
    }
    pub fn set_value(&self, allocs: &IRAllocs, value: impl ISubValueSSA) {
        self.value_use().set_operand(allocs, value);
    }

    pub fn builder(opcode: Opcode, value_ty: ValTypeID) -> AmoRmwBuilder {
        AmoRmwBuilder::new(opcode, value_ty)
    }
}

#[derive(Debug, Clone)]
pub struct AmoRmwBuilder {
    opcode: Opcode,
    value_ty: ValTypeID,
    ordering: AmoOrdering,
    is_volatile: bool,
    align_log2: u8,
    scope: SyncScope,
    ptr_operand: ValueSSA,
    val_operand: ValueSSA,
}

impl AmoRmwBuilder {
    pub fn new(opcode: Opcode, value_ty: ValTypeID) -> Self {
        assert_eq!(opcode.get_kind(), InstKind::AmoRmw);
        Self {
            opcode,
            value_ty,
            ordering: AmoOrdering::SeqCst,
            is_volatile: true,
            align_log2: 0,
            scope: SyncScope::System,
            ptr_operand: ValueSSA::None,
            val_operand: ValueSSA::None,
        }
    }

    pub fn opcode(mut self, opcode: Opcode) -> Self {
        assert_eq!(opcode.get_kind(), InstKind::AmoRmw);
        self.opcode = opcode;
        self
    }

    pub fn value_ty(mut self, value_ty: ValTypeID) -> Self {
        self.value_ty = value_ty;
        self
    }

    pub fn ordering(mut self, ordering: AmoOrdering) -> Self {
        self.ordering = ordering;
        self
    }

    pub fn volatile(mut self, is_volatile: bool) -> Self {
        self.is_volatile = is_volatile;
        self
    }

    pub fn align(mut self, align: usize) -> Self {
        self.align_log2 = align.trailing_zeros() as u8;
        self
    }

    pub fn align_log2(mut self, align_log2: u8) -> Self {
        self.align_log2 = align_log2;
        self
    }

    pub fn scope(mut self, scope: SyncScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn ptr_operand(mut self, ptr_operand: impl ISubValueSSA) -> Self {
        self.ptr_operand = ptr_operand.into_ir();
        self
    }

    pub fn val_operand(mut self, val_operand: impl ISubValueSSA) -> Self {
        self.val_operand = val_operand.into_ir();
        self
    }

    pub fn build(self, allocs: &IRAllocs) -> AmoRmw {
        let inst = AmoRmw {
            common: InstCommon::new(self.opcode, self.value_ty),
            operands: [Use::new(UseKind::AmoRmwPtr), Use::new(UseKind::AmoRmwVal)],
            ordering: self.ordering,
            is_volatile: self.is_volatile,
            align_log2: self.align_log2,
            scope: self.scope,
        };

        if self.ptr_operand != ValueSSA::None {
            inst.set_pointer(allocs, self.ptr_operand);
        }
        if self.val_operand != ValueSSA::None {
            inst.set_value(allocs, self.val_operand);
        }

        inst
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AmoRmwRef(InstRef);

impl ISubInstRef for AmoRmwRef {
    type InstDataT = AmoRmw;

    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        Self(inst_ref)
    }

    fn into_raw(self) -> InstRef {
        self.0
    }
}
