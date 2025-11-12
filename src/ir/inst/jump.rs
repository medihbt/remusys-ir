use crate::{
    impl_traceable_from_common,
    ir::{
        BlockID, IRAllocs, ISubInst, ISubInstID, ITerminatorInst, IUser, InstCommon, InstObj,
        JumpTargetID, JumpTargetKind, JumpTargets, Opcode, OperandSet, UseID,
    },
    subinst_id,
    typing::ValTypeID,
};

pub struct JumpInst {
    pub common: InstCommon,
    target: [JumpTargetID; 1],
}
impl_traceable_from_common!(JumpInst, true);
impl IUser for JumpInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&[])
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut []
    }
}
impl ISubInst for JumpInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::Jump(j) => Some(j),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::Jump(j) => Some(j),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::Jump(j) => Some(j),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Jump(self)
    }

    fn is_terminator(&self) -> bool {
        true
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        Some(JumpTargets::Fix(&self.target))
    }
}
impl ITerminatorInst for JumpInst {
    fn get_jts(&self) -> JumpTargets<'_> {
        JumpTargets::Fix(&self.target)
    }
    fn jts_mut(&mut self) -> &mut [JumpTargetID] {
        &mut self.target
    }
    fn terminates_function(&self) -> bool {
        false
    }
}
impl JumpInst {
    pub const JT_TARGET: usize = 0;

    pub fn new_uninit(allocs: &IRAllocs) -> Self {
        Self {
            common: InstCommon::new(Opcode::Jmp, ValTypeID::Void),
            target: [JumpTargetID::new(allocs, JumpTargetKind::Jump)],
        }
    }
    pub fn with_target(allocs: &IRAllocs, block: BlockID) -> Self {
        let inst = Self::new_uninit(allocs);
        inst.set_target(allocs, block);
        inst
    }

    pub fn target_jt(&self) -> JumpTargetID {
        self.target[Self::JT_TARGET]
    }
    pub fn get_target(&self, allocs: &IRAllocs) -> Option<BlockID> {
        self.target_jt().get_block(allocs)
    }
    pub fn set_target(&self, allocs: &IRAllocs, block: BlockID) {
        self.target_jt().set_block(allocs, block);
    }
}

subinst_id!(JumpInstID, JumpInst, terminator);
impl JumpInstID {
    pub fn new_uninit(allocs: &IRAllocs) -> Self {
        Self::allocate(allocs, JumpInst::new_uninit(allocs))
    }
    pub fn with_target(allocs: &IRAllocs, block: BlockID) -> Self {
        Self::allocate(allocs, JumpInst::with_target(allocs, block))
    }

    pub fn target_jt(&self, allocs: &IRAllocs) -> JumpTargetID {
        self.deref_ir(allocs).target_jt()
    }
    pub fn get_target(&self, allocs: &IRAllocs) -> Option<BlockID> {
        self.deref_ir(allocs).get_target(allocs)
    }
    pub fn set_target(&self, allocs: &IRAllocs, block: BlockID) {
        self.deref_ir(allocs).set_target(allocs, block);
    }
}
