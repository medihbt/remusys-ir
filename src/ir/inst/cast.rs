use slab::Slab;

use crate::{
    base::NullableValue,
    ir::{ValueSSA, module::Module, opcode::Opcode},
    typing::{TypeMismatchError, id::ValTypeID, types::FloatTypeKind},
};

use super::{
    InstDataCommon, InstDataUnique, InstError,
    checking::check_type_kind_match,
    usedef::{UseData, UseKind, UseRef},
};

#[derive(Debug, Clone, Copy)]
pub enum CastError {
    ExtCastOperandTooLarge(ValTypeID, ValTypeID),
    TruncCastOperandTooSmall(ValTypeID, ValTypeID),
    InvalidOpcode(Opcode),
}

pub struct CastOp {
    pub from_op: UseRef,
}

impl InstDataUnique for CastOp {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self.from_op = common.alloc_use(alloc_use, UseKind::CastOpFrom);
    }

    fn check_operands(&self, common: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let alloc_use = module.borrow_use_alloc();
        let from_value = self.from_op.get_operand(&alloc_use);
        if from_value.is_null() {
            return Ok(());
        }
        let from_ty = from_value.get_value_type(module);
        let ret_ty = common.ret_type;
        match common.opcode {
            Opcode::Zext | Opcode::Sext => match (from_ty, ret_ty) {
                (ValTypeID::Int(from_bits), ValTypeID::Int(ret_bits)) => {
                    if from_bits >= ret_bits {
                        Err(InstError::InvalidCast(CastError::ExtCastOperandTooLarge(
                            ret_ty, from_ty,
                        )))
                    } else {
                        Ok(())
                    }
                }
                _ => Err(InstError::OperandTypeMismatch(
                    TypeMismatchError::KindNotMatch(ret_ty, from_ty),
                    from_value,
                )),
            },
            Opcode::Fpext => match (from_ty, ret_ty) {
                (ValTypeID::Float(from_kind), ValTypeID::Float(ret_kind)) => {
                    if from_kind.get_binary_bits() <= ret_kind.get_binary_bits() {
                        Ok(())
                    } else {
                        Err(InstError::InvalidCast(CastError::ExtCastOperandTooLarge(
                            ret_ty, from_ty,
                        )))
                    }
                }
                _ => Err(InstError::OperandTypeMismatch(
                    TypeMismatchError::KindNotMatch(ret_ty, from_ty),
                    from_value,
                )),
            },
            Opcode::Trunc => match (from_ty, ret_ty) {
                (ValTypeID::Int(from_bits), ValTypeID::Int(ret_bits)) => {
                    if from_bits <= ret_bits {
                        Err(InstError::InvalidCast(CastError::TruncCastOperandTooSmall(
                            ret_ty, from_ty,
                        )))
                    } else {
                        Ok(())
                    }
                }
                _ => Err(InstError::OperandTypeMismatch(
                    TypeMismatchError::KindNotMatch(ret_ty, from_ty),
                    from_value,
                )),
            },
            Opcode::Fptrunc => match (from_ty, ret_ty) {
                (ValTypeID::Float(from_kind), ValTypeID::Float(ret_kind)) => {
                    if from_kind.get_binary_bits() >= ret_kind.get_binary_bits() {
                        Ok(())
                    } else {
                        Err(InstError::InvalidCast(CastError::TruncCastOperandTooSmall(
                            ret_ty, from_ty,
                        )))
                    }
                }
                _ => Err(InstError::OperandTypeMismatch(
                    TypeMismatchError::KindNotMatch(ret_ty, from_ty),
                    from_value,
                )),
            },
            Opcode::Bitcast => {
                // Bitcast is allowed between any types whose instance size are equal.
                let type_ctx = &*module.type_ctx;
                if from_ty.get_instance_size(type_ctx) == ret_ty.get_instance_size(type_ctx) {
                    Ok(())
                } else {
                    Err(InstError::InvalidCast(CastError::ExtCastOperandTooLarge(
                        ret_ty, from_ty,
                    )))
                }
            }
            Opcode::PtrToInt => check_type_kind_match(ValTypeID::Ptr, from_ty)
                .map_err(|e| InstError::OperandTypeMismatch(e, from_value)),
            Opcode::IntToPtr => check_type_kind_match(ValTypeID::Ptr, ret_ty)
                .map_err(|e| InstError::OperandTypeMismatch(e, from_value)),
            Opcode::Sitofp | Opcode::Uitofp => check_type_kind_match(ValTypeID::Int(0), from_ty)
                .map_err(|e| InstError::OperandTypeMismatch(e, from_value)),
            Opcode::Fptosi => {
                check_type_kind_match(ValTypeID::Float(FloatTypeKind::Ieee32), from_ty)
                    .map_err(|e| InstError::OperandTypeMismatch(e, from_value))
            }
            _ => panic!("Invalid cast opcode: {:?}", common.opcode),
        }
    }
}

impl CastOp {
    pub fn new_raw(
        mut_module: &Module,
        opcode: Opcode,
        ret_ty: ValTypeID,
    ) -> Result<(InstDataCommon, Self), InstError> {
        if !opcode.is_cast_op() {
            return Err(InstError::InvalidCast(CastError::InvalidOpcode(opcode)));
        }
        let mut common =
            InstDataCommon::new(opcode, ret_ty, &mut mut_module.borrow_use_alloc_mut());
        let mut ret = Self { from_op: UseRef::new_null() };
        ret.build_operands(&mut common, &mut mut_module.borrow_use_alloc_mut());
        Ok((common, ret))
    }

    pub fn new(
        mut_module: &Module,
        opcode: Opcode,
        ret_ty: ValTypeID,
        from_value: ValueSSA,
    ) -> Result<(InstDataCommon, Self), InstError> {
        let (common, ret) = Self::new_raw(mut_module, opcode, ret_ty)?;
        let alloc_use = mut_module.borrow_use_alloc();
        ret.from_op.set_operand_nordfg(&alloc_use, from_value);
        Ok((common, ret))
    }
}
