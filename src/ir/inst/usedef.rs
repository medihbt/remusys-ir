use std::cell::Cell;

use slab::Slab;

use crate::base::slablist::{SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef};
use crate::base::slabref::SlabRef;
use crate::base::NullableValue;
use crate::ir::ValueRef;

use super::InstRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UseRef(usize);

pub struct UseData {
    pub node_head: Cell<SlabRefListNodeHead>,
    pub operand:   Cell<Option<ValueRef>>,
    pub user:      InstRef,
}

impl UseRef {
    pub fn get_oprerand_ref(&self, alloc: &Slab<UseData>) -> Option<ValueRef> {
        self.to_slabref(alloc)
            .expect("Invalid reference (UAF)")
            .operand
            .get()
    }
    pub fn set_oprerand_ref(&self, alloc: &Slab<UseData>, value: Option<ValueRef>) {
        self.to_slabref(alloc)
            .expect("Invalid reference (UAF)")
            .operand
            .set(value);
    }
}

impl SlabRef for UseRef {
    type Item = UseData;

    fn from_handle(handle: usize) -> Self { UseRef(handle) }
    fn get_handle (&self) -> usize { self.0 }
}

impl SlabRefListNodeRef for UseRef {}

impl UseData {
    pub fn new(user: InstRef) -> Self {
        Self {
            node_head:  Cell::new(SlabRefListNodeHead::new()),
            operand:    Cell::new(None),
            user,
        }
    }
    pub fn new_with_operand(user: InstRef, operand: ValueRef) -> Self {
        Self {
            node_head: Cell::new(SlabRefListNodeHead::new()),
            operand:   Cell::new(Some(operand)),
            user,
        }
    }
}

impl SlabRefListNode for UseData {
    fn new_guide() -> Self {
        Self { node_head: Cell::new(SlabRefListNodeHead::new()), operand: Cell::new(None), user: InstRef::new_null() }
    }

    fn load_node_head(&self) -> SlabRefListNodeHead {
        self.node_head.get()
    }

    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        self.node_head.set(node_head);
    }
}