use slab::Slab;

use crate::{
    base::INullableValue,
    ir::{
        block::jump_target::JumpTargets, inst::{ISubInstRef, InstOperands}, BlockData, BlockRef, IRAllocs, ISubInst, ISubValueSSA, ITerminatorInst, InstCommon, InstData, InstRef, JumpTarget, JumpTargetKind, Opcode, Use, UseKind, ValueSSA
    },
    typing::id::ValTypeID,
};
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

#[derive(Debug)]
pub struct Switch {
    common: InstCommon,
    cond: [Rc<Use>; 1],
    targets: RefCell<Vec<Rc<JumpTarget>>>,
}

impl ISubInst for Switch {
    fn new_empty(_: Opcode) -> Self {
        Self {
            common: InstCommon::new(Opcode::Switch, ValTypeID::Void),
            cond: [Use::new(UseKind::BranchCond)],
            targets: RefCell::new(vec![JumpTarget::new(JumpTargetKind::SwitchDefault)]),
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        match inst {
            InstData::Switch(switch) => Some(switch),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        match inst {
            InstData::Switch(switch) => Some(switch),
            _ => None,
        }
    }
    fn into_ir(self) -> InstData {
        InstData::Switch(self)
    }
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }

    fn is_terminator(&self) -> bool {
        true
    }
    fn get_operands(&self) -> InstOperands {
        InstOperands::Fixed(&self.cond)
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.cond
    }

    fn init_self_reference(&mut self, self_ref: InstRef) {
        self.common_mut().self_ref = self_ref;
        for user in &self.get_common().users {
            user.operand.set(ValueSSA::Inst(self_ref));
        }
        for operand in self.operands_mut() {
            operand.inst.set(self_ref);
        }
        for jt in &*self.targets.borrow() {
            jt.terminator.set(self_ref);
        }
    }
}

impl ITerminatorInst for Switch {
    fn read_jts<T>(&self, reader: impl FnOnce(&[Rc<JumpTarget>]) -> T) -> T {
        reader(&self.targets.borrow())
    }

    fn jts_mut(&mut self) -> &mut [Rc<JumpTarget>] {
        self.targets.get_mut()
    }

    fn get_jts(&self) -> JumpTargets {
        JumpTargets::AsRef(self.targets.borrow())
    }
}

impl Switch {
    pub fn new(allocs: &IRAllocs, cond: ValueSSA) -> Self {
        let mut switch = Self::new_empty(Opcode::Switch);
        switch.set_cond(allocs, cond);
        switch
    }

    pub fn cond(&self) -> &Rc<Use> {
        &self.cond[0]
    }
    pub fn get_cond(&self) -> ValueSSA {
        self.cond[0].get_operand()
    }
    pub fn set_cond(&mut self, allocs: &IRAllocs, cond: ValueSSA) {
        if cond != ValueSSA::None && !matches!(cond.get_valtype(allocs), ValTypeID::Int(_)) {
            panic!(
                "Switch condition must be an integer type, got: {:?}",
                cond.get_valtype(allocs)
            );
        }
        self.cond[0].set_operand(allocs, cond);
    }

    pub fn default(&self) -> Ref<Rc<JumpTarget>> {
        Ref::map(self.targets.borrow(), |targets| &targets[0])
    }
    pub fn clone_default(&self) -> Rc<JumpTarget> {
        self.default().clone()
    }
    pub fn get_default(&self) -> BlockRef {
        self.default().get_block()
    }
    pub fn set_default(&self, alloc: &Slab<BlockData>, block: BlockRef) {
        self.default().set_block(alloc, block);
    }

    pub fn cases(&self) -> Ref<[Rc<JumpTarget>]> {
        Ref::map(self.targets.borrow(), |targets| &targets[1..])
    }
    pub fn ref_case<T: Into<i128>>(&self, case: T) -> Option<Ref<Rc<JumpTarget>>> {
        let case_value = case.into();
        let cases = self.cases();
        let case_index = cases
            .iter()
            .position(|jt| jt.kind == JumpTargetKind::SwitchCase(case_value))?;
        Some(Ref::map(cases, |cases| &cases[case_index]))
    }
    pub fn get_case<T: Into<i128>>(&self, case: T) -> Option<BlockRef> {
        self.ref_case(case).map(|jt| jt.get_block())
    }

    /// 如果成功, 返回 true, 否则返回 false
    pub fn set_existing_case<T: Into<i128>>(
        &self,
        alloc: &Slab<BlockData>,
        case: T,
        block: BlockRef,
    ) -> bool {
        let case_value = case.into();
        let Some(case) = self.ref_case(case_value) else {
            return false;
        };
        case.set_block(alloc, block);
        true
    }

    pub fn set_case<T: Into<i128>>(
        &self,
        alloc: &Slab<BlockData>,
        case: T,
        block: BlockRef,
    ) -> Rc<JumpTarget> {
        let case_value = case.into();
        if let Some(existing_case) = self.ref_case(case_value) {
            existing_case.set_block(alloc, block);
            existing_case.clone()
        } else {
            let new_case = JumpTarget::new(JumpTargetKind::SwitchCase(case_value));
            new_case.set_block(alloc, block);
            self.targets.borrow_mut().push(new_case.clone());
            new_case
        }
    }

    pub fn remove_case<T: Into<i128>>(&self, case: T) -> bool {
        let case_value = case.into();
        let mut targets = self.targets.borrow_mut();
        if let Some(index) = targets
            .iter()
            .position(|jt| jt.kind == JumpTargetKind::SwitchCase(case_value))
        {
            targets.remove(index);
            true
        } else {
            false
        }
    }

    pub fn remove_cases_when(
        &self,
        alloc: &Slab<BlockData>,
        condition: impl Fn(&Rc<JumpTarget>) -> bool,
    ) {
        let mut targets = self.targets.borrow_mut();
        targets.retain(|jt| {
            if jt.kind == JumpTargetKind::SwitchDefault || !condition(jt) {
                true
            } else {
                jt.set_block(alloc, BlockRef::new_null());
                false
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchRef(InstRef);

impl ISubInstRef for SwitchRef {
    type InstDataT = Switch;
    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        Self(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}
