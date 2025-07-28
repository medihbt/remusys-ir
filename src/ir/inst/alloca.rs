use std::num::NonZero;

use super::{InstDataCommon, InstDataUnique, InstError, usedef::UseData};
use crate::{
    ir::{PtrStorage, module::Module, opcode::Opcode},
    typing::id::ValTypeID,
};
use slab::Slab;

/// Allocate a piece of fixed-size memory on the stack.
/// The allocation returns a pointer to the allocated memory, which is
/// the only mutable part in the IR system.
///
/// To keep it simple, Remusys does not support `VLA`-like stack
/// allocation, or `alloca()` function. If required, Remusys will
/// introduce `DynAlloca` and `DynDropAlloca` unary instruction
/// to handle dynamic stack allocation.
pub struct Alloca {
    pub pointee_ty: ValTypeID,
    pub align_log2: u8,
}

impl InstDataUnique for Alloca {
    fn build_operands(&mut self, _: &mut InstDataCommon, _: &mut Slab<UseData>) {}
    fn check_operands(&self, _: &InstDataCommon, _: &Module) -> Result<(), InstError> {
        /* No operand */
        Ok(())
    }
}

impl PtrStorage for Alloca {
    fn get_stored_pointee_type(&self) -> ValTypeID {
        self.pointee_ty
    }
    fn get_stored_pointee_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(1 << self.align_log2)
    }
}

impl Alloca {
    pub fn from_alloc_use(
        alloc_use: &mut Slab<UseData>,
        pointee_ty: ValTypeID,
        align_log2: u8,
    ) -> (Self, InstDataCommon) {
        let mut inst = Self { pointee_ty, align_log2 };
        let mut common = InstDataCommon::new(Opcode::Alloca, ValTypeID::Ptr, alloc_use);
        inst.build_operands(&mut common, alloc_use);
        (inst, common)
    }
    pub fn from_module(
        module: &Module,
        pointee_ty: ValTypeID,
        align_log2: u8,
    ) -> (Self, InstDataCommon) {
        Self::from_alloc_use(&mut module.borrow_use_alloc_mut(), pointee_ty, align_log2)
    }
}
