use std::cell::Cell;

use slab::Slab;

use crate::{
    base::{
        slablist::{SlabRefList, SlabRefListError, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef}, NullableValue
    },
    impl_slabref,
};

use super::{
    global::GlobalRef,
    inst::{InstData, InstDataCommon, InstRef},
    module::Module,
};

pub mod jump_target;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRef(usize);

impl_slabref!(BlockRef, BlockData);

impl SlabRefListNodeRef for BlockRef {}

/// Basic block data.
pub struct BlockData {
    pub insructions: SlabRefList<InstRef>,
    pub phi_node_end: Cell<InstRef>,
    pub(super) _entry: Cell<InstRef>,
    pub(super) _inner: Cell<BlockDataInner>,
}

#[derive(Debug, Clone, Copy)]
pub struct BlockDataInner {
    pub(super) _node_head: SlabRefListNodeHead,
    pub(super) _parent_func: GlobalRef,
    pub(super) _id: usize,
}

impl BlockDataInner {
    fn insert_node_head(self, node_head: SlabRefListNodeHead) -> Self {
        Self {
            _node_head: node_head,
            _parent_func: self._parent_func,
            _id: self._id,
        }
    }
    fn insert_parent_func(self, parent_func: GlobalRef) -> Self {
        Self {
            _node_head: self._node_head,
            _parent_func: parent_func,
            _id: self._id,
        }
    }
    fn insert_id(self, id: usize) -> Self {
        Self {
            _node_head: self._node_head,
            _parent_func: self._parent_func,
            _id: id,
        }
    }
    fn assign_to(&self, cell: &Cell<BlockDataInner>) {
        cell.set(*self);
    }
}

impl SlabRefListNode for BlockData {
    fn new_guide() -> Self {
        Self {
            insructions: SlabRefList::new_guide(),
            phi_node_end: Cell::new(InstRef::new_null()),
            _entry: Cell::new(InstRef::new_null()),
            _inner: Cell::new(BlockDataInner {
                _node_head: SlabRefListNodeHead::new(),
                _parent_func: GlobalRef::new_null(),
                _id: 0,
            }),
        }
    }

    fn load_node_head(&self) -> SlabRefListNodeHead {
        self._inner.get()._node_head
    }

    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        self._inner
            .get()
            .insert_node_head(node_head)
            .assign_to(&self._inner);
    }
}

impl BlockData {
    pub fn get_parent_func(&self) -> GlobalRef {
        self._inner.get()._parent_func
    }
    pub fn set_parent_func(&self, parent_func: GlobalRef) {
        self._inner
            .get()
            .insert_parent_func(parent_func)
            .assign_to(&self._inner);
    }

    pub fn get_id(&self) -> usize {
        self._inner.get()._id
    }
    pub fn set_id(&self, id: usize) {
        self._inner.get().insert_id(id).assign_to(&self._inner);
    }

    pub fn build_add_inst(&self, inst: InstRef) {
        todo!("build add inst");
    }
    pub fn build_add_phi(&self, inst: InstRef) {
        todo!("build add phi");
    }
    pub fn get_entry(&self) -> InstRef {
        self._entry.get()
    }
    pub fn set_entry(&self, alloc_inst: &Slab<InstData>, inst: InstRef) {
        todo!("set entry");
    }
}

impl BlockData {
    pub fn new_empty(module: &Module) -> Self {
        Self {
            insructions: SlabRefList::from_slab(&mut module.borrow_value_alloc_mut()._alloc_inst),
            phi_node_end: Cell::new(InstRef::new_null()),
            _entry: Cell::new(InstRef::new_null()),
            _inner: Cell::new(BlockDataInner {
                _node_head: SlabRefListNodeHead::new(),
                _parent_func: GlobalRef::new_null(),
                _id: 0,
            }),
        }
    }

    pub fn new_unreachable(module: &Module) -> Result<Self, SlabRefListError> {
        let ret = Self::new_empty(module);
        ret.insructions.push_back_value(
            &mut module.borrow_value_alloc_mut()._alloc_inst,
            InstData::new_unreachable(),
        )?;
        Ok(ret)
    }

    pub fn new_return_zero(module: &Module) -> Result<Self, SlabRefListError> {
        todo!("new return zero");
    }
}
