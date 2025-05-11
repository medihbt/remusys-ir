//! Edges in control flow graph (CFG) are represented as jump targets.

use std::cell::Cell;

use slab::Slab;

use crate::{
    base::{
        NullableValue,
        slablist::{
            SlabRefList, SlabRefListError, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef,
        },
        slabref::SlabRef,
    },
    impl_slabref,
    ir::inst::InstRef,
};

use super::BlockRef;

pub enum JumpTargetKind {
    None,
    Jump,
    BrTrue,
    BrFalse,
    SwitchDefault,
    SwitchCase(i128),
}

pub struct JumpTargetData {
    pub(crate) _node_head: Cell<SlabRefListNodeHead>,
    pub(crate) _terminator: Cell<InstRef>,
    pub(crate) _block: Cell<BlockRef>,
    pub(crate) _kind: JumpTargetKind,
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
            _terminator: Cell::new(InstRef::new_null()),
            _block: Cell::new(BlockRef::new_null()),
            _kind: kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpTargetRef(usize);
impl_slabref!(JumpTargetRef, JumpTargetData);
impl SlabRefListNodeRef for JumpTargetRef {
    fn on_node_push_next(
        _: Self,
        _: Self,
        _: &Slab<JumpTargetData>,
    ) -> Result<(), SlabRefListError> {
        Ok(())
    }

    fn on_node_push_prev(
        _: Self,
        _: Self,
        _: &Slab<JumpTargetData>,
    ) -> Result<(), SlabRefListError> {
        Ok(())
    }

    fn on_node_unplug(_: Self, _: &Slab<JumpTargetData>) -> Result<(), SlabRefListError> {
        Ok(())
    }
}

impl JumpTargetRef {
    pub fn get_block(&self, jt_alloc: &Slab<JumpTargetData>) -> BlockRef {
        self.to_slabref_unwrap(jt_alloc)._block.get()
    }
    pub fn set_block(&self, jt_alloc: &Slab<JumpTargetData>, block: BlockRef) {
        self.to_slabref_unwrap(jt_alloc)._block.set(block);
    }

    pub fn get_terminator(&self, jt_alloc: &Slab<JumpTargetData>) -> InstRef {
        self.to_slabref_unwrap(jt_alloc)._terminator.get()
    }
}

impl SlabRefList<JumpTargetRef> {
    pub fn push_back_value(
        &self,
        alloc: &mut Slab<JumpTargetData>,
        jt_data: JumpTargetData,
    ) -> Result<JumpTargetRef, SlabRefListError> {
        let terminator = self._head.to_slabref_unwrap(alloc)._terminator.get();
        jt_data._terminator.set(terminator);
        let jt_ref = JumpTargetRef::from_handle(alloc.insert(jt_data));
        self.push_back_ref(alloc, jt_ref).map(|_| jt_ref)
    }
}
