//! Load and Store instructions

use std::{cell::Cell, num::NonZero};

use slab::Slab;

use crate::{
    ir::{PtrUser, module::Module},
    typing::id::ValTypeID,
};

use super::{
    InstDataCommon, InstDataUnique, InstError,
    checking::check_operand_type_match,
    usedef::{UseData, UseRef},
};

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

impl InstDataUnique for LoadOp {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self.source = common.alloc_use(alloc_use)
    }

    fn check_operands(&self, _: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let source = self.source.get_operand(&module.borrow_use_alloc());
        check_operand_type_match(ValTypeID::Ptr, source, module)
    }
}

impl InstDataUnique for StoreOp {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self.source = common.alloc_use(alloc_use);
        self.target = common.alloc_use(alloc_use);
    }

    fn check_operands(&self, _: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let alloc_use = module.borrow_use_alloc();
        let source = self.source.get_operand(&alloc_use);
        let target = self.target.get_operand(&alloc_use);

        /* target is a pointer whose pointee type should be `target_ty`.
           So type of `self.source` is `self.target_ty`. */
        check_operand_type_match(ValTypeID::Ptr, target, module)?;
        check_operand_type_match(self.target_ty, source, module)
    }
}
