use std::cell::{Ref, RefCell};

use crate::{
    impl_slabref,
    ir::{ValueSSA, module::Module},
    typing::{id::ValTypeID, types::ArrayTypeRef},
};

pub enum ConstExprData {
    Array(Array),
    Struct(Struct),
}

pub struct Array {
    pub arrty: ArrayTypeRef,
    pub elems: Vec<ValueSSA>,
}

pub struct Struct {
    pub structty: ValTypeID,
    pub elems: Vec<ValueSSA>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstExprRef(usize);
impl_slabref!(ConstExprRef, ConstExprData);

impl ConstExprRef {
    pub fn get_value_type(&self, module: &Module) -> ValTypeID {
        match &*module.get_expr(self.clone()) {
            ConstExprData::Array(data) => ValTypeID::Array(data.arrty),
            ConstExprData::Struct(data) => data.structty.clone(),
        }
    }
}
