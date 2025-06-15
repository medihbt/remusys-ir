use std::{
    cell::Cell,
    collections::{BTreeSet, HashSet},
};

use slab::Slab;

use crate::{
    base::slablist::{
        SlabRefList, SlabRefListError, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef,
    },
    impl_slabref,
    mir::{
        inst::{MachineInst, MachineInstRef},
        operand::RegOperand,
    },
};

#[derive(Debug)]
pub struct MachineBlock {
    pub self_head: Cell<MachineBlockHead>,
    pub name: String,
    pub successors: BTreeSet<MachineBlockRef>,
    pub livein_regs: HashSet<RegOperand>,
    pub insts: SlabRefList<MachineInstRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MachineBlockHead {
    pub node_head: SlabRefListNodeHead,
    pub parent:    MachineBlockRef,
}

impl MachineBlockHead {
    pub fn new_cell() -> Cell<Self> {
        Cell::new(Self {
            node_head: SlabRefListNodeHead::new(),
            parent: MachineBlockRef(0),
        })
    }

    pub fn insert_node_head(self, node_head: SlabRefListNodeHead) -> Self {
        Self {
            node_head,
            parent: self.parent,
        }
    }
    pub fn insert_parent(self, parent: MachineBlockRef) -> Self {
        Self {
            node_head: self.node_head,
            parent,
        }
    }
    pub fn assign_to(self, target: &Cell<Self>) {
        target.set(self);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MachineBlockRef(pub usize);
impl_slabref!(MachineBlockRef, MachineBlock);

impl MachineBlock {
    pub fn new(name: String, alloc_minst: &mut Slab<MachineInst>) -> Self {
        Self {
            self_head: MachineBlockHead::new_cell(),
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

impl SlabRefListNode for MachineBlock {
    fn new_guide() -> Self {
        Self {
            self_head: MachineBlockHead::new_cell(),
            name: String::new(),
            successors: BTreeSet::new(),
            livein_regs: HashSet::new(),
            insts: SlabRefList::new_guide(),
        }
    }
    fn load_node_head(&self) -> SlabRefListNodeHead {
        self.self_head.get().node_head
    }
    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        self.self_head
            .get()
            .insert_node_head(node_head)
            .assign_to(&self.self_head);
    }
}

impl SlabRefListNodeRef for MachineBlockRef {
    fn on_node_push_next(_: Self, _: Self, _: &Slab<MachineBlock>) -> Result<(), SlabRefListError> {
        Ok(())
    }
    fn on_node_push_prev(_: Self, _: Self, _: &Slab<MachineBlock>) -> Result<(), SlabRefListError> {
        Ok(())
    }
    fn on_node_unplug(_: Self, _: &Slab<MachineBlock>) -> Result<(), SlabRefListError> {
        Ok(())
    }
}
