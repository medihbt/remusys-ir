use std::{
    cell::Cell,
    collections::{BTreeSet, HashSet},
};

use slab::Slab;

use crate::{
    base::slablist::{SlabRefList, SlabRefListNode, SlabRefListNodeHead},
    impl_slabref,
    mir::{inst::{MachineInst, MachineInstRef}, operand::RegOperand},
};

#[derive(Debug)]
pub struct MachineBlock {
    pub self_head: Cell<SlabRefListNodeHead>,
    pub name: String,
    pub successors: BTreeSet<MachineBlockRef>,
    pub livein_regs: HashSet<RegOperand>,
    pub insts: SlabRefList<MachineInstRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MachineBlockRef(pub usize);
impl_slabref!(MachineBlockRef, MachineBlock);

impl MachineBlock {
    pub fn new(name: String, alloc_minst: &mut Slab<MachineInst>) -> Self {
        Self {
            self_head: Cell::new(SlabRefListNodeHead::new()),
            name,
            successors: BTreeSet::new(),
            livein_regs: HashSet::new(),
            insts: SlabRefList::from_slab(alloc_minst),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn add_successor(&mut self, block_ref: MachineBlockRef) {
        self.successors.insert(block_ref);
    }
    pub fn add_livein_reg(&mut self, reg: RegOperand) {
        self.livein_regs.insert(reg);
    }
}