use slab::Slab;
use std::ops::Deref;

use crate::{
    base::{INullableValue, SlabRef},
    ir::{IRAllocs, IRAllocsEditable, IRAllocsReadable, ISubInst, ISubValueSSA, InstData, InstRef},
};

pub enum IRAllocsZipRef<'a> {
    Fix(&'a IRAllocs),
    Mut(&'a mut IRAllocs),
}
impl<'a, T> From<&'a T> for IRAllocsZipRef<'a>
where
    T: IRAllocsReadable,
{
    fn from(value: &'a T) -> Self {
        IRAllocsZipRef::Fix(value.get_allocs_ref())
    }
}
impl<'a, T> From<&'a mut T> for IRAllocsZipRef<'a>
where
    T: IRAllocsEditable,
{
    fn from(value: &'a mut T) -> Self {
        IRAllocsZipRef::Mut(value.get_allocs_mutref())
    }
}
impl<'a> Deref for IRAllocsZipRef<'a> {
    type Target = IRAllocs;

    fn deref(&self) -> &IRAllocs {
        self.get()
    }
}
impl<'a> IRAllocsZipRef<'a> {
    pub fn get(&self) -> &IRAllocs {
        match self {
            IRAllocsZipRef::Fix(allocs) => allocs,
            IRAllocsZipRef::Mut(allocs) => &**allocs,
        }
    }
    pub fn as_mut(&mut self) -> Option<&mut IRAllocs> {
        match self {
            IRAllocsZipRef::Fix(_) => None,
            IRAllocsZipRef::Mut(allocs) => Some(&mut **allocs),
        }
    }
}

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
    allocs: IRAllocsZipRef<'a>,
}

pub type ManagedInst<'a> = IRManaged<'a, InstRef>;

impl<'a, T: IManageableIRValue> Drop for IRManaged<'a, T> {
    fn drop(&mut self) {
        if self.val.is_null() {
            return;
        }
        self.val.defer_cleanup_self(&self.allocs);
        if let Some(allocs) = self.allocs.as_mut() {
            self.val.free_from_alloc(T::select_alloc_mut(allocs));
        }
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
    pub fn new<AllocRef>(val: T, allocs: AllocRef) -> Self
    where
        IRAllocsZipRef<'a>: From<AllocRef>,
    {
        Self { val, allocs: allocs.into() }
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

    pub fn get_allocs(&self) -> &IRAllocs {
        &self.allocs
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
