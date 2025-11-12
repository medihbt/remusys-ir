use crate::ir::{
    IRAllocs, ISubGlobal, ISubInst, ISubValueSSA, ITraceableValue, IUser, InstID, Module, UserID,
    ValueSSA,
    module::allocs::{IPoolAllocated, PoolAllocatedDisposeErr, PoolAllocatedDisposeRes},
};
use mtb_entity_slab::{EntityList, IEntityListNodeID};
use std::panic::Location;

pub(super) fn traceable_init_id<T: ITraceableValue>(t: &T, self_id: ValueSSA, allocs: &IRAllocs) {
    let Some(users) = t.try_get_users() else {
        return;
    };
    users.forall_with_sentinel(&allocs.uses, |_, u| {
        u.operand.set(self_id);
        true
    });
}
pub(super) fn user_init_id<T: IUser>(t: &T, self_id: UserID, allocs: &IRAllocs) {
    traceable_init_id(t, self_id.into_ir(), allocs);
    for u in t.get_operands() {
        u.set_user(allocs, Some(self_id));
    }
}

// Dispose helpers

pub(super) fn dispose_entity_list<T>(
    list: &EntityList<T::PtrID>,
    pool: &T::MinRelatedPoolT,
) -> PoolAllocatedDisposeRes
where
    T: IPoolAllocated<PtrID: IEntityListNodeID>,
{
    let alloc = T::get_alloc(pool.as_ref());
    while let Ok(id) = list.pop_front(alloc) {
        T::dispose_id(id, pool)?;
    }
    T::dispose_id(list.head, pool)?;
    T::dispose_id(list.tail, pool)
}
pub(super) fn traceable_dispose<T: ITraceableValue>(
    t: &T,
    allocs: &IRAllocs,
) -> PoolAllocatedDisposeRes {
    let Some(users) = t.try_get_users() else {
        return Ok(());
    };
    users.clean(&allocs.uses);
    users.sentinel.dispose(allocs)
}
pub(super) fn user_dispose<T: IUser>(t: &T, allocs: &IRAllocs) -> PoolAllocatedDisposeRes {
    for u in t.get_operands() {
        // 重复dispose不会有问题
        let _ = u.dispose(allocs);
    }
    traceable_dispose(t, allocs)
}
pub(super) fn inst_dispose<T: ISubInst>(
    inst: &T,
    id: InstID,
    allocs: &IRAllocs,
) -> PoolAllocatedDisposeRes {
    let common = inst.get_common();
    if common.disposed.get() {
        return Err(PoolAllocatedDisposeErr::AlreadyDisposed);
    }
    common.disposed.set(true);

    if let Some(bb) = inst.get_parent()
        && !common.is_sentinel()
    {
        let insts = &bb.get_body(allocs).insts;
        insts
            .node_unplug(id, &allocs.insts)
            .expect("Failed to unplug instruction from parent basic block");
    }
    user_dispose(inst, allocs)?;
    if let Some(jt_list) = inst.try_get_jts() {
        for &jt_id in jt_list.iter() {
            let _ = jt_id.dispose(allocs);
        }
    }
    Ok(())
}
pub(super) fn global_common_dispose<T: ISubGlobal>(
    global: &T,
    module: &Module,
) -> PoolAllocatedDisposeRes {
    let common = global.get_common();
    if common.dispose_mark.get() {
        return Err(PoolAllocatedDisposeErr::AlreadyDisposed);
    }
    common.dispose_mark.set(true);

    let symbol_registered = {
        let symbols = module.symbols.borrow();
        symbols.contains_key(&common.name)
    };
    if symbol_registered {
        let symbols = module.symbols.try_borrow_mut();
        let mut symbols = match symbols {
            Ok(s) => s,
            Err(_) => {
                let err = PoolAllocatedDisposeErr::SymtabBorrowError(Location::caller());
                return Err(err);
            }
        };
        symbols.remove(&common.name);
    }
    user_dispose(global, &module.allocs)
}
