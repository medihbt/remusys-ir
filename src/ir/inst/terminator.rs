use std::{cell::RefCell, collections::BTreeMap};

use slab::Slab;

use crate::{
    base::{slablist::SlabRefList, slabref::SlabRef, NullableValue},
    ir::{block::BlockRef, opcode::Opcode, Module},
    typing::id::ValTypeID,
};

use super::{
    instructions::CallOp, jump_targets::JumpTargetRef, usedef::{UseData, UseRef}, Inst, InstCommon, InstDataTrait, InstRef
};

pub trait TerminatorInst {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>>;

    fn get_n_jump_targets(&self) -> usize {
        self.get_jump_targets().map_or(0, SlabRefList::get_size)
    }

    /// Whether this terminator terminates the function control flow.
    /// True value means whether this instruction will return from the function
    /// or makes the control flow unreachable.
    fn terminates_function(&self) -> bool {
        self.get_jump_targets().is_none()
    }
}

pub struct Unreachable;
pub struct Ret {
    pub retval: UseRef,
}
pub struct TailCallOp(pub CallOp);

pub struct Jump {
    pub jump_targets: SlabRefList<JumpTargetRef>,
}
pub struct Br {
    pub cond: UseRef,
    pub jump_targets: SlabRefList<JumpTargetRef>,
}
pub struct Switch {
    pub cond:           UseRef,
    pub jump_targets:   SlabRefList<JumpTargetRef>,
    pub default_target: JumpTargetRef,
    pub cases:          RefCell<BTreeMap<i128, JumpTargetRef>>,
}

impl TerminatorInst for Unreachable {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        None
    }
}
impl TerminatorInst for Ret {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        None
    }
}
impl TerminatorInst for TailCallOp {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        None
    }
}

impl TerminatorInst for Jump {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        Some(&self.jump_targets)
    }
}
impl TerminatorInst for Br {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        Some(&self.jump_targets)
    }
}
impl TerminatorInst for Switch {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        Some(&self.jump_targets)
    }
}

impl InstDataTrait for Ret {
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        let common = InstCommon::new(opcode, ty, parent, module);
        self.retval = common.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        common
    }
}

impl InstDataTrait for Jump {}

impl InstDataTrait for Br {
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        let common = InstCommon::new(opcode, ty, parent, module);
        self.cond = common.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        common
    }
}

impl InstDataTrait for Switch {
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        let common = InstCommon::new(opcode, ty, parent, module);
        self.cond = common.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        common
    }
}

/**
 * Simple view reference of terminator instruction.
 */
pub struct TerminatorInstView<'a>(pub(crate) usize, pub(crate) &'a Slab<Inst>);

impl<'a> TerminatorInstView<'a> {
    pub fn from_inst(inst_ref: InstRef, inst_alloc: &'a Slab<Inst>) -> Option<Self> {
        let is_terminator = inst_ref.to_slabref(&inst_alloc)
            .map(Inst::is_terminator)
            .expect("Invalid instruction reference (Use after free?)");
        if is_terminator {
            Some(TerminatorInstView(inst_ref.get_handle(), inst_alloc))
        } else {
            None
        }
    }
    pub fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        let inst_ref = InstRef::from_handle(self.0);
        inst_ref.to_slabref(&self.1)
            .map(|inst| {
                match inst {
                    Inst::Jump  (_, j) => j.get_jump_targets(),
                    Inst::Br    (_, b) => b.get_jump_targets(),
                    Inst::Switch(_, s) => s.get_jump_targets(),
                    _ => None,
                }
            })
            .expect("Invalid instruction reference (Use after free?)")
    }

    pub fn as_inst(&self) -> InstRef {
        InstRef::from_handle(self.0)
    }
}
