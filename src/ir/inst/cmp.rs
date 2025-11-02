use crate::{
    impl_traceable_from_common,
    ir::{
        CmpCond, IRAllocs, ISubInst, ISubInstID, IUser, InstCommon, InstID, InstObj, JumpTargets,
        Opcode, OperandSet, UseID, UseKind, ValueSSA,
    },
    typing::{FixVecType, ScalarType, ValTypeID},
};

/// 比较指令
///
/// 执行两个操作数的比较运算，根据比较条件返回布尔值结果。
/// 支持整数、浮点数等类型的各种比较操作（相等、大于、小于等）。
///
/// ### LLVM IR 语法
///
/// ```llvm
/// %<result> = <op> <cond> <type> <lhs>, <rhs>
/// ```
///
/// ### 操作数布局
/// - `operands[0]`: 左操作数 (LHS)
/// - `operands[1]`: 右操作数 (RHS)
///
/// ### 返回类型
/// 固定返回布尔类型 (`ValTypeID::Int(1)`)
pub struct CmpInst {
    pub common: InstCommon,
    operands: [UseID; 2],
    pub cond: CmpCond,
    pub operand_ty: ValTypeID,
}
impl_traceable_from_common!(CmpInst, true);
impl IUser for CmpInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl ISubInst for CmpInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::Cmp(c) => Some(c),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::Cmp(c) => Some(c),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::Cmp(c) => Some(c),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Cmp(self)
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        None
    }
}
impl CmpInst {
    pub const OP_LHS: usize = 0;
    pub const OP_RHS: usize = 1;

    pub fn new_uninit(
        allocs: &IRAllocs,
        opcode: Opcode,
        cond: CmpCond,
        operand_ty: ValTypeID,
    ) -> Self {
        Self::check_ops(opcode, operand_ty).unwrap();
        Self {
            common: InstCommon::new(opcode, ValTypeID::Int(1)),
            operands: [UseID::new(allocs, UseKind::CmpLhs), UseID::new(allocs, UseKind::CmpRhs)],
            cond,
            operand_ty,
        }
    }

    pub fn check_ops(opcode: Opcode, operand_ty: ValTypeID) -> Result<(), String> {
        match (opcode, operand_ty) {
            (Opcode::Icmp, ValTypeID::Int(_)) => Ok(()),
            (Opcode::Icmp, ValTypeID::FixVec(FixVecType(ScalarType::Int(_), _))) => Ok(()),
            (Opcode::Fcmp, ValTypeID::Float(_)) => Ok(()),
            (Opcode::Fcmp, ValTypeID::FixVec(FixVecType(ScalarType::Float(_), _))) => Ok(()),
            (Opcode::Icmp, _) | (Opcode::Fcmp, _) => Err(format!(
                "Operand type {operand_ty:?} is not valid for opcode {opcode:?}"
            )),
            (..) => Err(format!("Invalid opcode for CmpInst: {opcode:?}")),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CmpInstID(pub InstID);

impl ISubInstID for CmpInstID {
    type InstObjT = CmpInst;

    fn raw_from_ir(id: InstID) -> Self {
        Self(id)
    }
    fn into_ir(self) -> InstID {
        self.0
    }
}
impl CmpInstID {
    pub fn new_uninit(
        allocs: &IRAllocs,
        opcode: Opcode,
        cond: CmpCond,
        operand_ty: ValTypeID,
    ) -> Self {
        let inst = CmpInst::new_uninit(allocs, opcode, cond, operand_ty);
        Self::allocate(allocs, inst)
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

    pub fn get_cond(self, allocs: &IRAllocs) -> CmpCond {
        self.deref_ir(allocs).cond
    }
    pub fn operand_ty(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).operand_ty
    }
}
