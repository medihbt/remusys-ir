use slab::Slab;

use crate::ir::{
    block::jump_target::JumpTargetData,
    inst::usedef::UseData,
    module::{Module, ModuleAllocatorInner, rcfg},
};

use super::redirect::Redirector;

pub(super) struct CompactAlloc<'a> {
    pub(super) redirector: &'a Redirector<'a>,
}

impl<'a> CompactAlloc<'a> {
    pub(super) fn from_redirector(redirector: &'a Redirector<'a>) -> Self {
        Self { redirector, }
    }

    fn get_module(&self) -> &Module {
        self.redirector.module
    }

    fn compact_generate_allocs(&mut self) {
        let module = self.get_module();
        let live_set = &self.redirector.live_set;
        let mut old_alloc_value = module.borrow_value_alloc_mut();

        let mut live_exprs = Vec::with_capacity(module.borrow_value_alloc().alloc_expr.len());
        for (i, expr) in module.borrow_value_alloc().alloc_expr.iter() {
            
        }
    }
}
