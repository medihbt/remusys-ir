use crate::{
    impl_traceable_from_common,
    ir::{
        IRAllocs, ISubInst, ISubInstID, ITerminatorID, ITerminatorInst, IUser, InstID, InstObj,
        JumpTargetID, JumpTargets, Opcode, OperandSet, UseID, inst::InstCommon,
    },
    typing::ValTypeID,
};

/// 不可达指令: 表示函数控制流不可达
///
/// ### LLVM 语法
///
/// ```llvm
/// unreachable
/// ```
pub struct UnreachableInst {
    pub common: InstCommon,
}

impl_traceable_from_common!(UnreachableInst, true);
impl IUser for UnreachableInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&[])
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut []
    }
}
impl ISubInst for UnreachableInst {
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
        if let InstObj::Unreachable(unreach) = inst { Some(unreach) } else { None }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        if let InstObj::Unreachable(unreach) = inst { Some(unreach) } else { None }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        if let InstObj::Unreachable(unreach) = inst { Some(unreach) } else { None }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Unreachable(self)
    }
}
impl ITerminatorInst for UnreachableInst {
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
impl UnreachableInst {
    pub fn new() -> Self {
        Self {
            common: InstCommon::new(Opcode::Unreachable, ValTypeID::Void),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnreachableInstID(pub InstID);

impl ISubInstID for UnreachableInstID {
    type InstObjT = UnreachableInst;

    fn raw_from_ir(id: InstID) -> Self {
        Self(id)
    }
    fn into_ir(self) -> InstID {
        self.0
    }
}
impl ITerminatorID for UnreachableInstID {}
impl UnreachableInstID {
    pub fn new_full(allocs: &IRAllocs) -> Self {
        Self::new(allocs, UnreachableInst::new())
    }
}
