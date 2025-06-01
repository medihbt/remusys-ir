//! Compare instructions.

use slab::Slab;

use crate::{
    base::NullableValue,
    ir::{ValueSSA, cmp_cond::CmpCond, module::Module, opcode::Opcode},
    typing::{TypeMismatchError, id::ValTypeID},
};

use super::{
    InstDataCommon, InstDataUnique, InstError,
    checking::check_operand_type_match,
    usedef::{UseData, UseKind, UseRef},
};

pub struct CmpOp {
    pub lhs: UseRef,
    pub rhs: UseRef,
    pub cmp_ty: ValTypeID,
    pub cond: CmpCond,
}

impl InstDataUnique for CmpOp {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self.lhs = common.alloc_use(alloc_use, UseKind::CmpLhs);
        self.rhs = common.alloc_use(alloc_use, UseKind::CmpRhs);
    }

    fn check_operands(&self, _: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let alloc_use = module.borrow_use_alloc();
        let lhs = self.lhs.get_operand(&alloc_use);
        let rhs = self.rhs.get_operand(&alloc_use);

        check_operand_type_match(self.cmp_ty, lhs, module)?;
        check_operand_type_match(self.cmp_ty, rhs, module)
    }
}

impl CmpOp {
    pub fn new_raw(
        cmp_ty: ValTypeID,
        cond: CmpCond,
        mut_module: &Module,
    ) -> Result<(InstDataCommon, Self), InstError> {
        let (cond, opcode) = match cmp_ty {
            ValTypeID::Int(_) => (cond.switch_to_int(), Opcode::Icmp),
            ValTypeID::Float(_) => (cond.switch_to_float(), Opcode::Fcmp),
            _ => {
                return Err(InstError::OperandTypeMismatch(
                    TypeMismatchError::NotPrimitive(cmp_ty),
                    ValueSSA::None,
                ));
            }
        };
        let mut common = InstDataCommon::new(
            opcode,
            ValTypeID::new_boolean(),
            &mut mut_module.borrow_use_alloc_mut(),
        );
        let mut ret = Self {
            lhs: UseRef::new_null(),
            rhs: UseRef::new_null(),
            cmp_ty,
            cond,
        };
        ret.build_operands(&mut common, &mut mut_module.borrow_use_alloc_mut());
        Ok((common, ret))
    }

    pub fn new_with_operands(
        mut_module: &Module,
        cond: CmpCond,
        lhs: ValueSSA,
        rhs: ValueSSA,
    ) -> Result<(InstDataCommon, Self), InstError> {
        let cmp_ty = lhs.get_value_type(&mut_module);
        check_operand_type_match(cmp_ty, rhs, mut_module)?;
        Self::new_raw(cmp_ty, cond, &mut_module).map(|(common, cmp)| {
            let alloc = mut_module.borrow_use_alloc();
            cmp.lhs.set_operand_nordfg(&alloc, lhs);
            cmp.rhs.set_operand_nordfg(&alloc, rhs);
            (common, cmp)
        })
    }
}
