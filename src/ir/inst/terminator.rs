use std::ops::ControlFlow;

use crate::{base::slabref::SlabRef, ir::{block::BlockRef, opcode::Opcode, Module}, typing::id::ValTypeID};

use super::{instructions::CallOp, usedef::{UseData, UseRef}, InstCommon, InstDataTrait, InstRef};


pub trait TerminatorInst {
    fn get_n_jump_targets(&self) -> usize;

    /// Whether this terminator terminates the function control flow.
    /// True value means whether this instruction will return from the function
    /// or makes the control flow unreachable.
    fn terminates_function(&self) -> bool;

    fn read_jump_targets<R>  (&self,     reader: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R>;
    fn modify_jump_targets<R>(&mut self, editor: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R>;
}

pub struct Unreachable;
pub struct Ret  { pub retval: UseRef }
pub struct TailCallOp (pub CallOp);

pub struct Jump { pub target: BlockRef }
pub struct Br {
    pub cond: UseRef,
    pub true_block:  BlockRef,
    pub false_block: BlockRef,
}
pub struct Switch {
    pub cond:    UseRef,
    pub default: BlockRef,
    pub cases:   Vec<(i128, BlockRef)>,
}

impl TerminatorInst for Unreachable {
    fn get_n_jump_targets(&self) -> usize { 0 }
    fn terminates_function(&self) -> bool { true }
    fn read_jump_targets<R>(&self, _: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R> {
        ControlFlow::Continue(())
    }
    fn modify_jump_targets<R>(&mut self, _: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R> {
        ControlFlow::Continue(())
    }
}
impl TerminatorInst for Ret {
    fn get_n_jump_targets(&self) -> usize { 0 }
    fn terminates_function(&self) -> bool { true }
    fn read_jump_targets<R>(&self, _: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R> {
        ControlFlow::Continue(())
    }
    fn modify_jump_targets<R>(&mut self, _: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R> {
        ControlFlow::Continue(())
    }
}
impl TerminatorInst for TailCallOp {
    fn get_n_jump_targets(&self) -> usize { 0 }
    fn terminates_function(&self) -> bool { true }
    fn read_jump_targets<R>(&self, _: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R> {
        ControlFlow::Continue(())
    }
    fn modify_jump_targets<R>(&mut self, _: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R> {
        ControlFlow::Continue(())
    }
}

impl TerminatorInst for Jump {
    fn get_n_jump_targets(&self) -> usize { 1 }
    fn terminates_function(&self) -> bool { false }
    fn read_jump_targets<R>(&self, mut reader: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R> {
        reader(&self.target)
    }
    fn modify_jump_targets<R>(&mut self, mut editor: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R> {
        editor(&self.target)
    }
}
impl TerminatorInst for Br {
    fn get_n_jump_targets(&self) -> usize { 2 }
    fn terminates_function(&self) -> bool { false }
    fn read_jump_targets<R>(&self, mut reader: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R> {
        if let ControlFlow::Break(r) = reader(&self.true_block) {
            return ControlFlow::Break(r);
        }
        if let ControlFlow::Break(r) = reader(&self.false_block) {
            return ControlFlow::Break(r);
        }
        ControlFlow::Continue(())
    }
    fn modify_jump_targets<R>(&mut self, mut editor: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R> {
        if let ControlFlow::Break(r) = editor(&self.true_block) {
            return ControlFlow::Break(r);
        }
        if let ControlFlow::Break(r) = editor(&self.false_block) {
            return ControlFlow::Break(r);
        }
        ControlFlow::Continue(())
    }
}
impl TerminatorInst for Switch {
    fn get_n_jump_targets(&self) -> usize { self.cases.len() + 1 }
    fn terminates_function(&self) -> bool { false }
    fn read_jump_targets<R>(&self, mut reader: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R> {
        if let ControlFlow::Break(r) = reader(&self.default) {
            return ControlFlow::Break(r);
        }
        for (_, block) in &self.cases {
            if let ControlFlow::Break(r) = reader(block) {
                return ControlFlow::Break(r);
            }
        }
        ControlFlow::Continue(())
    }
    fn modify_jump_targets<R>(&mut self, mut editor: impl FnMut(&BlockRef) -> ControlFlow<R>) -> ControlFlow<R> {
        if let ControlFlow::Break(r) = editor(&self.default) {
            return ControlFlow::Break(r);
        }
        for (_, block) in &mut self.cases {
            if let ControlFlow::Break(r) = editor(block) {
                return ControlFlow::Break(r);
            }
        }
        ControlFlow::Continue(())
    }
}

impl InstDataTrait for Ret {
    fn init_common(&mut self, opcode: Opcode, ty: ValTypeID, parent: BlockRef, module: &mut Module) -> InstCommon {
        let common = InstCommon::new(opcode, ty, parent, module);
        self.retval = common.add_use(
            UseData::new(InstRef::new_nil()),
            &mut module._alloc_use
        );
        common
    }
}

impl InstDataTrait for Jump {}

impl InstDataTrait for Br {
    fn init_common(&mut self, opcode: Opcode, ty: ValTypeID, parent: BlockRef, module: &mut Module) -> InstCommon {
        let common = InstCommon::new(opcode, ty, parent, module);
        self.cond = common.add_use(
            UseData::new(InstRef::new_nil()),
            &mut module._alloc_use
        );
        common
    }
}

impl InstDataTrait for Switch {
    fn init_common(&mut self, opcode: Opcode, ty: ValTypeID, parent: BlockRef, module: &mut Module) -> InstCommon {
        let common = InstCommon::new(opcode, ty, parent, module);
        self.cond = common.add_use(
            UseData::new(InstRef::new_nil()),
            &mut module._alloc_use
        );
        common
    }
}