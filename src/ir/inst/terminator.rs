use slab::Slab;

use crate::{
    base::slablist::SlabRefList,
    ir::{
        block::jump_target::{JumpTargetData, JumpTargetKind, JumpTargetRef},
        module::Module,
    },
    typing::id::ValTypeID,
};

use super::{
    InstDataCommon, InstDataUnique, InstError,
    checking::{check_operand_type_kind_match, check_operand_type_match},
    usedef::{UseData, UseRef},
};

pub trait TerminatorInst {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>>;

    fn get_n_jump_targets(&self) -> usize {
        self.get_jump_targets().map_or(0, |targets| targets.len())
    }

    /// Whether this terminator terminates the function control flow.
    /// True value means whether this instruction will return from the function
    /// or makes the control flow unreachable.
    fn terminates_function(&self) -> bool {
        self.get_jump_targets().is_none()
    }

    fn init_jump_targets(&mut self, jt_alloc: &mut Slab<JumpTargetData>);
}

pub struct Ret {
    _retval: UseRef,
}

pub struct JumpCommon {
    _targets: SlabRefList<JumpTargetRef>,
    _condition: UseRef,
}

pub struct Jump(JumpCommon);
pub struct Br(JumpCommon);

pub struct Switch {
    _common: JumpCommon,
    _default: JumpTargetRef,
    _cases: Vec<(i128, JumpTargetRef)>,
}

impl TerminatorInst for Ret {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        None
    }
    fn init_jump_targets(&mut self, _: &mut Slab<JumpTargetData>) {}
}
impl TerminatorInst for Jump {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        Some(&self.0._targets)
    }
    fn init_jump_targets(&mut self, jt_alloc: &mut Slab<JumpTargetData>) {
        let list = SlabRefList::from_slab(jt_alloc);
        list.push_back_value(
            jt_alloc,
            JumpTargetData::new_with_kind(JumpTargetKind::Jump),
        )
        .unwrap();
        self.0._targets = list;
    }
}
impl TerminatorInst for Br {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        Some(&self.0._targets)
    }
    fn init_jump_targets(&mut self, jt_alloc: &mut Slab<JumpTargetData>) {
        let list = SlabRefList::from_slab(jt_alloc);
        list.push_back_value(
            jt_alloc,
            JumpTargetData::new_with_kind(JumpTargetKind::BrFalse),
        )
        .unwrap();
        list.push_back_value(
            jt_alloc,
            JumpTargetData::new_with_kind(JumpTargetKind::BrTrue),
        )
        .unwrap();
        self.0._targets = list;
    }
}
impl TerminatorInst for Switch {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        Some(&self._common._targets)
    }

    fn init_jump_targets(&mut self, jt_alloc: &mut Slab<JumpTargetData>) {
        let list = SlabRefList::from_slab(jt_alloc);
        self._default = list
            .push_back_value(
                jt_alloc,
                JumpTargetData::new_with_kind(JumpTargetKind::SwitchDefault),
            )
            .unwrap();
        self._common._targets = list;
    }
}

impl InstDataUnique for Ret {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self._retval = common.alloc_use(alloc_use);
    }

    fn check_operands(&self, common: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let retval = self._retval.get_operand(&module.borrow_use_alloc());
        check_operand_type_match(common.ret_type, retval, module)
    }
}
impl InstDataUnique for Jump {
    fn build_operands(&mut self, _: &mut InstDataCommon, _: &mut Slab<UseData>) {}

    fn check_operands(&self, _: &InstDataCommon, _: &Module) -> Result<(), InstError> {
        Ok(())
    }
}
impl InstDataUnique for Br {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self.0._condition = common.alloc_use(alloc_use)
    }

    fn check_operands(&self, _: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let cond = self.0._condition.get_operand(&module.borrow_use_alloc());
        check_operand_type_match(ValTypeID::new_boolean(), cond, module)
    }
}
impl InstDataUnique for Switch {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self._common._condition = common.alloc_use(alloc_use)
    }

    fn check_operands(&self, _: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let cond = self
            ._common
            ._condition
            .get_operand(&module.borrow_use_alloc());
        check_operand_type_kind_match(ValTypeID::Int(0), cond, module)
    }
}
