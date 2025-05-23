use slab::Slab;
use std::cell::Ref;

use super::{IRGraphEdge, IRGraphEdgeHolder, IRGraphNode};
use crate::{
    base::{slablist::SlabRefList, slabref::SlabRef},
    ir::{
        block::{
            BlockData, BlockRef,
            jump_target::{JumpTargetData, JumpTargetRef},
        },
        inst::{
            InstData,
            terminator::{TerminatorInst, TerminatorInstRef},
        },
        module::Module,
    },
};

impl IRGraphEdge for JumpTargetRef {
    type UserT = TerminatorInstRef;
    type OperandT = BlockRef;

    fn module_borrow_self_alloc<'a>(module: &'a Module) -> Ref<'a, Slab<JumpTargetData>> {
        module.borrow_jt_alloc()
    }
    fn graph_get_user_from_alloc(&self, alloc: &Slab<JumpTargetData>) -> TerminatorInstRef {
        TerminatorInstRef(self.to_slabref_unwrap(alloc)._terminator.get())
    }
    fn graph_get_operand_from_alloc(&self, alloc: &Slab<JumpTargetData>) -> BlockRef {
        self.to_slabref_unwrap(alloc).get_block()
    }
}

impl IRGraphEdgeHolder for TerminatorInstRef {
    type EdgeT = JumpTargetRef;

    fn module_borrow_edge_holder_alloc<'a>(module: &'a Module) -> Ref<'a, Slab<InstData>> {
        Ref::map(module.borrow_value_alloc(), |alloc_value| {
            &alloc_value.alloc_inst
        })
    }
    fn graph_edges_from_data<'a>(data: &'a InstData) -> Option<&'a SlabRefList<JumpTargetRef>> {
        match data {
            InstData::Jump(_, j) => j.get_jump_targets(),
            InstData::Br(_, br) => br.get_jump_targets(),
            InstData::Switch(_, sw) => sw.get_jump_targets(),
            _ => None,
        }
    }
}

impl IRGraphNode for BlockRef {
    type OperandT = BlockRef;
    type EdgeHolderT = TerminatorInstRef;
    type EdgeT = JumpTargetRef;

    fn module_borrow_self_alloc<'a>(module: &'a Module) -> Ref<'a, Slab<BlockData>> {
        Ref::map(module.borrow_value_alloc(), |alloc_value| {
            &alloc_value.alloc_block
        })
    }
    fn edge_holder_from_allocs(
        &self,
        alloc_block: &Slab<BlockData>,
        alloc_inst: &Slab<InstData>,
    ) -> TerminatorInstRef {
        self.to_slabref_unwrap(alloc_block)
            .get_terminator_subref_from_alloc(alloc_inst)
            .unwrap()
    }

    fn graph_collect_operands_from_module(
        self,
        module: &Module,
        dedup: bool,
    ) -> Vec<Self::OperandT> {
        if dedup {
            self.edge_holder_from_module(module)
                .collect_jump_blocks_from_module(module)
        } else {
            self.edge_holder_from_module(module)
                .collect_jump_blocks_from_module_nodedup(module)
        }
    }
}
