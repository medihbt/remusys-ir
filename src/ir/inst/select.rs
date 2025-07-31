//! Instructions that do not fit into the other categories.

use slab::Slab;

use crate::{
    base::INullableValue,
    ir::{ValueSSA, module::Module, opcode::Opcode},
    typing::id::ValTypeID,
};

use super::{
    InstDataCommon, InstDataUnique, InstError,
    checking::check_operand_type_match,
    usedef::{UseData, UseKind, UseRef},
};

pub struct SelectOp {
    pub cond: UseRef,
    pub true_val: UseRef,
    pub false_val: UseRef,
}

impl InstDataUnique for SelectOp {
    fn build_operands(
        &mut self,
        common: &mut super::InstDataCommon,
        alloc_use: &mut Slab<UseData>,
    ) {
        self.cond = common.alloc_use(alloc_use, UseKind::SelectCond);
        self.true_val = common.alloc_use(alloc_use, UseKind::SelectTrueVal);
        self.false_val = common.alloc_use(alloc_use, UseKind::SelectFalseVal);
    }

    fn check_operands(&self, common: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let self_type = common.ret_type;
        let alloc_use = module.borrow_use_alloc();
        let cond_value = self.cond.get_operand(&alloc_use);
        let true_value = self.true_val.get_operand(&alloc_use);
        let false_value = self.false_val.get_operand(&alloc_use);

        check_operand_type_match(ValTypeID::new_boolean(), cond_value, module)?;
        check_operand_type_match(self_type, true_value, module)?;
        check_operand_type_match(self_type, false_value, module)
    }
}

impl SelectOp {
    pub fn new_raw(mut_module: &Module, valtype: ValTypeID) -> (InstDataCommon, Self) {
        let mut alloc_use = mut_module.borrow_use_alloc_mut();
        let mut common = InstDataCommon::new(Opcode::Select, valtype, &mut alloc_use);
        let mut inst = SelectOp {
            cond: UseRef::new_null(),
            true_val: UseRef::new_null(),
            false_val: UseRef::new_null(),
        };
        inst.build_operands(&mut common, &mut alloc_use);
        (common, inst)
    }

    pub fn new(
        mut_module: &Module,
        cond: ValueSSA,
        true_val: ValueSSA,
        false_val: ValueSSA,
    ) -> Result<(InstDataCommon, Self), InstError> {
        match (cond, true_val, false_val) {
            (ValueSSA::None, ..) | (_, ValueSSA::None, _) | (_, _, ValueSSA::None) => {
                return Err(InstError::OperandNull);
            }
            _ => {}
        }
        check_operand_type_match(ValTypeID::new_boolean(), cond, mut_module)?;
        let value_type = true_val.get_value_type(mut_module);
        check_operand_type_match(value_type, false_val, mut_module)?;

        let (c, s) = Self::new_raw(mut_module, value_type);

        let mut alloc_use = mut_module.borrow_use_alloc_mut();
        s.cond.set_operand_nordfg(&mut alloc_use, cond);
        s.true_val.set_operand_nordfg(&mut alloc_use, true_val);
        s.false_val.set_operand_nordfg(&mut alloc_use, false_val);
        Ok((c, s))
    }
}
