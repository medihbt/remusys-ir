//! Managed IR structures with auto-disposal capabilities.
use crate::ir::*;
use mtb_entity::IEntityAllocID;

pub struct IRManaged<'ir, T: IPoolAllocated> {
    allocs: &'ir IRAllocs,
    id: T::ModuleID,
}
impl<'ir, T: IPoolAllocated> Drop for IRManaged<'ir, T> {
    fn drop(&mut self) {
        T::dispose_id(self.id, self.allocs);
    }
}
impl<'ir, T: IPoolAllocated> IRManaged<'ir, T> {
    pub fn new(allocs: &'ir IRAllocs, id: T::ModuleID) -> Self {
        Self { allocs, id }
    }

    pub fn as_ref(&self) -> &'ir T {
        let ptr = T::from_module_id(self.id);
        let alloc = T::get_alloc(self.allocs);
        ptr.deref(alloc)
    }

    pub fn release(self) -> T::ModuleID {
        let id = self.id;
        std::mem::forget(self);
        id
    }
}

pub type ManagedExpr<'ir> = IRManaged<'ir, ExprObj>;
pub type ManagedGlobal<'ir> = IRManaged<'ir, GlobalObj>;
pub type ManagedBlock<'ir> = IRManaged<'ir, BlockObj>;
pub type ManagedInst<'ir> = IRManaged<'ir, InstObj>;
pub type ManagedJT<'ir> = IRManaged<'ir, JumpTarget>;
pub type ManagedUse<'ir> = IRManaged<'ir, Use>;
