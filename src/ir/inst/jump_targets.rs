use std::cell::Cell;

use crate::{base::{slablist::{SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef}, slabref::SlabRef, NullableValue}, ir::block::BlockRef};

use super::InstRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct JumpTargetRef(pub(crate) usize);

pub struct JumpTargetData {
    pub node_head:  Cell<SlabRefListNodeHead>,
    pub terminator: InstRef,
    pub target:     BlockRef,
}

impl SlabRef for JumpTargetRef {
    type RefObject = JumpTargetData;

    fn from_handle(handle: usize) -> Self {
        Self(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}
impl SlabRefListNodeRef for JumpTargetRef {}

impl JumpTargetData {
}
impl SlabRefListNode for JumpTargetData {
    fn new_guide() -> Self {
        Self {
            node_head:  Cell::new(SlabRefListNodeHead::new()),
            terminator: InstRef::new_null(),
            target:     BlockRef::new_null()
        }
    }

    fn load_node_head(&self) -> SlabRefListNodeHead {
        self.node_head.get()
    }
    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        self.node_head.set(node_head);
    }
}
