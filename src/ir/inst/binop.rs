use crate::{
    _remusys_ir_subinst,
    ir::{
        IRAllocs, ISubInst, ISubInstID, ISubValueSSA, IUser, InstCommon, InstObj, Opcode,
        OperandSet, UseID, UseKind, ValueSSA,
    },
    typing::ValTypeID,
};
use bitflags::bitflags;
use std::cell::Cell;

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BinOPFlags: u8 {
        const NONE  = 0;
        const EXACT = 0b0000_0001;
        const NUW   = 0b0000_0010;
        const NSW   = 0b0000_0100;
    }
}
impl std::fmt::Display for BinOPFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl BinOPFlags {
    pub fn has_exact(self) -> bool {
        self.contains(BinOPFlags::EXACT)
    }
    pub fn has_nuw(self) -> bool {
        self.contains(BinOPFlags::NUW)
    }
    pub fn has_nsw(self) -> bool {
        self.contains(BinOPFlags::NSW)
    }

    pub fn opcode_supports(opcode: Opcode) -> Self {
        match opcode {
            // Integer add/sub/mul support no-wrap flags (nuw/nsw)
            Opcode::Add | Opcode::Sub | Opcode::Mul => BinOPFlags::NUW | BinOPFlags::NSW,
            // Left shift supports no-wrap flags (nuw/nsw); exact is NOT applicable
            Opcode::Shl => BinOPFlags::NUW | BinOPFlags::NSW,
            // Right shifts and integer divisions support exact; no-wrap flags don't apply here
            Opcode::Lshr | Opcode::Ashr | Opcode::Sdiv | Opcode::Udiv => BinOPFlags::EXACT,
            _ => BinOPFlags::NONE,
        }
    }

    pub fn filter_by_opcode(self, opcode: Opcode) -> Self {
        let supported = Self::opcode_supports(opcode);
        Self::from_bits_truncate(self.bits() & supported.bits())
    }

    /// Canonical, allocation-free textual form (for writer):
    /// - Order: "nuw nsw exact"
    /// - Empty set: "" (no flags printed)
    pub fn as_str(self) -> &'static str {
        const EXACT_NUW: BinOPFlags = BinOPFlags::from_bits_retain(0b0000_0011);
        const EXACT_NSW: BinOPFlags = BinOPFlags::from_bits_retain(0b0000_0101);
        const NUW_NSW: BinOPFlags = BinOPFlags::from_bits_retain(0b0000_0110);
        const EXACT_NUW_NSW: BinOPFlags = BinOPFlags::from_bits_retain(0b0000_0111);
        match self {
            Self::NONE => "",
            Self::NUW => "nuw",
            Self::NSW => "nsw",
            Self::EXACT => "exact",
            NUW_NSW => "nuw nsw",
            EXACT_NUW => "nuw exact",
            EXACT_NSW => "nsw exact",
            EXACT_NUW_NSW => "nuw nsw exact",
            _ => "",
        }
    }
    pub fn from_name_str(s: &str) -> Option<Self> {
        let mut flags = BinOPFlags::NONE;
        for part in s.split_whitespace() {
            match part {
                "exact" => flags |= BinOPFlags::EXACT,
                "nuw" => flags |= BinOPFlags::NUW,
                "nsw" => flags |= BinOPFlags::NSW,
                _ => return None,
            }
        }
        Some(flags)
    }
}

/// 二元操作指令: 执行两个操作数的二元运算（算术运算、逻辑运算、移位运算），并返回结果。
///
/// ### LLVM 语法
///
/// ```llvm
/// %<result> = <opcode> <ty> <op1>, <op2>
/// ```
pub struct BinOPInst {
    pub common: InstCommon,
    operands: [UseID; 2],
    flags: Cell<BinOPFlags>,
}

impl IUser for BinOPInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl ISubInst for BinOPInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::BinOP(b) => Some(b),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::BinOP(b) => Some(b),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::BinOP(b) => Some(b),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::BinOP(self)
    }
    fn try_get_jts(&self) -> Option<crate::ir::JumpTargets<'_>> {
        None
    }
}
impl BinOPInst {
    pub const OP_LHS: usize = 0;
    pub const OP_RHS: usize = 1;

