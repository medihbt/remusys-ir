//! GerElemPtr Instruction.

use std::num::NonZero;

use crate::{
    ir::{PtrStorage, PtrUser},
    typing::id::ValTypeID,
};

use super::usedef::UseRef;

pub struct IndexPtrOp {
    pub base_ptr: UseRef,
    pub base_pointee_ty: ValTypeID,
    pub ret_pointee_ty: ValTypeID,
    pub base_pointee_align: usize,
    pub ret_pointee_align: usize,
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
