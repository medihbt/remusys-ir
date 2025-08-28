use slab::Slab;
use std::ops::Deref;

use crate::{
    base::{INullableValue, SlabRef},
    ir::{IRAllocs, ISubInst, ISubValueSSA, InstData, InstRef},
};

pub trait IManageableIRValue: ISubValueSSA + SlabRef {
    fn defer_cleanup_self(&self, allocs: &IRAllocs);

    /// 获取当前值的分配器
    fn select_alloc(allocs: &IRAllocs) -> &Slab<Self::RefObject>;

    /// 获取当前值的分配器 (可变版本)
    fn select_alloc_mut(allocs: &mut IRAllocs) -> &mut Slab<Self::RefObject>;

    /// 完成所有析构并删除自身
    fn delete_self(&self, allocs: &mut IRAllocs) {
        self.defer_cleanup_self(allocs);
        self.free_from_alloc(Self::select_alloc_mut(allocs));
    }

    /// 不析构, 直接移除自身
    fn free_from_allocs(&self, allocs: &mut IRAllocs) {
        self.free_from_alloc(Self::select_alloc_mut(allocs));
    }
}

pub struct IRManaged<'a, T: IManageableIRValue> {
    val: T,
    allocs: &'a IRAllocs,
}

pub type ManagedInst<'a> = IRManaged<'a, InstRef>;

impl<'a, T: IManageableIRValue> Drop for IRManaged<'a, T> {
    fn drop(&mut self) {
        if self.val.is_null() {
            return;
        }
        self.val.defer_cleanup_self(self.allocs);
    }
}

impl<'a, T: IManageableIRValue> Deref for IRManaged<'a, T> {
    type Target = T::RefObject;

    fn deref(&self) -> &T::RefObject {
        let alloc = T::select_alloc(&self.allocs);
        self.val.to_data(alloc)
    }
}

impl<'a, T: IManageableIRValue> IRManaged<'a, T> {
    pub fn new(val: T, allocs: &'a IRAllocs) -> Self {
        Self { val, allocs }
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

impl IManageableIRValue for InstRef {
    fn defer_cleanup_self(&self, allocs: &IRAllocs) {
        self.to_data(&allocs.insts).cleanup();
        // 把自己从基本块中删除
        let parent = self.get_parent_from_alloc(&allocs.insts);
        if parent.is_nonnull() {
            parent
                .insts_from_alloc(&allocs.blocks)
                .unplug_node(&allocs.insts, *self)
                .expect("Failed to unplug instruction from block");
        }
    }
    fn select_alloc(allocs: &IRAllocs) -> &Slab<InstData> {
        &allocs.insts
    }
    fn select_alloc_mut(allocs: &mut IRAllocs) -> &mut Slab<InstData> {
        &mut allocs.insts
    }
}
