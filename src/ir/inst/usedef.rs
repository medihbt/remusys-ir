use std::cell::Cell;

use slab::Slab;

use crate::{
    base::{
        NullableValue,
        slablist::{SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef},
        slabref::SlabRef,
    },
    impl_slabref,
    ir::ValueSSA,
};

use super::InstRef;

pub struct UseData {
    pub(crate) _node_head: Cell<SlabRefListNodeHead>,
    pub(crate) _operand: Cell<ValueSSA>,
    pub(crate) _user: InstRef,
}

impl SlabRefListNode for UseData {
    fn new_guide() -> Self {
        Self {
            _node_head: Cell::new(SlabRefListNodeHead::new()),
            _user: InstRef::new_null(),
            _operand: Cell::new(ValueSSA::None),
        }
    }

    fn load_node_head(&self) -> SlabRefListNodeHead {
        self._node_head.get()
    }

    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        self._node_head.set(node_head);
    }
}

impl UseData {
    pub fn new(parent: InstRef, operand: ValueSSA) -> Self {
        Self {
            _node_head: Cell::new(SlabRefListNodeHead::new()),
            _user: parent,
            _operand: Cell::new(operand),
        }
    }

    pub fn get_user(&self) -> InstRef {
        self._user
    }

    pub fn get_operand(&self) -> ValueSSA {
        self._operand.get()
    }

    pub fn set_operand(&self, operand: ValueSSA) {
        self._operand.set(operand);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseRef(usize);
impl_slabref!(UseRef, UseData);
impl SlabRefListNodeRef for UseRef {}

impl UseRef {
    pub fn get_user(&self, alloc: &Slab<UseData>) -> InstRef {
        self.to_slabref_unwrap(alloc).get_user()
    }
    pub fn get_operand(&self, alloc: &Slab<UseData>) -> ValueSSA {
        self.to_slabref_unwrap(alloc).get_operand()
    }
    pub fn set_operand(&self, alloc: &Slab<UseData>, operand: ValueSSA) {
        self.to_slabref_unwrap(alloc).set_operand(operand);
    }
}
