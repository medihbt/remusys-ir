use crate::{
    impl_debug_for_subinst_id, impl_traceable_from_common,
    ir::{
        IRAllocs, ISubInst, ISubInstID, ISubValueSSA, IUser, InstCommon, InstID, InstObj, Opcode,
        OperandSet, UseID, UseKind, ValueSSA,
    },
    typing::ValTypeID,
};

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
}
impl_traceable_from_common!(BinOPInst, true);
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
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BinOPInstID(pub InstID);
impl_debug_for_subinst_id!(BinOPInstID);
impl ISubInstID for BinOPInstID {
    type InstObjT = BinOPInst;

    fn raw_from_ir(id: InstID) -> Self {
        Self(id)
    }
    fn into_ir(self) -> InstID {
        self.0
    }
}

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
}
