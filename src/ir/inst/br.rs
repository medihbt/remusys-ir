use crate::{
    impl_traceable_from_common,
    ir::{
        BlockID, IRAllocs, ISubInst, ISubInstID, ISubValueSSA, ITerminatorInst, IUser, InstCommon,
        InstObj, JumpTargetID, JumpTargetKind, JumpTargets, Opcode, OperandSet, UseID, UseKind,
        ValueSSA,
    },
    subinst_id,
    typing::ValTypeID,
};

pub struct BrInst {
    pub common: InstCommon,
    cond: [UseID; 1],
    target: [JumpTargetID; 2],
}
impl_traceable_from_common!(BrInst, true);
impl IUser for BrInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.cond)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.cond
    }
}
impl ISubInst for BrInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::Br(b) => Some(b),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::Br(b) => Some(b),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::Br(b) => Some(b),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Br(self)
    }

    fn is_terminator(&self) -> bool {
        true
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        Some(JumpTargets::Fix(&self.target))
    }
}
impl ITerminatorInst for BrInst {
    fn get_jts(&self) -> JumpTargets<'_> {
        JumpTargets::Fix(&self.target)
    }
    fn jts_mut(&mut self) -> &mut [JumpTargetID] {
        &mut self.target
    }
}
impl BrInst {
    pub const OP_COND: usize = 0;
    pub const JT_THEN: usize = 0;
    pub const JT_ELSE: usize = 1;

    pub fn new_uninit(allocs: &IRAllocs) -> Self {
        Self {
            common: InstCommon::new(Opcode::Br, ValTypeID::Void),
            cond: [UseID::new(allocs, UseKind::BranchCond)],
            target: [
                JumpTargetID::new(allocs, JumpTargetKind::BrThen),
                JumpTargetID::new(allocs, JumpTargetKind::BrElse),
            ],
        }
    }

    #[inline]
    pub fn cond_use(&self) -> UseID {
        self.cond[Self::OP_COND]
    }
    pub fn get_cond(&self, allocs: &IRAllocs) -> ValueSSA {
        self.cond_use().get_operand(allocs)
    }
    pub fn set_cond(&self, allocs: &IRAllocs, val: ValueSSA) {
        assert_eq!(
            val.get_valtype(allocs),
            ValTypeID::Int(1),
            "br condition must be boolean"
        );
        self.cond_use().set_operand(allocs, val);
    }

    #[inline]
    pub fn then_jt(&self) -> JumpTargetID {
        self.target[Self::JT_THEN]
    }
    pub fn get_then(&self, allocs: &IRAllocs) -> Option<BlockID> {
        self.then_jt().get_block(allocs)
    }
    pub fn set_then(&self, allocs: &IRAllocs, block: BlockID) {
        self.then_jt().set_block(allocs, block);
    }

    #[inline]
    pub fn else_jt(&self) -> JumpTargetID {
        self.target[Self::JT_ELSE]
    }
    pub fn get_else(&self, allocs: &IRAllocs) -> Option<BlockID> {
        self.else_jt().get_block(allocs)
    }
    pub fn set_else(&self, allocs: &IRAllocs, block: BlockID) {
        self.else_jt().set_block(allocs, block);
    }
}

subinst_id!(BrInstID, BrInst, terminator);
impl BrInstID {
    pub fn new_uninit(allocs: &IRAllocs) -> Self {
        Self::allocate(allocs, BrInst::new_uninit(allocs))
    }
    pub fn new(allocs: &IRAllocs, cond: ValueSSA, thenbb: BlockID, elsebb: BlockID) -> Self {
        let ret = Self::new_uninit(allocs);
        assert_eq!(
            cond.get_valtype(allocs),
            ValTypeID::Int(1),
            "br condition must be boolean"
        );
        ret.set_cond(allocs, cond);
        ret.set_then(allocs, thenbb);
        ret.set_else(allocs, elsebb);
        ret
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

    pub fn then_jt(&self, allocs: &IRAllocs) -> JumpTargetID {
        self.deref_ir(allocs).then_jt()
    }
    pub fn get_then(&self, allocs: &IRAllocs) -> Option<BlockID> {
        self.deref_ir(allocs).get_then(allocs)
    }
    pub fn set_then(&self, allocs: &IRAllocs, block: BlockID) {
        self.deref_ir(allocs).set_then(allocs, block);
    }

    pub fn else_jt(&self, allocs: &IRAllocs) -> JumpTargetID {
        self.deref_ir(allocs).else_jt()
    }
    pub fn get_else(&self, allocs: &IRAllocs) -> Option<BlockID> {
        self.deref_ir(allocs).get_else(allocs)
    }
    pub fn set_else(&self, allocs: &IRAllocs, block: BlockID) {
        self.deref_ir(allocs).set_else(allocs, block);
    }
}
