//! Instructions that do not fit into the other categories.

use super::{InstDataUnique, usedef::UseRef};

pub struct SelectOp {
    pub cond: UseRef,
    pub true_val: UseRef,
    pub false_val: UseRef,
}

impl InstDataUnique for SelectOp {
    fn update_build_common(
        &mut self,
        common: super::InstDataCommon,
        mut_module: &crate::ir::module::Module,
    ) -> super::InstDataCommon {
        // Update the common data with the new operands
        common
            .operands
            .push_back_ref(&*mut_module.borrow_use_alloc(), self.cond)
            .unwrap();
        common
            .operands
            .push_back_ref(&*mut_module.borrow_use_alloc(), self.true_val)
            .unwrap();
        common
            .operands
            .push_back_ref(&*mut_module.borrow_use_alloc(), self.false_val)
            .unwrap();

        // Return the updated common data
        common
    }
}
