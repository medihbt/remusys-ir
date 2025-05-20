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
    ir::{
        inst::InstRef,
        module::{Module, rcfg::RcfgAlloc},
    },
};

use super::BlockRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn get_kind(&self) -> JumpTargetKind {
        self._kind
    }
    pub fn get_block(&self) -> BlockRef {
        self._block.get()
    }
    pub fn set_block_norcfg(&self, block: BlockRef) {
        self._block.set(block);
    }
    pub fn set_block_with_rcfg(&self, handle: JumpTargetRef, rcfg: &RcfgAlloc, block: BlockRef) {
        let prev = self._block.get();
        if prev == block {
            return;
        }
        self._block.set(block);
        if prev.is_nonnull() {
            rcfg.get_node(prev).remove_predecessor(handle);
        }
        if block.is_nonnull() {
            rcfg.get_node(block).add_predecessor(handle);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct JumpTargetRef(usize);
impl_slabref!(JumpTargetRef, JumpTargetData);
impl SlabRefListNodeRef for JumpTargetRef {
    fn on_node_push_next(
        curr: Self,
        next: Self,
        alloc: &Slab<JumpTargetData>,
    ) -> Result<(), SlabRefListError> {
        let terminator = curr.to_slabref_unwrap(alloc)._terminator.get();
        next.to_slabref_unwrap(alloc)._terminator.set(terminator);
        Ok(())
    }

    fn on_node_push_prev(
        curr: Self,
        prev: Self,
        alloc: &Slab<JumpTargetData>,
    ) -> Result<(), SlabRefListError> {
        let terminator = curr.to_slabref_unwrap(alloc)._terminator.get();
        prev.to_slabref_unwrap(alloc)._terminator.set(terminator);
        Ok(())
    }

    fn on_node_unplug(curr: Self, alloc: &Slab<JumpTargetData>) -> Result<(), SlabRefListError> {
        curr.to_slabref_unwrap(alloc)
            ._terminator
            .set(InstRef::new_null());
        Ok(())
    }
}

impl JumpTargetRef {
    pub fn get_block(&self, alloc_jt: &Slab<JumpTargetData>) -> BlockRef {
        self.to_slabref_unwrap(alloc_jt)._block.get()
    }
    pub fn set_block_norcfg(&self, alloc_jt: &Slab<JumpTargetData>, block: BlockRef) {
        self.to_slabref_unwrap(alloc_jt)._block.set(block);
    }
    pub fn set_block(&self, module: &Module, block: BlockRef) {
        let alloc_jt = module.borrow_jt_alloc();
        let rcfg = match module.borrow_rcfg_alloc() {
            Some(rcfg) => rcfg,
            None => {
                self.set_block_norcfg(&alloc_jt, block);
                return;
            }
        };
        self.to_slabref_unwrap(&alloc_jt)
            .set_block_with_rcfg(self.clone(), &rcfg, block);
    }

    pub fn get_terminator(&self, alloc_jt: &Slab<JumpTargetData>) -> InstRef {
        self.to_slabref_unwrap(alloc_jt)._terminator.get()
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
