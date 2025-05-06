use std::cell::Cell;

use slab::Slab;

use crate::{base::{
    slablist::{SlabRefList, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef}, slabref::SlabRef, NullableValue
}, typing::id::ValTypeID};

use super::{
    global::GlobalRef, inst::{terminator::{self, TerminatorInstView}, Inst, InstCommon, InstRef}, opcode::Opcode, Module
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockRef(pub(crate) usize);

pub struct BlockData {
    pub(crate) _node_head:        Cell<SlabRefListNodeHead>,
    pub(crate) _instruction_list: SlabRefList<InstRef>,
    pub(crate) _parent_func:      GlobalRef,
}

impl SlabRef for BlockRef {
    type Item = BlockData;

    fn from_handle(handle: usize) -> Self {
        BlockRef(handle)
    }

    fn get_handle(&self) -> usize {
        self.0
    }
}
impl SlabRefListNodeRef for BlockRef {}
impl BlockRef {
    pub fn new_unreachable(module: &mut Module, void_ty: ValTypeID) -> BlockRef {
        let block_data = BlockData::new(&mut module._alloc_inst);
        let block = module.alloc_block(block_data);

        let inst = Inst::Unreachable(
            InstCommon::new(Opcode::Unreachable, void_ty, block.clone(), module),
            terminator::Unreachable
        );
        let inst_ref = module.alloc_inst(inst);
        block.to_slabref_mut(&mut module._alloc_block)
             .expect("Invalid block reference (Use after free?)")
             ._instruction_list
             .push_back_ref(&mut module._alloc_inst, inst_ref)
             .expect("Failed to push back terminator instruction");
        block
    }
}

impl BlockData {
    pub fn new(inst_alloc: &mut Slab<Inst>) -> Self {
        Self {
            _node_head:         Cell::new(SlabRefListNodeHead::new()),
            _instruction_list:  SlabRefList::from_slab(inst_alloc),
            _parent_func:       GlobalRef::new_null(),
        }
    }

    pub fn view_terminator<'a>(&'a self, inst_alloc: &'a Slab<Inst>) -> Option<TerminatorInstView<'a>> {
        self._instruction_list
            .get_back_ref(inst_alloc)
            .map(|id| TerminatorInstView::from_inst(id, inst_alloc))
            .flatten()
    }
    pub fn set_terminator_ref(&self, inst_ref: InstRef, inst_alloc: &mut Slab<Inst>) -> Option<InstRef> {
        Self::_check_inst_to_be_terminator(inst_ref, inst_alloc);
        let instructions = &self._instruction_list;
        if instructions.is_empty() {
            instructions.push_back_ref(inst_alloc, inst_ref)
                        .expect("Failed to push back terminator instruction");
            return None;
        }
        
        let last_inst = instructions.get_back_ref(inst_alloc).unwrap();
        if last_inst == inst_ref {
            return None;
        }

        if last_inst.to_slabref(inst_alloc).unwrap().is_terminator() {
            let ret = instructions.pop_back(inst_alloc)
                        .expect("Failed to pop back terminator instruction");
            instructions.push_back_ref(inst_alloc, inst_ref)
                        .expect("Failed to push back terminator instruction");
            Some(ret)
        } else {
            instructions.push_back_ref(inst_alloc, inst_ref)
                        .expect("Failed to push back terminator instruction");
            None
        }
    }

    fn _check_inst_to_be_terminator(inst_ref: InstRef, inst_alloc: &Slab<Inst>) {
        let inst = inst_ref.to_slabref(inst_alloc)
                           .expect("Invalid instruction reference (Use after free?)");
        if !inst.is_terminator() {
            panic!("Instruction is not a terminator");
        }
    }
}

impl SlabRefListNode for BlockData {
    fn new_guide() -> Self {
        Self {
            _node_head: Cell::new(SlabRefListNodeHead::new()),
            _instruction_list: SlabRefList::new_guide(),
            _parent_func: GlobalRef::new_null(),
        }
    }

    fn load_node_head(&self) -> SlabRefListNodeHead {
        self._node_head.get()
    }

    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        self._node_head.set(node_head);
    }
}
