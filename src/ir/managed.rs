//! Managed IR structures with auto-disposal capabilities.
use crate::ir::{module::allocs::IPoolAllocated, *};
use mtb_entity::IEntityAllocID;

struct IRManagedImpl<'ir, T: IPoolAllocated> {
    pool: &'ir T::MinRelatedPoolT,
    id: T::ModuleID,
}
impl<'ir, T: IPoolAllocated> Drop for IRManagedImpl<'ir, T> {
    fn drop(&mut self) {
        T::dispose_id(self.id, self.pool).expect("Failed to dispose managed IR entity");
    }
}
impl<'ir, T: IPoolAllocated> IRManagedImpl<'ir, T> {
    pub fn new(pool: &'ir T::MinRelatedPoolT, id: T::ModuleID) -> Self {
        Self { pool, id }
    }

    pub fn as_ref(&self) -> &'ir T {
        let ptr = T::from_module_id(self.id);
        let alloc = T::get_alloc(self.pool.as_ref());
        ptr.deref(alloc)
    }

    pub fn release(self) -> T::ModuleID {
        let id = self.id;
        std::mem::forget(self);
        id
    }
}

macro_rules! define_managed {
    ($ManagedName:ident, $Typaname:ty, $PoolT:ty, $ModuleID:ty) => {
        pub struct $ManagedName<'ir> {
            inner: IRManagedImpl<'ir, $Typaname>,
        }
        impl<'ir> $ManagedName<'ir> {
            pub fn new(pool: &'ir $PoolT, id: $ModuleID) -> Self {
                Self { inner: IRManagedImpl::new(pool, id) }
            }
            pub fn as_ref(&self) -> &'ir $Typaname {
                self.inner.as_ref()
            }
            pub fn release(self) -> $ModuleID {
                self.inner.release()
            }
        }
    };
}

define_managed!(ManagedExpr, ExprObj, IRAllocs, ExprID);
define_managed!(ManagedGlobal, GlobalObj, Module, GlobalID);
define_managed!(ManagedBlock, BlockObj, IRAllocs, BlockID);
define_managed!(ManagedInst, InstObj, IRAllocs, InstID);
define_managed!(ManagedJT, JumpTarget, IRAllocs, JumpTargetID);
define_managed!(ManagedUse, Use, IRAllocs, UseID);
