use crate::{
    base::{slabref::SlabRef, NullableValue},
    ir::{block::BlockRef, opcode::Opcode, Module, ValueRef},
    typing::id::{ValTypeID, ValTypeUnion},
};

use super::{
    InstCommon, InstDataTrait, InstRef,
    usedef::{UseData, UseRef},
};

pub struct BinSelect {
    pub cond: UseRef,
    pub true_value:  UseRef,
    pub false_value: UseRef,
}

pub struct BinOp {
    pub lhs: UseRef,
    pub rhs: UseRef,
}

pub struct CastOp {
    pub src: UseRef,
}

pub struct IndexPtrOp {
    pub aggr_ty:    ValTypeID,
    pub index_ty:   ValTypeID,
    pub indexed:    UseRef,
    pub indices:    Vec<UseRef>,
}

pub struct CallOp {
    pub func_ty: ValTypeID,
    pub callee: UseRef,
    pub args: Vec<UseRef>,
}

pub struct LoadOp {
    pub source_ty: ValTypeID,
    pub source:    UseRef,
}

pub struct StoreOp {
    pub source_ty: ValTypeID,
    pub source:    UseRef,
    pub target:    UseRef,
}


impl InstDataTrait for BinSelect {
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        let ret = InstCommon::new(opcode, ty, parent, module);
        self.cond = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        self.true_value = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        self.false_value = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        ret
    }
}

impl InstDataTrait for BinOp {
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        let ret = InstCommon::new(opcode, ty, parent, module);
        self.lhs = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        self.rhs = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        ret
    }
}

impl InstDataTrait for CastOp {
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        let ret = InstCommon::new(opcode, ty, parent, module);
        self.src = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        ret
    }
}

impl InstDataTrait for IndexPtrOp {
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        let ret = InstCommon::new(opcode, ty, parent, module);
        self.indexed = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        // `self.indices` cannot be initialized because lacking of actual container type.
        ret
    }
}

impl CallOp {
    pub fn get_callee(&self, module: &Module) -> ValueRef {
        self.callee
            .to_slabref(&module._alloc_use)
            .unwrap()
            .operand
            .get()
            .unwrap_or(ValueRef::None)
    }
    pub fn set_callee(&mut self, callee: ValueRef, module: &mut Module) {
        self.callee
            .to_slabref_mut(&mut module._alloc_use)
            .unwrap()
            .operand.set(callee.to_option());
    }
    pub fn get_args<'a>(&'a self, module: &'a Module) -> impl Iterator<Item = ValueRef> + 'a {
        self.args
            .iter()
            .map(move |u| {
                u.to_slabref(&module._alloc_use).unwrap()
                 .operand.get().unwrap_or(ValueRef::None)
            })
    }
    pub fn args_mut<'a>(&'a self, module: &'a mut Module) -> impl Iterator<Item = ValueRef> + 'a {
        self.args
            .iter()
            .map(move |u| {
                u.to_slabref_mut(&mut module._alloc_use).unwrap()
                 .operand.get().unwrap_or(ValueRef::None)
            })
    }
    pub fn get_arg(&self, i: usize, module: &Module) -> ValueRef {
        ValueRef::from_option(self.get_args(module).nth(i))
    }
    pub fn set_arg(&self, i: usize, arg: ValueRef, module: &mut Module) {
        self.args[i]
            .to_slabref_mut(&mut module._alloc_use)
            .unwrap()
            .operand.set(arg.to_option());
    }
    pub fn get_n_args(&self) -> usize {
        self.args.len()
    }
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
