pub mod addr;
pub mod cond;
pub mod impls;
pub mod inst;
pub mod mir_call;
pub mod mirops;
pub mod opcode;
pub mod pseudo;
pub mod reg_restore;
pub mod reg_save;
pub mod subinst;
pub mod switch;

use crate::{
    base::{
        slablist::{SlabRefListError, SlabRefListNodeHead, SlabRefListNodeRef},
        slabref::SlabRef,
    },
    mir::{
        inst::{inst::MirInst, opcode::MirOP},
        module::MirModule,
        operand::MirOperand,
    },
};
use slab::Slab;
use std::cell::{Cell, Ref};

pub use self::subinst::IMirSubInst;

#[derive(Debug, Clone)]
pub struct MirInstCommon {
    node_head: Cell<SlabRefListNodeHead>,
    pub opcode: MirOP,
}

impl MirInstCommon {
    pub fn new(opcode: MirOP) -> Self {
        MirInstCommon {
            node_head: Cell::new(SlabRefListNodeHead::new()),
            opcode,
        }
    }

    pub fn new_guide() -> Self {
        MirInstCommon {
            node_head: Cell::new(SlabRefListNodeHead::new()),
            opcode: MirOP::Add32I,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirInstRef(usize);

impl SlabRef for MirInstRef {
    type RefObject = MirInst;
    fn from_handle(handle: usize) -> Self {
        MirInstRef(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl SlabRefListNodeRef for MirInstRef {
    fn on_node_push_next(_: Self, _: Self, _: &Slab<MirInst>) -> Result<(), SlabRefListError> {
        Ok(())
    }
    fn on_node_push_prev(_: Self, _: Self, _: &Slab<MirInst>) -> Result<(), SlabRefListError> {
        Ok(())
    }
    fn on_node_unplug(_: Self, _: &Slab<MirInst>) -> Result<(), SlabRefListError> {
        Ok(())
    }
}

impl MirInstRef {
    pub fn from_alloc(alloc: &mut Slab<MirInst>, data: MirInst) -> Self {
        MirInstRef(alloc.insert(data))
    }
    pub fn from_module(module: &MirModule, data: MirInst) -> Self {
        let mut alloc_inst = module.borrow_alloc_inst_mut();
        MirInstRef::from_alloc(&mut alloc_inst, data)
    }
    pub fn from_mut_module(module: &mut MirModule, data: MirInst) -> Self {
        let allocs = module.allocs.get_mut();
        MirInstRef::from_alloc(&mut allocs.inst, data)
    }

    pub fn data_from_module<'a>(&self, module: &'a MirModule) -> Ref<'a, MirInst> {
        let alloc_inst = module.borrow_alloc_inst();
        Ref::map(alloc_inst, |alloc| {
            alloc.get(self.get_handle()).expect("Invalid MirInstRef")
        })
    }
}

pub mod utils {
    use crate::mir::operand::reg::{GPReg, RegUseFlags, VFReg};

    use super::*;
    pub fn mark_out_operands_defined(operands: &[Cell<MirOperand>]) {
        for operand in operands {
            mark_operand_defined(operand);
        }
    }
    pub fn mark_operand_defined(operand: &Cell<MirOperand>) {
        let old = operand.get();
        let new = match old {
            MirOperand::GPReg(GPReg(id, si, mut uf)) => {
                uf.insert(RegUseFlags::DEF);
                MirOperand::GPReg(GPReg(id, si, uf))
            }
            MirOperand::VFReg(VFReg(id, si, mut uf)) => {
                uf.insert(RegUseFlags::DEF);
                MirOperand::VFReg(VFReg(id, si, uf))
            }
            _ => return,
        };
        operand.set(new);
    }
    pub fn mark_in_operands_used(operands: &[Cell<MirOperand>]) {
        for operand in operands {
            mark_operand_used(operand);
        }
    }
    pub fn mark_operand_used(operand: &Cell<MirOperand>) {
        let old = operand.get();
        let new = match old {
            MirOperand::GPReg(GPReg(id, si, mut uf)) => {
                uf.insert(RegUseFlags::USE);
                MirOperand::GPReg(GPReg(id, si, uf))
            }
            MirOperand::VFReg(VFReg(id, si, mut uf)) => {
                uf.insert(RegUseFlags::USE);
                MirOperand::VFReg(VFReg(id, si, uf))
            }
            _ => return,
        };
        operand.set(new);
    }
}
