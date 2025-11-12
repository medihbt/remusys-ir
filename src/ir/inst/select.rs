use crate::{
    impl_traceable_from_common,
    ir::{
        IRAllocs, ISubInst, ISubInstID, ISubValueSSA, IUser, InstCommon, InstObj, Opcode,
        OperandSet, UseID, UseKind, ValueSSA,
    },
    subinst_id,
    typing::ValTypeID,
};

/// 选择指令: 根据条件选择两个值中的一个作为结果。
///
/// ### LLVM IR 语法
///
/// ```llvm
/// %<name> = select <type>, i1 <cond>, <true value>, <false value>
/// ```
///
/// ### 操作数布局
///
/// - `operands[0] = cond`: 条件操作数，类型为 `i1`.
/// - `operands[1] = then_val`: 条件为真时选择的值。
/// - `operands[2] = else_val`: 条件为假时选择的值。
pub struct SelectInst {
    pub common: InstCommon,
    operands: [UseID; 3],
}
impl_traceable_from_common!(SelectInst, true);
impl IUser for SelectInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl ISubInst for SelectInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::Select(s) => Some(s),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::Select(s) => Some(s),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::Select(s) => Some(s),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Select(self)
    }
    fn try_get_jts(&self) -> Option<crate::ir::JumpTargets<'_>> {
        None
    }
}
impl SelectInst {
    pub const OP_COND: usize = 0;
    pub const OP_THEN: usize = 1;
    pub const OP_ELSE: usize = 2;

    pub fn new_uninit(allocs: &IRAllocs, ty: ValTypeID) -> Self {
        Self {
            common: InstCommon::new(Opcode::Select, ty),
            operands: [
                UseID::new(allocs, UseKind::SelectCond),
                UseID::new(allocs, UseKind::SelectThen),
                UseID::new(allocs, UseKind::SelectElse),
            ],
        }
    }
    pub fn new(allocs: &IRAllocs, cond: ValueSSA, then_val: ValueSSA, else_val: ValueSSA) -> Self {
        let then_ty = then_val.get_valtype(allocs);
        let else_ty = else_val.get_valtype(allocs);
        assert_eq!(
            then_ty, else_ty,
            "then_val and else_val must have the same type"
        );
        let cond_ty = cond.get_valtype(allocs);
        assert_eq!(cond_ty, ValTypeID::Int(1), "cond must be of type i1");

        let inst = Self::new_uninit(allocs, then_ty);
        inst.cond_use().set_operand(allocs, cond);
        inst.then_use().set_operand(allocs, then_val);
        inst.else_use().set_operand(allocs, else_val);
        inst
    }

    pub fn cond_use(&self) -> UseID {
        self.operands[Self::OP_COND]
    }
    pub fn get_cond(&self, allocs: &IRAllocs) -> ValueSSA {
        self.cond_use().get_operand(allocs)
    }
    pub fn set_cond(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.cond_use().set_operand(allocs, val);
    }

    pub fn then_use(&self) -> UseID {
        self.operands[Self::OP_THEN]
    }
    pub fn get_then(&self, allocs: &IRAllocs) -> ValueSSA {
        self.then_use().get_operand(allocs)
    }
    pub fn set_then(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.then_use().set_operand(allocs, val);
    }

    pub fn else_use(&self) -> UseID {
        self.operands[Self::OP_ELSE]
    }
    pub fn get_else(&self, allocs: &IRAllocs) -> ValueSSA {
        self.else_use().get_operand(allocs)
    }
    pub fn set_else(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.else_use().set_operand(allocs, val);
    }
}

subinst_id!(SelectInstID, SelectInst);
impl SelectInstID {
    pub fn new_uninit(allocs: &IRAllocs, ty: ValTypeID) -> Self {
        let inst = SelectInst::new_uninit(allocs, ty);
        Self::allocate(allocs, inst)
    }
    pub fn new(allocs: &IRAllocs, cond: ValueSSA, then_val: ValueSSA, else_val: ValueSSA) -> Self {
        let inst = SelectInst::new(allocs, cond, then_val, else_val);
        Self::allocate(allocs, inst)
    }

    pub fn cond_use(&self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).cond_use()
    }
    pub fn get_cond(&self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_cond(allocs)
    }
    pub fn set_cond(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_cond(allocs, val);
    }

    pub fn then_use(&self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).then_use()
    }
    pub fn get_then(&self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_then(allocs)
    }
    pub fn set_then(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_then(allocs, val);
    }

    pub fn else_use(&self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).else_use()
    }
    pub fn get_else(&self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_else(allocs)
    }
    pub fn set_else(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_else(allocs, val);
    }
}
