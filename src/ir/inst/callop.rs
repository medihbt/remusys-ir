//! Call operation

use std::num::NonZero;

use slab::Slab;

use crate::{
    base::NullableValue,
    ir::{
        PtrUser, ValueSSA, ValueSSAError,
        global::{
            GlobalData, GlobalRef,
            func::{FuncStorage, FuncUser},
        },
        module::Module,
        opcode::Opcode,
    },
    typing::{context::TypeContext, id::ValTypeID, types::FuncTypeRef},
};

use super::{
    InstDataCommon, InstError,
    checking::check_operand_type_kind_match,
    usedef::{UseData, UseRef},
};

use super::InstDataUnique;

pub struct CallOp {
    pub callee: UseRef,
    pub callee_ty: FuncTypeRef,
    pub fixed_nargs: usize,
    pub is_vararg: bool,
    pub args: Box<[UseRef]>,
}

impl PtrUser for CallOp {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        ValTypeID::Func(self.callee_ty)
    }
    fn get_operand_align(&self) -> Option<NonZero<usize>> {
        None
    }
}
impl FuncUser for CallOp {}

impl InstDataUnique for CallOp {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self.callee = common.alloc_use(alloc_use);
        for arg in self.args.iter_mut() {
            *arg = common.alloc_use(alloc_use);
        }
    }

    fn check_operands(&self, _: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        // Check the callee. NOTE: call expression always calls a pointer to function.
        let callee = self.callee.get_operand(&module.borrow_use_alloc());
        check_operand_type_kind_match(ValTypeID::Ptr, callee, module)?;

        // Check the arguments.
        let type_ctx = module.type_ctx.as_ref();
        let nargs = self.callee_ty.get_nargs(&type_ctx);

        Self::_check_operand_count(type_ctx, self.callee_ty, self.args.len())?;

        let alloc_use = module.borrow_use_alloc();
        for (i, arg) in self.args.iter().enumerate() {
            if i >= nargs {
                break; // Cannot verify the vararg arguments.
            }
            let arg = arg.get_operand(&alloc_use);
            let arg_ty = self.callee_ty.get_arg(type_ctx, i).unwrap();
            check_operand_type_kind_match(arg_ty, arg, module)?;
        }

        Ok(())
    }
}

impl CallOp {
    pub fn new_raw_fixed(
        mut_module: &Module,
        callee_func_ty: FuncTypeRef,
    ) -> Result<(InstDataCommon, Self), InstError> {
        Self::new_raw(
            mut_module,
            callee_func_ty,
            callee_func_ty.get_nargs(mut_module.type_ctx.as_ref()),
        )
    }

    pub fn new_raw(
        mut_module: &Module,
        callee_func_ty: FuncTypeRef,
        nargs: usize,
    ) -> Result<(InstDataCommon, Self), InstError> {
        let type_ctx = mut_module.type_ctx.as_ref();

        Self::_check_operand_count(type_ctx, callee_func_ty, nargs)?;

        let mut alloc_use = mut_module.borrow_use_alloc_mut();
        let mut common = InstDataCommon::new(
            Opcode::Call,
            callee_func_ty.get_return_type(type_ctx),
            &mut alloc_use,
        );

        let mut ret = Self {
            callee: UseRef::new_null(),
            callee_ty: callee_func_ty,
            fixed_nargs: nargs,
            is_vararg: callee_func_ty.is_vararg(type_ctx),
            args: vec![UseRef::new_null(); nargs].into_boxed_slice(),
        };
        ret.build_operands(&mut common, &mut alloc_use);
        Ok((common, ret))
    }

    pub fn new(
        mut_module: &Module,
        callee_func_ty: FuncTypeRef,
        callee: ValueSSA,
        args: impl Iterator<Item = ValueSSA> + Clone,
    ) -> Result<(InstDataCommon, Self), InstError> {
        assert!(callee.is_null() || callee.get_value_type(&mut_module) == ValTypeID::Ptr);
        let nargs = args.clone().count();
        let (common, ret) = Self::new_raw(mut_module, callee_func_ty, nargs)?;
        let alloc_use = mut_module.borrow_use_alloc();
        ret.callee.set_operand_nordfg(&alloc_use, callee);
        for (useref, value) in ret.args.iter().zip(args) {
            useref.set_operand_nordfg(&alloc_use, value);
        }
        Ok((common, ret))
    }

    pub fn new_from_func(
        mut_module: &Module,
        callee_func: GlobalRef,
        args: impl Iterator<Item = ValueSSA> + Clone,
    ) -> Result<(InstDataCommon, Self), InstError> {
        let functy = match &*mut_module.get_global(callee_func) {
            GlobalData::Func(f) => f.get_stored_func_type(),
            _ => {
                return Err(InstError::OperandError(ValueSSAError::NotFunction(
                    ValueSSA::Global(callee_func),
                )));
            }
        };
        Self::new(mut_module, functy, ValueSSA::Global(callee_func), args)
    }

    fn _check_operand_count(
        type_ctx: &TypeContext,
        callee_func_ty: FuncTypeRef,
        call_nargs: usize,
    ) -> Result<(), InstError> {
        let is_vararg = callee_func_ty.is_vararg(type_ctx);
        let fixed_nargs = callee_func_ty.get_nargs(type_ctx);

        if is_vararg {
            if call_nargs < fixed_nargs {
                Err(InstError::InvalidArgumentCount(fixed_nargs, call_nargs))
            } else {
                Ok(())
            }
        } else if call_nargs != fixed_nargs {
            Err(InstError::InvalidArgumentCount(fixed_nargs, call_nargs))
        } else {
            Ok(())
        }
    }
}
