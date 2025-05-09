//! Instructions that do not fit into the other categories.

use slab::Slab;

use crate::{ir::module::Module, typing::id::ValTypeID};

use super::{
    InstDataCommon, InstDataUnique, InstError,
    checking::check_operand_type_match,
    usedef::{UseData, UseRef},
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
        self.cond = common.alloc_use(alloc_use);
        self.true_val = common.alloc_use(alloc_use);
        self.false_val = common.alloc_use(alloc_use);
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
