use std::cell::Cell;

use crate::base::slablist::SlabRefListNodeHead;
use crate::base::slabref::SlabRef;
use crate::ir::ValueRef;

use super::InstRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UseRef(usize);

pub struct UseData {
    pub node_head: Cell<SlabRefListNodeHead>,
    pub operand:   Cell<Option<ValueRef>>,
    pub user:      InstRef,
}

impl SlabRef for UseRef {
    type Item = UseData;

    fn from_handle(handle: usize) -> Self { UseRef(handle) }
    fn get_handle (&self) -> usize { self.0 }
}

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