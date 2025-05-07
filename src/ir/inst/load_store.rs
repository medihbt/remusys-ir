//! Load and Store instructions

use std::{cell::Cell, num::NonZero};

use crate::{ir::PtrUser, typing::id::ValTypeID};

use super::usedef::UseRef;

pub struct LoadOp {
    pub source: UseRef,
    pub source_ty: ValTypeID,
    pub align: Cell<usize>,
}

pub struct StoreOp {
    pub source: UseRef,
    pub target: UseRef,
    pub target_ty: ValTypeID,
    pub align: Cell<usize>,
}

impl PtrUser for LoadOp {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        self.source_ty
    }

    fn get_operand_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(self.align.get())
    }
}

impl PtrUser for StoreOp {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        self.target_ty
    }

    fn get_operand_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(self.align.get())
    }
}