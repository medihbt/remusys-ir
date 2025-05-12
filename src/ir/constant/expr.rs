use std::cell::Ref;

use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    impl_slabref,
    ir::{ValueSSA, module::Module},
    typing::{id::ValTypeID, types::ArrayTypeRef},
};

#[derive(Debug, Clone)]
pub enum ConstExprData {
    Array(Array),
    Struct(Struct),
}

#[derive(Debug, Clone)]
pub struct Array {
    pub arrty: ArrayTypeRef,
    pub elems: Vec<ValueSSA>,
}

#[derive(Debug, Clone)]
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

    pub fn is_aggregate(&self) -> bool {
        true
    }
    pub fn as_aggregate<'a>(&self, module: &'a Module) -> Option<ConstAggregateView<'a>> {
        match &*module.get_expr(self.clone()) {
            ConstExprData::Array(..) | ConstExprData::Struct(..) => {} // _ => return None
        }
        Some(ConstAggregateView(self.clone(), module))
    }

    pub fn read_accept(&self, expr_alloc: &Slab<ConstExprData>, visitor: impl IConstExprVisitor) {
        match self.to_slabref_unwrap(expr_alloc) {
            ConstExprData::Array(a) => visitor.read_array(*self, a),
            ConstExprData::Struct(s) => visitor.read_struct(*self, s),
        }
    }
}

#[derive(Clone)]
pub struct ConstAggregateView<'a>(pub ConstExprRef, &'a Module);

impl<'a> ConstAggregateView<'a> {
    fn borrow_elems(&self) -> Ref<Vec<ValueSSA>> {
        let ConstAggregateView(handle, module) = self.clone();
        Ref::map(module.get_expr(handle), |x| match x {
            ConstExprData::Array(arr) => &arr.elems,
            ConstExprData::Struct(s) => &s.elems,
        })
    }

    pub fn load_elems(&self) -> Vec<ValueSSA> {
        self.borrow_elems().clone()
    }

    pub fn get_elem(&self, index: usize) -> Option<ValueSSA> {
        self.borrow_elems().get(index).map(|x| *x)
    }
    pub fn get_nelems(&self) -> usize {
        self.borrow_elems().len()
    }

    pub fn insert_elem_to_data(&self, index: usize, value: ValueSSA) -> Option<ConstExprData> {
        let ConstAggregateView(handle, module) = self.clone();
        match &*module.get_expr(handle) {
            ConstExprData::Array(a) => {
                if a.elems.len() <= index {
                    None
                } else {
                    let mut ret = a.clone();
                    ret.elems[index] = value;
                    Some(ConstExprData::Array(ret))
                }
            }
            ConstExprData::Struct(s) => {
                if s.elems.len() <= index {
                    None
                } else {
                    let mut ret = s.clone();
                    ret.elems[index] = value;
                    Some(ConstExprData::Struct(ret))
                }
            }
        }
    }
    pub fn insert_elem_to_ref(&self, index: usize, value: ValueSSA) -> Option<ConstExprRef> {
        self.insert_elem_to_data(index, value)
            .map(|data| self.1.insert_expr(data))
    }
}

pub trait IConstExprVisitor {
    fn read_array(&self, array_ref: ConstExprRef, array_data: &Array);
    fn read_struct(&self, struct_ref: ConstExprRef, struct_data: &Struct);
}
