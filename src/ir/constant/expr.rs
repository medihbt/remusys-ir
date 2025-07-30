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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn binary_is_zero(&self, module: &Module) -> bool {
        module.get_expr(self.clone()).binary_is_zero(module)
    }
    pub fn binary_is_zero_from_alloc(&self, alloc_expr: &Slab<ConstExprData>) -> bool {
        self.to_data(alloc_expr)
            .binary_is_zero_from_alloc(alloc_expr)
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

    fn expr_visitor_dispatch(&self, expr_ref: ConstExprRef, alloc_expr: &Slab<ConstExprData>) {
        let expr_data = expr_ref.to_data(alloc_expr);
        match expr_data {
            ConstExprData::Array(array) => self.read_array(expr_ref, array),
            ConstExprData::Struct(struct_data) => self.read_struct(expr_ref, struct_data),
        }
    }
}

impl ConstExprData {
    pub fn binary_is_zero(&self, module: &Module) -> bool {
        match self {
            ConstExprData::Array(a) => Self::binary_is_zero_aggregate(&a.elems, module),
            ConstExprData::Struct(s) => Self::binary_is_zero_aggregate(&s.elems, module),
        }
    }
    pub fn binary_is_zero_from_alloc(&self, alloc_expr: &Slab<ConstExprData>) -> bool {
        match self {
            ConstExprData::Array(a) => a
                .elems
                .iter()
                .all(|v| v.binary_is_zero_from_alloc(alloc_expr)),
            ConstExprData::Struct(s) => s
                .elems
                .iter()
                .all(|v| v.binary_is_zero_from_alloc(alloc_expr)),
        }
    }

    fn binary_is_zero_aggregate(values: &[ValueSSA], module: &Module) -> bool {
        if values.is_empty() { true } else { values.iter().all(|v| v.binary_is_zero(module)) }
    }
}
