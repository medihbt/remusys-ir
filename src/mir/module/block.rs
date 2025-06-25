use std::{
    cell::Cell,
    collections::{BTreeSet, HashSet},
};

use slab::Slab;

use crate::{
    base::slablist::{SlabRefListError, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef},
    impl_slabref,
    mir::{inst::MirInst, operand::reg::RegOperand},
};

/// MIR Blocks: labels and instruction collections.
#[derive(Debug, Clone)]
pub struct MirBlock {
    pub node_head: Cell<SlabRefListNodeHead>,
    pub name: String,
    pub insts: Vec<MirInst>,
    pub livein_regs: HashSet<RegOperand>,
    pub successors: BTreeSet<MirBlockRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MirBlockRef(usize);
impl_slabref!(MirBlockRef, MirBlock);

impl SlabRefListNode for MirBlock {
    fn new_guide() -> Self {
        Self::new(String::new())
    }
    fn load_node_head(&self) -> SlabRefListNodeHead {
        self.node_head.get()
    }
    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        self.node_head.set(node_head);
    }
}

impl SlabRefListNodeRef for MirBlockRef {
    fn on_node_push_next(_: Self, _: Self, _: &Slab<MirBlock>) -> Result<(), SlabRefListError> {
        Ok(())
    }
    fn on_node_push_prev(_: Self, _: Self, _: &Slab<MirBlock>) -> Result<(), SlabRefListError> {
        Ok(())
    }
    fn on_node_unplug(_: Self, _: &Slab<MirBlock>) -> Result<(), SlabRefListError> {
        Ok(())
    }
}

impl MirBlock {
    pub fn new(name: String) -> Self {
        Self {
            node_head: Cell::new(SlabRefListNodeHead::new()),
            name,
            insts: Vec::new(),
            livein_regs: HashSet::new(),
            successors: BTreeSet::new(),
        }
    }
    pub fn add_inst(&mut self, inst: MirInst) {
        self.insts.push(inst);
    }
}