    pub fn new_uninit(allocs: &IRAllocs, opcode: Opcode, ty: ValTypeID) -> Self {
        assert!(
            opcode.is_binary_op(),
            "Opcode {opcode:?} is not a binary operation"
        );
        Self {
            common: InstCommon::new(opcode, ty),
            operands: [
                UseID::new(allocs, UseKind::BinOpLhs),
                UseID::new(allocs, UseKind::BinOpRhs),
            ],
            flags: Cell::new(BinOPFlags::NONE),
        }
    }

    pub fn lhs_use(&self) -> UseID {
        self.operands[Self::OP_LHS]
    }
    pub fn get_lhs(&self, allocs: &IRAllocs) -> ValueSSA {
        self.lhs_use().get_operand(allocs)
    }
    pub fn set_lhs(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.lhs_use().set_operand(allocs, val);
    }

    pub fn rhs_use(&self) -> UseID {
        self.operands[Self::OP_RHS]
    }
    pub fn get_rhs(&self, allocs: &IRAllocs) -> ValueSSA {
        self.rhs_use().get_operand(allocs)
    }
    pub fn set_rhs(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.rhs_use().set_operand(allocs, val);
    }

    pub fn get_flags(&self) -> BinOPFlags {
        self.flags.get()
    }
    pub fn set_flags(&self, flags: BinOPFlags) {
        let filtered = flags.filter_by_opcode(self.common.opcode);
        self.flags.set(filtered);
    }
    pub fn add_flags(&self, flags: BinOPFlags) {
        let current = self.flags.get();
        let combined = current | flags;
        let filtered = combined.filter_by_opcode(self.common.opcode);
        self.flags.set(filtered);
    }
    pub fn del_flags(&self, flags: BinOPFlags) {
        let current = self.flags.get();
        let removed = current - flags;
        let filtered = removed.filter_by_opcode(self.common.opcode);
        self.flags.set(filtered);
    }
    pub fn has_flag(&self, flag: BinOPFlags) -> bool {
        let current = self.flags.get();
        current.contains(flag)
    }
    pub fn has_oneof_flags(&self, flags: BinOPFlags) -> bool {
        let current = self.flags.get();
        !(current & flags).is_empty()
    }
}

_remusys_ir_subinst!(BinOPInstID, BinOPInst);
impl BinOPInstID {
    pub fn new_uninit(allocs: &IRAllocs, opcode: Opcode, ty: ValTypeID) -> Self {
        let inst = BinOPInst::new_uninit(allocs, opcode, ty);
        Self::allocate(allocs, inst)
    }
    pub fn new(allocs: &IRAllocs, opcode: Opcode, lhs: ValueSSA, rhs: ValueSSA) -> Self {
        let inst_id = Self::new_uninit(allocs, opcode, lhs.get_valtype(allocs));
        let inst = inst_id.deref_ir(allocs);
        inst.set_lhs(allocs, lhs);
        inst.set_rhs(allocs, rhs);
        inst_id
    }

    pub fn lhs_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).lhs_use()
    }
    pub fn get_lhs(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_lhs(allocs)
    }
    pub fn set_lhs(self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_lhs(allocs, val);
    }

    pub fn rhs_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).rhs_use()
    }
    pub fn get_rhs(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_rhs(allocs)
    }
    pub fn set_rhs(self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_rhs(allocs, val);
    }

    pub fn get_flags(self, allocs: &IRAllocs) -> BinOPFlags {
        self.deref_ir(allocs).get_flags()
    }
    pub fn set_flags(self, allocs: &IRAllocs, flags: BinOPFlags) {
        self.deref_ir(allocs).set_flags(flags);
    }
    pub fn add_flags(self, allocs: &IRAllocs, flags: BinOPFlags) {
        self.deref_ir(allocs).add_flags(flags);
    }
    pub fn del_flags(self, allocs: &IRAllocs, flags: BinOPFlags) {
        self.deref_ir(allocs).del_flags(flags);
    }
    pub fn has_flag(self, allocs: &IRAllocs, flag: BinOPFlags) -> bool {
        self.deref_ir(allocs).has_flag(flag)
    }
    pub fn has_oneof_flags(self, allocs: &IRAllocs, flags: BinOPFlags) -> bool {
        self.deref_ir(allocs).has_oneof_flags(flags)
    }
}
