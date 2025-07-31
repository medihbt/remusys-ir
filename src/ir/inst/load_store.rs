//! Load and Store instructions

use std::{cell::Cell, num::NonZero};

use slab::Slab;

use crate::{
    base::INullableValue,
    ir::{PtrUser, ValueSSA, module::Module, opcode::Opcode},
    typing::{TypeMismatchError, id::ValTypeID},
};

use super::{
    InstDataCommon, InstDataUnique, InstError,
    checking::check_operand_type_match,
    usedef::{UseData, UseKind, UseRef},
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
        self.source = common.alloc_use(alloc_use, UseKind::LoadSource)
    }

    fn check_operands(&self, _: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let source = self.source.get_operand(&module.borrow_use_alloc());
        check_operand_type_match(ValTypeID::Ptr, source, module)
    }
}

impl InstDataUnique for StoreOp {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self.source = common.alloc_use(alloc_use, UseKind::StoreSource);
        self.target = common.alloc_use(alloc_use, UseKind::StoreTarget);
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

impl LoadOp {
    pub fn new_raw(
        mut_module: &Module,
        source_ty: ValTypeID,
        source_align: usize,
    ) -> (InstDataCommon, Self) {
        let mut alloc_use = mut_module.borrow_use_alloc_mut();
        let mut common = InstDataCommon::new(Opcode::Load, source_ty, &mut alloc_use);
        let mut load_op = Self {
            source: UseRef::new_null(),
            source_ty,
            align: Cell::new(source_align),
        };
        load_op.build_operands(&mut common, &mut alloc_use);
        (common, load_op)
    }

    pub fn new(
        mut_module: &Module,
        source_ty: ValTypeID,
        source_align: usize,
        source: ValueSSA,
    ) -> Result<(InstDataCommon, Self), InstError> {
        if source.get_value_type(mut_module) != ValTypeID::Ptr {
            return Err(InstError::OperandTypeMismatch(
                TypeMismatchError::IDNotEqual(ValTypeID::Ptr, source.get_value_type(mut_module)),
                source,
            ));
        }
        let (common, load_op) = Self::new_raw(mut_module, source_ty, source_align);
        let alloc_use = mut_module.borrow_use_alloc();
        load_op.source.set_operand_nordfg(&alloc_use, source);
        Ok((common, load_op))
    }
}

impl StoreOp {
    pub fn new_raw(
        mut_module: &Module,
        target_ty: ValTypeID,
        target_align: usize,
    ) -> (InstDataCommon, Self) {
        let mut alloc_use = mut_module.borrow_use_alloc_mut();
        let mut common = InstDataCommon::new(Opcode::Store, ValTypeID::Void, &mut alloc_use);
        let mut store_op = Self {
            source: UseRef::new_null(),
            target: UseRef::new_null(),
            target_ty,
            align: Cell::new(target_align),
        };
        store_op.build_operands(&mut common, &mut alloc_use);
        (common, store_op)
    }

    pub fn new(
        mut_module: &Module,
        target_ty: ValTypeID,
        target_align: usize,
        source: ValueSSA,
        target: ValueSSA,
    ) -> Result<(InstDataCommon, Self), InstError> {
        if source.get_value_type(mut_module) != target_ty {
            return Err(InstError::OperandTypeMismatch(
                TypeMismatchError::IDNotEqual(target_ty, source.get_value_type(mut_module)),
                source,
            ));
        }
        let (common, store_op) = Self::new_raw(mut_module, target_ty, target_align);
        let alloc_use = mut_module.borrow_use_alloc();
        store_op.source.set_operand_nordfg(&alloc_use, source);
        store_op.target.set_operand_nordfg(&alloc_use, target);
        Ok((common, store_op))
    }
}
