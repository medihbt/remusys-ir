use crate::{ir::{
    GlobalID, IRAllocs, ISubInst, ISubInstID, ISubValueSSA, ITraceableValue, InstID, InstObj,
    JumpTargetID, PredList, UserList, ValueClass, ValueSSA,
}, typing::ValTypeID};
use mtb_entity::{
    EntityAlloc, EntityList, EntityListError, EntityListHead, IEntityAllocID, IEntityListNode,
    IndexedID, PtrID, PtrListRes,
};
use std::cell::Cell;

pub struct BlockObj {
    head: Cell<EntityListHead<BlockObj>>,
    parent_func: Cell<Option<GlobalID>>,
    body: Option<BlockObjBody>,
}
pub struct BlockObjBody {
    pub insts: EntityList<InstObj>,
    pub phi_end: InstID,
    pub users: UserList,
    pub preds: PredList,
}
impl BlockObjBody {
    fn new(allocs: &IRAllocs) -> Self {
        let insts = EntityList::new(&allocs.insts);
        let phi_end = InstID::new(allocs, InstObj::new_phi_end());
        insts
            .push_back_id(phi_end, &allocs.insts)
            .expect("Failed to add phi_end to new BlockObjBody");
        let users = UserList::new(&allocs.uses);
        let preds = PredList::new(&allocs.jts);
        Self { insts, phi_end, users, preds }
    }

    fn init_self_id(&self, self_id: BlockID, allocs: &IRAllocs) {
        let init_inst = |inst: InstID| {
            inst.deref_ir(allocs).get_common().set_parent(Some(self_id));
        };
        init_inst(self.insts.head);
        init_inst(self.phi_end);
        init_inst(self.insts.tail);
    }
}

impl IEntityListNode for BlockObj {
    fn load_head(&self) -> EntityListHead<Self> {
        self.head.get()
    }
    fn store_head(&self, head: EntityListHead<Self>) {
        self.head.set(head);
    }

    fn is_sentinel(&self) -> bool {
        self.body.is_none()
    }
    fn new_sentinel() -> Self {
        Self {
            head: Cell::new(EntityListHead::none()),
            parent_func: Cell::new(None),
            body: None,
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
        if curr_obj.body.is_none() {
            return Err(EntityListError::ItemFalselyDetached(curr));
        }
        curr_obj.parent_func.set(None);
        Ok(())
    }
}
impl ITraceableValue for BlockObj {
    fn users(&self) -> &UserList {
        &self.get_body().users
    }
    fn has_unique_ref_semantics(&self) -> bool {
        true
    }
}
impl BlockObj {
    pub fn new(allocs: &IRAllocs) -> Self {
        Self {
            head: Cell::new(EntityListHead::none()),
            parent_func: Cell::new(None),
            body: Some(BlockObjBody::new(allocs)),
        }
    }

    pub fn get_body(&self) -> &BlockObjBody {
        self.body
            .as_ref()
            .expect("Error: Attempted to access body of sentinel BlockObj")
    }
    pub fn get_preds(&self) -> &PredList {
        &self.get_body().preds
    }

    pub(crate) fn add_jump_target(&self, allocs: &IRAllocs, jt_id: JumpTargetID) {
        self.get_body()
            .preds
            .push_back_id(jt_id.inner(), &allocs.jts)
            .expect("Failed to add JumpTarget to BlockObj preds");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockID(pub PtrID<BlockObj>);

impl ISubValueSSA for BlockID {
    fn try_from_ir(ir: ValueSSA) -> Option<Self> {
        match ir {
            ValueSSA::Block(id) => Some(id),
            _ => None,
        }
    }
    fn into_ir(self) -> ValueSSA {
        ValueSSA::Block(self)
    }

    fn get_valtype(self, _: &IRAllocs) -> ValTypeID {
        ValTypeID::Void
    }

    fn can_trace(self) -> bool {
        true
    }
    fn get_class(self) -> ValueClass {
        ValueClass::Block
    }
    fn try_get_users(self, allocs: &IRAllocs) -> Option<&UserList> {
        self.deref_ir(allocs).try_get_users()
    }
}

impl BlockID {
    pub fn inner(self) -> PtrID<BlockObj> {
        self.0
    }

    pub fn deref_ir(self, allocs: &IRAllocs) -> &BlockObj {
        self.inner().deref(&allocs.blocks)
    }
    pub fn deref_ir_mut(self, allocs: &mut IRAllocs) -> &mut BlockObj {
        self.inner().deref_mut(&mut allocs.blocks)
    }
    pub fn get_indexed(self, allocs: &IRAllocs) -> IndexedID<BlockObj> {
        self.inner()
            .as_indexed(&allocs.blocks)
            .expect("Error: Attempted to get indexed ID of freed BlockID")
    }

    pub fn get_parent_func(self, allocs: &IRAllocs) -> Option<GlobalID> {
        self.deref_ir(allocs).parent_func.get()
    }
    pub fn set_parent_func(self, allocs: &IRAllocs, func: GlobalID) {
        self.deref_ir(allocs).parent_func.set(Some(func));
    }

    pub fn get_body(self, allocs: &IRAllocs) -> &BlockObjBody {
        self.deref_ir(allocs).get_body()
    }
    pub fn get_insts(self, allocs: &IRAllocs) -> &EntityList<InstObj> {
        &self.get_body(allocs).insts
    }
    pub fn get_phi_end(self, allocs: &IRAllocs) -> InstID {
        self.get_body(allocs).phi_end
    }
    pub fn get_users(self, allocs: &IRAllocs) -> &UserList {
        &self.get_body(allocs).users
    }
    pub fn get_preds(self, allocs: &IRAllocs) -> &PredList {
        &self.get_body(allocs).preds
    }

    pub fn new(allocs: &IRAllocs, mut obj: BlockObj) -> Self {
        if let None = obj.body {
            obj.body = Some(BlockObjBody::new(allocs));
        }
        let ptr_id = allocs.blocks.allocate(obj);
        let block_id = Self(ptr_id);
        block_id.get_body(allocs).init_self_id(block_id, allocs);
        block_id
    }
    pub fn dispose(self, allocs: &IRAllocs) {
        let obj = self.deref_ir(allocs);
        let Some(body) = &obj.body else {
            return;
        };
        for (inst_id, _) in body.insts.iter(&allocs.insts) {
            inst_id.dispose(allocs);
        }
        body.users.clean(&allocs.uses);
        body.preds.clean(&allocs.jts);
        obj.parent_func.set(None);
    }

    pub fn delete(self, allocs: &mut IRAllocs) {
        self.dispose(allocs);
        self.0.free(&mut allocs.blocks);
    }
}
