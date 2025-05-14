use std::cell::Cell;

use slab::Slab;

use crate::{
    base::{
        NullableValue,
        slablist::{SlabRefListError, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef},
        slabref::SlabRef,
    },
    impl_slabref,
    ir::{
        ValueSSA,
        module::{Module, ModuleError, rdfg::RdfgAlloc},
    },
};

use super::InstRef;

pub struct UseData {
    pub(crate) _node_head: Cell<SlabRefListNodeHead>,
    pub(crate) _operand: Cell<ValueSSA>,
    pub(crate) _user: Cell<InstRef>,
}

impl SlabRefListNode for UseData {
    fn new_guide() -> Self {
        Self {
            _node_head: Cell::new(SlabRefListNodeHead::new()),
            _user: Cell::new(InstRef::new_null()),
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
            _user: Cell::new(parent),
            _operand: Cell::new(operand),
        }
    }

    pub fn get_user(&self) -> InstRef {
        self._user.get()
    }

    pub fn get_operand(&self) -> ValueSSA {
        self._operand.get()
    }

    pub fn set_operand_nordfg(&self, operand: ValueSSA) {
        self._operand.set(operand);
    }
    pub fn set_operand_with_rdfg(&self, selfref: UseRef, rdfg: &RdfgAlloc, operand: ValueSSA) {
        let old_value = self._operand.get();
        if old_value == operand {
            return;
        }
        self._operand.set(operand);

        if old_value.is_nonnull() {
            rdfg.get_node(old_value).remove_user_use(selfref);
        }
        if operand.is_nonnull() {
            rdfg.get_node(operand).add_user_use(selfref);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseRef(usize);
impl_slabref!(UseRef, UseData);
impl SlabRefListNodeRef for UseRef {
    fn on_node_push_next(_: Self, _: Self, _: &Slab<UseData>) -> Result<(), SlabRefListError> {
        Ok(())
    }

    fn on_node_push_prev(_: Self, _: Self, _: &Slab<UseData>) -> Result<(), SlabRefListError> {
        Ok(())
    }

    fn on_node_unplug(_: Self, _: &Slab<UseData>) -> Result<(), SlabRefListError> {
        Ok(())
    }
}

impl UseRef {
    pub fn get_user(&self, alloc: &Slab<UseData>) -> InstRef {
        self.to_slabref_unwrap(alloc).get_user()
    }

    /// Get the operand of this use reference.
    pub fn get_operand(&self, alloc: &Slab<UseData>) -> ValueSSA {
        self.to_slabref_unwrap(alloc).get_operand()
    }

    /// Set the operand of this use reference regardless of the def-use graph.
    /// This method does not update the def-use graph.
    pub fn set_operand_nordfg(&self, alloc: &Slab<UseData>, operand: ValueSSA) {
        self.to_slabref_unwrap(alloc).set_operand_nordfg(operand);
    }

    /// Set the operand of this use reference and update the def-use graph.
    pub fn set_operand(&self, module: &Module, operand: ValueSSA) {
        // Update the operand of this use reference.
        let use_alloc = module.borrow_use_alloc();
        let self_data = self.to_slabref_unwrap(&*use_alloc);
        let old_value = self_data.get_operand();
        if old_value == operand {
            return;
        }
        self_data.set_operand_nordfg(operand);

        // Now update the def-use reverse graph (RDFG).
        if !old_value.is_none() {
            Self::_handle_setop_err(module.operand_del_use(old_value, self.clone()));
        }
        if !operand.is_none() {
            Self::_handle_setop_err(module.operand_add_use(operand, self.clone()));
        }
    }

    fn _handle_setop_err(res: Result<(), ModuleError>) {
        match res {
            Ok(_) => { /* Successfully inserted or deleted the use reference. */ }
            Err(ModuleError::DfgOperandNotReferece(..)) | Err(ModuleError::RDFGNotEnabled) => {
                /* Normal cases where the value is not a reference type or the RDFG is not enabled. */
            }
            _ => panic!(),
        }
    }
}
