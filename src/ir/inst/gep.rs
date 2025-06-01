//! GerElemPtr Instruction.

use std::num::NonZero;

use slab::Slab;

use crate::{
    base::NullableValue,
    ir::{PtrStorage, PtrUser, ValueSSA, module::Module, opcode::Opcode},
    typing::{TypeMismatchError, context::TypeContext, id::ValTypeID, types::StructTypeRef},
};

use super::{
    InstDataCommon, InstDataUnique, InstError,
    checking::{
        check_operand_integral_const, check_operand_type_kind_match, check_operand_type_match,
    },
    usedef::{UseData, UseKind, UseRef},
};

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

    fn get_stored_pointee_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(self.ret_pointee_align)
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
        // Set the base pointer.
        self.base_ptr = common.alloc_use(alloc_use, UseKind::GepBase);

        // Set the indices.
        for (i, index) in self.indices.iter_mut().enumerate() {
            *index = common.alloc_use(alloc_use, UseKind::GepIndex(i));
        }
    }

    /// This function traverses offset type chain to check the base pointer and indices.
    fn check_operands(&self, _: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        // Check the base pointer.
        let base_ptr = self.base_ptr.get_operand(&module.borrow_use_alloc());
        check_operand_type_match(ValTypeID::Ptr, base_ptr, module)?;

        // Go throuth the offset type chain to check indices.
        let base_ty = self.base_pointee_ty;
        let final_ty = self.ret_pointee_ty;

        // Check the layer 0..N indices (0 and i..N).
        let layer_n_ty = Self::unpack_indices_get_final_type(
            module,
            &module.type_ctx,
            base_ty,
            self.indices
                .iter()
                .map(|u| u.get_operand(&module.borrow_use_alloc())),
        )?;

        // Check the return type.
        if layer_n_ty == final_ty {
            Ok(())
        } else {
            Err(InstError::OperandTypeMismatch(
                TypeMismatchError::IDNotEqual(layer_n_ty, final_ty),
                ValueSSA::None,
            ))
        }
    }
}

impl IndexPtrOp {
    fn check_struct_layer(
        type_ctx: &TypeContext,
        layer_n_ty: StructTypeRef,
        idx_value: ValueSSA,
    ) -> Result<ValTypeID, InstError> {
        let (binbits, value) = check_operand_integral_const(idx_value)?;
        let value = (value as usize) & ((1 << binbits) - 1);
        match layer_n_ty.get_element_type(type_ctx, value) {
            Some(ty) => Ok(ty),
            None => Err(InstError::OperandOverflow),
        }
    }

    /// Unpack the aggregate layers of the given type.
    /// If the type is an array, the function returns the element type regardless of the index.
    /// If the type is a struct or a struct alias, the function checks the index and returns the element type.
    fn unpack_aggregate_layers_iter(
        module: &Module,
        type_ctx: &TypeContext,
        before_unpack_ty: ValTypeID,
        n_layer_index: ValueSSA,
    ) -> Result<ValTypeID, InstError> {
        match before_unpack_ty {
            ValTypeID::Array(arrref) => {
                check_operand_type_kind_match(ValTypeID::Int(0), n_layer_index, module)?;
                Ok(arrref.get_element_type(type_ctx))
            }
            ValTypeID::Struct(sref) => Self::check_struct_layer(type_ctx, sref, n_layer_index),
            ValTypeID::StructAlias(sa) => {
                // Check struct alias: prohibits overflow and return the element type at the right
                // constant integral index.
                let sref = sa.get_aliasee(type_ctx);
                Self::check_struct_layer(type_ctx, sref, n_layer_index)
            }
            _ => {
                return Err(InstError::OperandTypeMismatch(
                    TypeMismatchError::NotAggregate(before_unpack_ty),
                    n_layer_index,
                ));
            }
        }
    }

    /// Unpack the indices and go to the final type.
    /// The first index is the base pointer regarded as an array with unknown size.
    fn unpack_indices_get_final_type(
        module: &Module,
        type_ctx: &TypeContext,
        base_ty: ValTypeID,
        mut indices_with_layer0: impl Iterator<Item = ValueSSA>,
    ) -> Result<ValTypeID, InstError> {
        // Layer 0: check the base pointer.
        match indices_with_layer0.next() {
            Some(layer0) => check_operand_type_kind_match(ValTypeID::Int(0), layer0, module)?,
            None => return Err(InstError::OperandUninit),
        };

        // Layer 1..N: check the indices.
        let mut layer_n_ty = base_ty;
        for idx_value in indices_with_layer0 {
            layer_n_ty =
                Self::unpack_aggregate_layers_iter(module, type_ctx, layer_n_ty, idx_value)?;
        }
        Ok(layer_n_ty)
    }

    pub fn new_raw(
        mut_module: &Module,
        base_pointee_ty: ValTypeID,
        ret_pointee_ty: ValTypeID,
        base_pointee_align: usize,
        ret_pointee_align: usize,
        n_indices: usize,
    ) -> Result<(InstDataCommon, Self), InstError> {
        let mut common = InstDataCommon::new(
            Opcode::IndexPtr,
            ValTypeID::Ptr,
            &mut mut_module.borrow_use_alloc_mut(),
        );
        let mut ret = Self {
            base_ptr: UseRef::new_null(),
            base_pointee_ty,
            ret_pointee_ty,
            base_pointee_align,
            ret_pointee_align,
            indices: vec![UseRef::new_null(); n_indices].into_boxed_slice(),
        };
        ret.build_operands(&mut common, &mut mut_module.borrow_use_alloc_mut());
        Ok((common, ret))
    }

    pub fn new_from_indices(
        mut_module: &Module,
        base_pointee_ty: ValTypeID,
        base_pointee_align: usize,
        ret_pointee_align: usize,
        base_ptr: ValueSSA,
        indices: impl Iterator<Item = ValueSSA> + Clone,
    ) -> Result<(InstDataCommon, Self), InstError> {
        let ret_type = Self::unpack_indices_get_final_type(
            mut_module,
            &mut_module.type_ctx,
            base_pointee_ty,
            indices.clone(),
        )?;
        let (common, ret) = Self::new_raw(
            mut_module,
            base_pointee_ty,
            ret_type,
            base_pointee_align,
            ret_pointee_align,
            indices.clone().count(),
        )?;
        let alloc_use = mut_module.borrow_use_alloc();

        // Set the base pointer.
        ret.base_ptr.set_operand_nordfg(&alloc_use, base_ptr);

        // Set the indices.
        for (useref, idxvalue) in ret.indices.iter().zip(indices) {
            useref.set_operand_nordfg(&alloc_use, idxvalue);
        }

        Ok((common, ret))
    }
}
