//! GerElemPtr Instruction.

use std::num::NonZero;

use slab::Slab;

use crate::{
    ir::{PtrStorage, PtrUser, module::Module},
    typing::id::ValTypeID,
};

use super::{
    InstDataCommon, InstDataUnique, InstError,
    usedef::{UseData, UseRef},
};

pub struct IndexPtrOp {
    pub base_ptr: UseRef,
    pub base_pointee_ty: ValTypeID,
    pub ret_pointee_ty: ValTypeID,

    pub base_pointee_align: usize,
    pub ret_pointee_align: usize,

    pub n_deref_levels: usize,
    pub indices: Box<[UseRef]>,
}

impl PtrStorage for IndexPtrOp {
    fn get_stored_pointee_type(&self) -> ValTypeID {
        self.ret_pointee_ty
    }
}

impl PtrUser for IndexPtrOp {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        self.base_pointee_ty
    }

    fn get_operand_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(self.base_pointee_align)
    }
}

impl InstDataUnique for IndexPtrOp {
    /// Assume that we've known the actual levels of indexing.
    /// Without this assumption, it is impossible for us to allocate
    /// use edges for this instruction.
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        let mut indices = Vec::with_capacity(self.n_deref_levels);
        self.base_ptr = common.alloc_use(alloc_use);
        for _ in 0..self.n_deref_levels {
            indices.push(common.alloc_use(alloc_use));
        }
        self.indices = indices.into_boxed_slice();
    }

    fn check_operands(&self, _: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        todo!("Check base_ptr and then indices")
    }
}
