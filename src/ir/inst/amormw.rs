use crate::{
    impl_debug_for_subinst_id, impl_traceable_from_common,
    ir::{
        IPtrUniqueUser, IRAllocs, ISubInst, ISubInstID, IUser, InstCommon, InstID, InstObj, Opcode,
        OperandSet, UseID, UseKind, ValueSSA,
    },
    typing::ValTypeID,
};

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
    Other(&'static str),
}

impl SyncScope {
    pub fn as_str(self) -> &'static str {
        match self {
            SyncScope::SingleThread => "singlethread",
            SyncScope::System => "system",
            SyncScope::Other(name) => name,
        }
    }
    pub fn from_str(name: &'static str) -> Self {
        match name {
            "singlethread" => SyncScope::SingleThread,
            "system" => SyncScope::System,
            other => SyncScope::Other(other),
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
pub struct AmoRmwInst {
    pub common: InstCommon,
    operands: [UseID; 2],
    pub value_ty: ValTypeID,
    pub ordering: AmoOrdering,
    pub scope: SyncScope,
    pub is_volatile: bool,
    pub align_log2: u8,
}
impl_traceable_from_common!(AmoRmwInst, true);
impl IUser for AmoRmwInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl IPtrUniqueUser for AmoRmwInst {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        self.value_ty
    }
    fn get_operand_pointee_align(&self) -> u32 {
        1 << self.align_log2
    }
}
impl ISubInst for AmoRmwInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        let InstObj::AmoRmw(amormw) = inst else {
            return None;
        };
        Some(amormw)
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        let InstObj::AmoRmw(amormw) = inst else {
            return None;
        };
        Some(amormw)
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        let InstObj::AmoRmw(amormw) = inst else {
            return None;
        };
        Some(amormw)
    }
    fn into_ir(self) -> InstObj {
        InstObj::AmoRmw(self)
    }
    fn try_get_jts(&self) -> Option<crate::ir::JumpTargets<'_>> {
        None
    }
}
impl AmoRmwInst {
    pub const OP_POINTER: usize = 0;
    pub const OP_VALUE: usize = 1;

    pub fn builder(opcode: Opcode, value_ty: ValTypeID) -> AmoRmwBuilder {
        AmoRmwBuilder::new(opcode, value_ty)
    }

    pub fn pointer_use(&self) -> UseID {
        self.operands[Self::OP_POINTER]
    }
    pub fn get_pointer(&self, allocs: &IRAllocs) -> ValueSSA {
        self.pointer_use().get_operand(allocs)
    }
    pub fn set_pointer(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.pointer_use().set_operand(allocs, val);
    }

    pub fn value_use(&self) -> UseID {
        self.operands[Self::OP_VALUE]
    }
    pub fn get_value(&self, allocs: &IRAllocs) -> ValueSSA {
        self.value_use().get_operand(allocs)
    }
    pub fn set_value(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.value_use().set_operand(allocs, val);
    }

    /// Includes:
    ///
    /// ```text
    /// AmoXchg, AmoAdd, AmoSub, AmoAnd, AmoNand, AmoOr, AmoXor,
    /// AmoSMax, AmoSMin, AmoUMax, AmoUMin,
    /// AmoFAdd, AmoFSub, AmoFMax, AmoFMin,
    /// AmoUIncWrap, AmoUDecWrap, AmoUSubCond, AmoUSubStat,
    /// ```
    pub fn subop_get_name(opcode: Opcode) -> &'static str {
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
    pub fn subop_name(&self) -> &'static str {
        Self::subop_get_name(self.get_opcode())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AmoRmwInstID(pub InstID);
impl_debug_for_subinst_id!(AmoRmwInstID);
impl ISubInstID for AmoRmwInstID {
    type InstObjT = AmoRmwInst;

    fn raw_from_ir(id: InstID) -> Self {
        Self(id)
    }
    fn into_ir(self) -> InstID {
        self.0
    }
}
impl AmoRmwInstID {
    pub fn pointer_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).pointer_use()
    }
    pub fn get_pointer(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_pointer(allocs)
    }
    pub fn set_pointer(self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_pointer(allocs, val);
    }

    pub fn value_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).value_use()
    }
    pub fn get_value(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_value(allocs)
    }
    pub fn set_value(self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_value(allocs, val);
    }

    pub fn value_ty(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).value_ty
    }
    pub fn ordering(self, allocs: &IRAllocs) -> AmoOrdering {
        self.deref_ir(allocs).ordering
    }
    pub fn scope(self, allocs: &IRAllocs) -> SyncScope {
        self.deref_ir(allocs).scope
    }
    pub fn is_volatile(self, allocs: &IRAllocs) -> bool {
        self.deref_ir(allocs).is_volatile
    }
    pub fn align_log2(self, allocs: &IRAllocs) -> u8 {
        self.deref_ir(allocs).align_log2
    }
    pub fn align(self, allocs: &IRAllocs) -> u32 {
        1 << self.align_log2(allocs)
    }
}

pub trait IAmoRmwBuildable: Sized {
    fn new(opcode: Opcode, value_ty: ValTypeID) -> Self;
    fn value_ty(self, ty: ValTypeID) -> Self;
    fn ordering(self, ordering: AmoOrdering) -> Self;
    fn scope(self, scope: SyncScope) -> Self;
    fn is_volatile(self, is_volatile: bool) -> Self;
    fn align_log2(self, align_log2: u8) -> Self;

    fn build_obj(self, allocs: &IRAllocs) -> AmoRmwInst;
    fn build_id(self, allocs: &IRAllocs) -> AmoRmwInstID {
        AmoRmwInstID::allocate(allocs, self.build_obj(allocs))
    }
}
pub struct AmoRmwBuilder {
    opcode: Opcode,
    value_ty: ValTypeID,
    ordering: AmoOrdering,
    scope: SyncScope,
    is_volatile: bool,
    align_log2: u8,
}
impl IAmoRmwBuildable for AmoRmwBuilder {
    fn new(opcode: Opcode, value_ty: ValTypeID) -> Self {
        Self {
            opcode,
            value_ty,
            ordering: AmoOrdering::SeqCst,
            scope: SyncScope::System,
            is_volatile: true,
            align_log2: 0,
        }
    }
    fn value_ty(mut self, ty: ValTypeID) -> Self {
        self.value_ty = ty;
        self
    }
    fn ordering(mut self, ordering: AmoOrdering) -> Self {
        self.ordering = ordering;
        self
    }
    fn scope(mut self, scope: SyncScope) -> Self {
        self.scope = scope;
        self
    }
    fn is_volatile(mut self, is_volatile: bool) -> Self {
        self.is_volatile = is_volatile;
        self
    }
    fn align_log2(mut self, align_log2: u8) -> Self {
        self.align_log2 = align_log2;
        self
    }

    fn build_obj(self, allocs: &IRAllocs) -> AmoRmwInst {
        AmoRmwInst {
            common: InstCommon::new(self.opcode, self.value_ty),
            operands: [
                UseID::new(allocs, UseKind::AmoRmwPtr),
                UseID::new(allocs, UseKind::AmoRmwVal),
            ],
            value_ty: self.value_ty,
            ordering: self.ordering,
            scope: self.scope,
            is_volatile: self.is_volatile,
            align_log2: self.align_log2,
        }
    }
}
