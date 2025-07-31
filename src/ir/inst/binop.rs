//! Binary operation.

use slab::Slab;

use super::{
    InstDataCommon, InstDataUnique, InstError,
    checking::{check_operand_type_kind_match, check_operand_type_match},
    usedef::{UseData, UseKind, UseRef},
};

use crate::{
    base::INullableValue,
    ir::{ValueSSA, module::Module, opcode::Opcode},
    typing::{TypeMismatchError, id::ValTypeID},
};

pub struct BinOp {
    pub lhs: UseRef,
    pub rhs: UseRef,
}

impl InstDataUnique for BinOp {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self.lhs = common.alloc_use(alloc_use, UseKind::BinOpLhs);
        self.rhs = common.alloc_use(alloc_use, UseKind::BinOpRhs);
    }

    fn check_operands(&self, common: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let alloc_use = module.borrow_use_alloc();
        let lhs = self.lhs.get_operand(&alloc_use);
        let rhs = self.rhs.get_operand(&alloc_use);

        // Check the operands.
        check_operand_type_match(common.ret_type, lhs, module)?;

        // Check the opcode.
        match common.opcode {
            Opcode::Add
            | Opcode::Sub
            | Opcode::Mul
            | Opcode::Fadd
            | Opcode::Fsub
            | Opcode::Fmul
            | Opcode::Fdiv
            | Opcode::Frem => check_operand_type_match(common.ret_type, rhs, module),

            Opcode::Sdiv | Opcode::Udiv | Opcode::Srem | Opcode::Urem => {
                check_operand_type_match(common.ret_type, rhs, module)?;
                match rhs {
                    ValueSSA::ConstData(x) if x.is_zero() => Err(InstError::DividedByZero),
                    _ => Ok(()),
                }
            }

            Opcode::BitAnd
            | Opcode::BitOr
            | Opcode::BitXor
            | Opcode::Shl
            | Opcode::Lshr
            | Opcode::Ashr => check_operand_type_kind_match(ValTypeID::Int(0), rhs, module),

            _ => {
                if common.opcode.is_binary_op() {
                    panic!("Binary operation {:?} is not implemented", common.opcode);
                } else {
                    panic!("Opcode {:?} is not a binary operation", common.opcode);
                }
            }
        }
    }
}

impl BinOp {
    pub fn new_raw(
        opcode: Opcode,
        ret_ty: ValTypeID,
        mut_module: &Module,
    ) -> Result<(InstDataCommon, Self), InstError> {
        match ret_ty {
            ValTypeID::Int(_) | ValTypeID::Float(_) => {}
            _ => {
                return Err(InstError::OperandTypeMismatch(
                    TypeMismatchError::NotPrimitive(ret_ty),
                    ValueSSA::None,
                ));
            }
        }

        let mut common =
            InstDataCommon::new(opcode, ret_ty, &mut mut_module.borrow_use_alloc_mut());
        let mut ret = Self { lhs: UseRef::new_null(), rhs: UseRef::new_null() };
        ret.build_operands(&mut common, &mut mut_module.borrow_use_alloc_mut());
        Ok((common, ret))
    }

    pub fn new_with_operands(
        mut_module: &Module,
        opcode: Opcode,
        lhs: ValueSSA,
        rhs: ValueSSA,
    ) -> Result<(InstDataCommon, Self), InstError> {
        let (common, ret) = Self::new_raw(opcode, lhs.get_value_type(mut_module), mut_module)?;
        ret.lhs
            .set_operand_nordfg(&mut_module.borrow_use_alloc(), lhs);
        ret.rhs
            .set_operand_nordfg(&mut_module.borrow_use_alloc(), rhs);
        Ok((common, ret))
    }
}
