use std::{
    cell::{Cell, Ref},
    collections::{BTreeSet, HashSet},
};

use slab::Slab;

use crate::{
    base::{SlabListError, SlabListNode, SlabListNodeHead, SlabListNodeRef, SlabRef, SlabRefList},
    mir::{
        inst::{MirInstRef, inst::MirInst},
        module::MirModule,
        operand::reg::RegOperand,
    },
};

/// MIR Blocks: labels and instruction collections.
#[derive(Debug)]
pub struct MirBlock {
    pub node_head: Cell<SlabListNodeHead>,
    pub name: String,
    pub insts: SlabRefList<MirInstRef>,
    pub livein_regs: HashSet<RegOperand>,
    pub successors: BTreeSet<MirBlockRef>,
    pub predecessors: BTreeSet<MirBlockRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirBlockRef(usize);

impl SlabRef for MirBlockRef {
    type RefObject = MirBlock;
    fn from_handle(handle: usize) -> Self {
        MirBlockRef(handle)
    }
    fn get_handle(&self) -> usize {
        self.0 as usize
    }
}

impl SlabListNode for MirBlock {
    fn new_guide() -> Self {
        Self {
            node_head: Cell::new(SlabListNodeHead::new()),
            name: String::new(),
            insts: SlabRefList::new_guide(),
            livein_regs: HashSet::new(),
            successors: BTreeSet::new(),
            predecessors: BTreeSet::new(),
        }
    }
    fn load_node_head(&self) -> SlabListNodeHead {
        self.node_head.get()
    }
    fn store_node_head(&self, node_head: SlabListNodeHead) {
        self.node_head.set(node_head);
    }
}

impl SlabListNodeRef for MirBlockRef {
    fn on_node_push_next(_: Self, _: Self, _: &Slab<MirBlock>) -> Result<(), SlabListError> {
        Ok(())
    }
    fn on_node_push_prev(_: Self, _: Self, _: &Slab<MirBlock>) -> Result<(), SlabListError> {
        Ok(())
    }
    fn on_node_unplug(_: Self, _: &Slab<MirBlock>) -> Result<(), SlabListError> {
        Ok(())
    }
}

impl MirBlock {
    pub fn new(name: String, alloc_inst: &mut Slab<MirInst>) -> Self {
        Self {
            node_head: Cell::new(SlabListNodeHead::new()),
            name,
            insts: SlabRefList::from_slab(alloc_inst),
            livein_regs: HashSet::new(),
            successors: BTreeSet::new(),
            predecessors: BTreeSet::new(),
        }
    }
    pub fn push_inst(&self, inst: MirInst, alloc_inst: &mut Slab<MirInst>) -> MirInstRef {
        let inst_ref = MirInstRef::from_alloc(alloc_inst, inst);
        self.insts
            .push_back_ref(alloc_inst, inst_ref)
            .expect("Failed to add instruction to block");
        inst_ref
    }
    pub fn push_inst_from_module(&self, inst: MirInst, module: &MirModule) -> MirInstRef {
        let inst_ref = MirInstRef::from_module(module, inst);
        self.insts
            .push_back_ref(&*module.borrow_alloc_inst_mut(), inst_ref)
            .expect("Failed to add instruction to block");
        inst_ref
    }
    pub fn push_inst_ref(&self, inst: MirInstRef, alloc_inst: &Slab<MirInst>) {
        self.insts
            .push_back_ref(alloc_inst, inst)
            .expect("Failed to add instruction reference to block");
    }
    pub fn push_inst_ref_from_module(&mut self, inst: MirInstRef, module: &MirModule) {
        self.insts
            .push_back_ref(&*module.borrow_alloc_inst_mut(), inst)
            .expect("Failed to add instruction reference to block");
    }
}

impl MirBlockRef {
    pub fn from_alloc(alloc: &mut Slab<MirBlock>, data: MirBlock) -> Self {
        MirBlockRef(alloc.insert(data))
    }
    pub fn from_module(module: &MirModule, data: MirBlock) -> Self {
        let mut alloc = module.borrow_alloc_block_mut();
        MirBlockRef::from_alloc(&mut alloc, data)
    }

    pub fn data_from_module(self, module: &MirModule) -> Ref<'_, MirBlock> {
        let alloc = module.borrow_alloc_block();
        Ref::map(alloc, |a| self.as_data(a).expect("Invalid MirBlockRef"))
    }

    pub fn get_name(self, alloc: &Slab<MirBlock>) -> &str {
        let block = self.as_data(alloc).expect("Invalid MirBlockRef");
        &block.name
    }
    pub fn get_name_from_module(self, module: &MirModule) -> String {
        let alloc = module.borrow_alloc_block();
        self.get_name(&*alloc).to_string()
    }
    pub fn get_insts(self, alloc: &Slab<MirBlock>) -> &SlabRefList<MirInstRef> {
        let block = self.as_data(alloc).expect("Invalid MirBlockRef");
        &block.insts
    }
    pub fn get_insts_from_module(self, module: &MirModule) -> Ref<'_, SlabRefList<MirInstRef>> {
        let alloc = module.borrow_alloc_block();
        Ref::map(alloc, |a| self.get_insts(a))
    }

    pub fn get_predecessors(self, alloc: &Slab<MirBlock>) -> &BTreeSet<MirBlockRef> {
        let block = self.as_data(alloc).expect("Invalid MirBlockRef");
        &block.predecessors
    }
    pub fn has_predecessors(self, alloc: &Slab<MirBlock>) -> bool {
        let block = self.as_data(alloc).expect("Invalid MirBlockRef");
        !block.predecessors.is_empty()
    }
}
