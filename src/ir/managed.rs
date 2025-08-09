use slab::Slab;
use std::ops::Deref;

use crate::{
    base::SlabRef,
    ir::{IRAllocs, IRAllocsRef, ISubInst, ISubValueSSA, InstData, InstRef, Module},
};

pub trait IManagedIRValue: ISubValueSSA + SlabRef {
    fn defer_cleanup_self(&self, allocs: &IRAllocs);

    /// 获取当前值的分配器
    fn select_alloc(allocs: &IRAllocs) -> &Slab<Self::RefObject>;
}

pub struct IRManaged<'a, T: IManagedIRValue> {
    val: T,
    allocs: IRAllocsRef<'a>,
}

pub type ManagedInst<'a> = IRManaged<'a, InstRef>;

impl<'a, T: IManagedIRValue> Drop for IRManaged<'a, T> {
    fn drop(&mut self) {
        if self.val.is_null() {
            return;
        }
        self.val.defer_cleanup_self(&self.allocs);
    }
}

impl<'a, T: IManagedIRValue> Deref for IRManaged<'a, T> {
    type Target = T::RefObject;

    fn deref(&self) -> &T::RefObject {
        let alloc = T::select_alloc(&self.allocs);
        self.val.to_data(alloc)
    }
}

impl<'a, T: IManagedIRValue> IRManaged<'a, T> {
    pub fn new(val: T, allocs: IRAllocsRef<'a>) -> Self {
        Self { val, allocs }
    }
    pub fn from_module(val: T, module: &'a Module) -> Self {
        Self { val, allocs: IRAllocsRef::Dyn(module.borrow_allocs()) }
    }
    pub fn from_modmut(val: T, module: &'a mut Module) -> Self {
        Self { val, allocs: IRAllocsRef::Mut(module.allocs_mut()) }
    }
    pub fn from_allocs(val: T, allocs: &'a IRAllocs) -> Self {
        Self { val, allocs: IRAllocsRef::Fix(allocs) }
    }

    pub fn release(mut self) -> T {
        std::mem::replace(&mut self.val, T::new_null())
    }

    pub fn is_null(&self) -> bool {
        self.val.is_null()
    }
    pub fn is_nonnull(&self) -> bool {
        !self.is_null()
    }
}

impl IManagedIRValue for InstRef {
    fn defer_cleanup_self(&self, allocs: &IRAllocs) {
        self.to_data(&allocs.insts).cleanup();
    }
    fn select_alloc(allocs: &IRAllocs) -> &Slab<InstData> {
        &allocs.insts
    }
}
