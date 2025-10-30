use crate::ir::{GlobalID, IRAllocs, ITraceableValue, InstObj, UserList};
use mtb_entity::{
    EntityAlloc, EntityList, EntityListError, EntityListHead, IEntityAllocID, IEntityListNode,
    PtrID, PtrListRes,
};
use std::cell::Cell;

pub struct BlockObj {
    head: Cell<EntityListHead<BlockObj>>,
    parent_func: Cell<Option<GlobalID>>,
    insts: Option<EntityList<InstObj>>,
    users: Option<UserList>,
}

impl IEntityListNode for BlockObj {
    fn load_head(&self) -> EntityListHead<Self> {
        self.head.get()
    }
    fn store_head(&self, head: EntityListHead<Self>) {
        self.head.set(head);
    }

    fn is_sentinal(&self) -> bool {
        self.insts.is_none()
    }

    fn new_sentinal() -> Self {
        Self {
            head: Cell::new(EntityListHead::none()),
            parent_func: Cell::new(None),
            insts: None,
            users: None,
        }
    }

    fn on_push_next(
        curr: PtrID<Self>,
        next: PtrID<Self>,
        alloc: &EntityAlloc<Self>,
    ) -> PtrListRes<Self> {
        if curr == next {
            return Err(EntityListError::RepeatedNode);
        }
        let parent = curr.deref(alloc).parent_func.get();
        assert_ne!(parent, None, "Pushing block without parent function");
        next.deref(alloc).parent_func.set(parent);
        Ok(())
    }

    fn on_push_prev(
        curr: PtrID<Self>,
        prev: PtrID<Self>,
        alloc: &EntityAlloc<Self>,
    ) -> PtrListRes<Self> {
        if curr == prev {
            return Err(EntityListError::RepeatedNode);
        }
        let parent = curr.deref(alloc).parent_func.get();
        assert_ne!(parent, None, "Pushing block without parent function");
        prev.deref(alloc).parent_func.set(parent);
        Ok(())
    }

    fn on_unplug(curr: PtrID<Self>, alloc: &EntityAlloc<Self>) -> PtrListRes<Self> {
        let curr_obj = curr.deref(alloc);
        if curr_obj.insts.is_none() {
            return Err(EntityListError::ItemFalselyDetached(curr));
        }
        curr_obj.parent_func.set(None);
        Ok(())
    }
}
impl ITraceableValue for BlockObj {
    fn users(&self) -> &UserList {
        self.users.as_ref().unwrap()
    }
    fn has_single_reference_semantics(&self) -> bool {
        true
    }
}
impl BlockObj {
    pub fn new(allocs: &IRAllocs) -> Self {
        Self {
            head: Cell::new(EntityListHead::none()),
            parent_func: Cell::new(None),
            insts: Some(EntityList::new(&allocs.insts)),
            users: Some(UserList::new(&allocs.uses)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockID(pub PtrID<BlockObj>);

impl BlockID {
    pub fn inner(self) -> PtrID<BlockObj> {
        self.0
    }

    pub fn deref_ir(self, allocs: &IRAllocs) -> &BlockObj {
        self.inner().deref(&allocs.blocks)
    }

    pub fn new(allocs: &IRAllocs) -> Self {
        let obj = BlockObj::new(allocs);
        let ptr_id = allocs.blocks.allocate(obj);
        Self(ptr_id)
    }
}
