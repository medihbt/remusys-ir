use crate::{
    impl_debug_for_subinst_id, impl_traceable_from_common,
    ir::{
        IRAllocs, ISubInst, ISubInstID, ISubValueSSA, IUser, InstCommon, InstID, InstObj,
        JumpTargets, Opcode, OperandSet, UseID, UseKind, ValueSSA,
    },
    typing::ValTypeID,
};

/// Cast 指令：实现 LLVM IR 中的类型转换
///
/// ### LLVM IR 语法
///
/// ```llvm
/// %<result> = <op> <type> <value> to <type>
/// ```
///
/// ### 操作数布局
///
/// * `operands[0]`: 源操作数 (CastOpFrom) - 指向要转换的值
pub struct CastInst {
    pub common: InstCommon,
    operands: [UseID; 1],
    pub from_ty: ValTypeID,
}
impl_traceable_from_common!(CastInst, true);
impl IUser for CastInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl ISubInst for CastInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::Cast(cast) => Some(cast),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::Cast(cast) => Some(cast),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::Cast(cast) => Some(cast),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Cast(self)
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        None
    }
}
impl CastInst {
    pub const OP_FROM: usize = 0;

    pub fn new_uninit(allocs: &IRAllocs, opcode: Opcode, fromty: ValTypeID, ty: ValTypeID) -> Self {
        assert!(
            opcode.is_cast_op(),
            "Opcode {opcode:?} is not a cast opcode"
        );
        CastInst {
            common: InstCommon::new(opcode, ty),
            operands: [UseID::new(allocs, UseKind::CastOpFrom)],
            from_ty: fromty,
        }
    }

    pub fn from_use(&self) -> UseID {
        self.operands[Self::OP_FROM]
    }
    pub fn get_from(&self, allocs: &IRAllocs) -> ValueSSA {
        self.from_use().get_operand(allocs)
    }
    pub fn set_from(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.from_use().set_operand(allocs, val);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CastInstID(pub InstID);
impl_debug_for_subinst_id!(CastInstID);
impl ISubInstID for CastInstID {
    type InstObjT = CastInst;

    fn raw_from_ir(id: InstID) -> Self {
        CastInstID(id)
    }
    fn into_ir(self) -> InstID {
        self.0
    }
}
impl CastInstID {
    pub fn new_uninit(allocs: &IRAllocs, opcode: Opcode, fromty: ValTypeID, ty: ValTypeID) -> Self {
        let inst = CastInst::new_uninit(allocs, opcode, fromty, ty);
        Self::allocate(allocs, inst)
    }
    pub fn new(allocs: &IRAllocs, opcode: Opcode, from: ValueSSA, ty: ValTypeID) -> Self {
        let inst = Self::new_uninit(allocs, opcode, from.get_valtype(allocs), ty);
        inst.deref_ir(allocs).set_from(allocs, from);
        inst
    }

    pub fn from_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).from_use()
    }
    pub fn get_from(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_from(allocs)
    }
    pub fn set_from(self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_from(allocs, val);
    }

    pub fn from_ty(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).from_ty
    }
}
