//! Call operation

use std::num::NonZero;

use crate::{
    ir::{PtrUser, global::func::FuncUser},
    typing::{id::ValTypeID, types::FuncTypeRef},
};

use super::usedef::UseRef;

pub struct CallOp {
    pub callee: UseRef,
    pub callee_ty: FuncTypeRef,
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


pub struct IntrinOp(CallOp);
