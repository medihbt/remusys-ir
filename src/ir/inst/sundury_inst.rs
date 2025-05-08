//! Instructions that do not fit into the other categories.

use slab::Slab;

use crate::{ir::opcode::Opcode, typing::id::ValTypeID};

use super::{
    InstDataUnique,
    usedef::{UseData, UseRef},
};

pub struct SelectOp {
    pub cond: UseRef,
    pub true_val: UseRef,
    pub false_val: UseRef,
}

impl InstDataUnique for SelectOp {
    fn build_operands(&mut self, common: &mut super::InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self.cond      = common.alloc_use(alloc_use);
        self.true_val  = common.alloc_use(alloc_use);
        self.false_val = common.alloc_use(alloc_use);
    }
}
