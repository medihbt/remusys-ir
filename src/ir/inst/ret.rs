use crate::{
    impl_traceable_from_common,
    ir::{
        IRAllocs, ISubInst, ISubInstID, ITerminatorID, ITerminatorInst, IUser, InstID, InstObj,
        JumpTargetID, JumpTargets, Opcode, OperandSet, UseID, UseKind, ValueSSA, inst::InstCommon,
    },
    typing::ValTypeID,
};
use mtb_entity::PtrID;

/// 返回指令
///
/// ### LLVM 语法
///
/// ```llvm
/// ret <ty> <value> ; when returns a value
/// ret void ; when returns nothing
/// ```
pub struct RetInst {
    common: InstCommon,
    operands: [UseID; 1],
}
impl_traceable_from_common!(RetInst, true);
impl IUser for RetInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl ISubInst for RetInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }

    fn is_terminator(&self) -> bool {
        true
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        Some(JumpTargets::Fix(&[]))
    }

    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        if let InstObj::Ret(ret) = inst { Some(ret) } else { None }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        if let InstObj::Ret(ret) = inst { Some(ret) } else { None }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        if let InstObj::Ret(ret) = inst { Some(ret) } else { None }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Ret(self)
    }
}
impl ITerminatorInst for RetInst {
    fn get_jts(&self) -> JumpTargets<'_> {
        JumpTargets::Fix(&[])
    }
    fn jts_mut(&mut self) -> &mut [JumpTargetID] {
        &mut []
    }
    fn terminates_function(&self) -> bool {
        true
    }
}
impl RetInst {
    pub const OP_RETVAL: usize = 0;

    pub fn retval_use(&self) -> UseID {
        self.operands[Self::OP_RETVAL]
    }
    pub fn get_retval(&self, allocs: &IRAllocs) -> ValueSSA {
        self.retval_use().get_operand(allocs)
    }
    pub fn set_retval(&self, allocs: &IRAllocs, value: ValueSSA) {
        self.retval_use().set_operand(allocs, value);
    }
    pub fn has_retval(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    pub fn new_uninit(allocs: &IRAllocs, ret_ty: ValTypeID) -> Self {
        Self {
            common: InstCommon::new(Opcode::Ret, ret_ty),
            operands: [UseID::new(UseKind::RetValue, allocs)],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RetInstID(pub InstID);

impl ISubInstID for RetInstID {
    type InstObjT = RetInst;

    fn raw_from_ir(id: PtrID<InstObj>) -> Self {
        RetInstID(id)
    }
    fn into_ir(self) -> PtrID<InstObj> {
        self.0
    }
}
impl ITerminatorID for RetInstID {}
impl RetInstID {
    pub fn new_uninit(allocs: &IRAllocs, ret_ty: ValTypeID) -> Self {
        Self::new(allocs, RetInst::new_uninit(allocs, ret_ty))
    }

    pub fn retval_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).retval_use()
    }
    pub fn get_retval(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_retval(allocs)
    }
    pub fn set_retval(self, allocs: &IRAllocs, value: ValueSSA) {
        self.deref_ir(allocs).set_retval(allocs, value);
    }
    pub fn has_retval(self, allocs: &IRAllocs) -> bool {
        self.deref_ir(allocs).has_retval()
    }
}
