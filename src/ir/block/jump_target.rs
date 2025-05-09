//! Edges in control flow graph (CFG) are represented as jump targets.

use std::cell::Cell;

use slab::Slab;

use crate::{
    base::{
        slablist::{SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef}, slabref::SlabRef, NullableValue
    }, impl_slabref, ir::inst::InstRef
};

use super::BlockRef;

pub enum JumpTargetKind {
    None,
    Jump,
    BrTrue,
    BrFalse,
    SwitchDefault,
    SwitchCase(i128)
}

pub struct JumpTargetData {
    pub(crate) _node_head: Cell<SlabRefListNodeHead>,
    pub(crate) _terminator: InstRef,
    pub(crate) _block: Cell<BlockRef>,
    pub(crate) _kind:  JumpTargetKind
}

impl SlabRefListNode for JumpTargetData {
    fn new_guide() -> Self {
        Self::new_with_kind(JumpTargetKind::None)
    }

    fn load_node_head(&self) -> SlabRefListNodeHead {
        self._node_head.get()
    }

    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        self._node_head.set(node_head);
    }
}

impl JumpTargetData {
    pub fn new_with_kind(kind: JumpTargetKind) -> Self {
        Self {
            _node_head: Cell::new(SlabRefListNodeHead::new()),
            _terminator: InstRef::new_null(),
            _block: Cell::new(BlockRef::new_null()),
            _kind: kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpTargetRef(usize);
impl_slabref!(JumpTargetRef, JumpTargetData);
impl SlabRefListNodeRef for JumpTargetRef {}

impl JumpTargetRef {
    pub fn get_block(&self, jt_alloc: &Slab<JumpTargetData>) -> BlockRef {
        self.to_slabref_unwrap(jt_alloc)
            ._block.get()
    }
    pub fn set_block(&self, jt_alloc: &Slab<JumpTargetData>, block: BlockRef) {
        self.to_slabref_unwrap(jt_alloc)
            ._block.set(block);
    }
}