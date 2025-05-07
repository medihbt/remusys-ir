use core::panic;
use slab::Slab;

use crate::{
    base::{NullableValue, slabref::SlabRef},
    ir::{
        Module, ValueRef,
        block::BlockRef,
        global::{Global, GlobalRef, func::Func},
        opcode::Opcode,
    },
    typing::id::{ValTypeID, ValTypeUnion},
};

use super::{
    InstCommon, InstDataTrait, InstRef,
    usedef::{UseData, UseRef},
};

pub struct CallOp {
    pub func_ty: ValTypeID,
    pub callee:  UseRef,
    pub args:    Vec<UseRef>,
}

impl InstDataTrait for CallOp {
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        let ret = InstCommon::new(opcode, ty, parent, module);
        self.callee = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        // initialize `self.args` depending on the actual callee function type.
        let nargs = {
            let type_ctx = module.get_type_ctx().borrow();
            let func_ty = type_ctx.find_type(&self.func_ty).unwrap();

            match func_ty {
                ValTypeUnion::Func(func_ty) => func_ty.args.len(),
                _ => panic!("Invalid function type"),
            }
        };
        self.args.reserve(nargs);
        for _ in 0..nargs {
            self.args
                .push(ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use));
        }
        ret
    }
}

impl CallOp {
    pub fn get_func_callee_ref(&self, use_alloc: &Slab<UseData>) -> Option<GlobalRef> {
        let use_data = self
            .callee
            .to_slabref(use_alloc)
            .expect("Invalid reference: UAF?");
        use_data.operand.get().map(|v| match v {
            ValueRef::Global(g) => g,
            _ => panic!("Type error: requires Function, but got {:#?}", v),
        })
    }

    pub fn get_func_callee_data<'a>(&self, module: &'a Module) -> Option<&'a Func> {
        self.get_func_callee_ref(&module._alloc_use).map(|gref| {
            let globl = gref
                .to_slabref(&module._alloc_global)
                .expect("Invalid reference: UAF?");
            match globl {
                Global::Func(f) => f,
                Global::Alias(_) => panic!("Type error: requires Function, but got Global::Alias"),
                Global::Var(_) => panic!("Type error: requires Function, but got Global::Var"),
            }
        })
    }

    pub fn get_callee_value(&self, module: &Module) -> ValueRef {
        self.callee
            .to_slabref(&module._alloc_use)
            .unwrap()
            .operand
            .get()
            .unwrap_or(ValueRef::None)
    }
    pub fn set_callee_value(&mut self, callee: ValueRef, module: &mut Module) {
        self.callee
            .to_slabref_mut(&mut module._alloc_use)
            .unwrap()
            .operand
            .set(callee.to_option());
    }
    pub fn get_n_args(&self) -> usize {
        self.args.len()
    }

    pub fn get_arg(&self, use_alloc: &Slab<UseData>, index: usize) -> Option<ValueRef> {
        self.args[index].get_oprerand_ref(use_alloc)
    }
    #[allow(unused)]
    pub fn set_arg(&self, use_alloc: &Slab<UseData>, index: usize, value: Option<ValueRef>) {
        todo!()
    }
}
