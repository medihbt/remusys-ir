use crate::{
    ir::{
        FuncID, IRAllocs, ISubInstID, ISubValueSSA, ITraceableValue, InstID, InstObj, JumpTargetID,
        JumpTargets, ManagedInst, PredList, TerminatorID, UserList, ValueClass, ValueSSA,
        module::allocs::{IPoolAllocated, PoolAllocatedDisposeRes},
    },
    typing::ValTypeID,
};
use mtb_entity_slab::{
    EntityList, EntityListError, EntityListNodeHead, EntityListRes, IEntityAllocID,
    IEntityListNodeID, IPolicyPtrID, IndexedID, PtrID, entity_ptr_id,
};
use std::cell::Cell;

type TermiReplaceRes<'ir> = Result<Option<ManagedInst<'ir>>, EntityListError<InstID>>;

#[entity_ptr_id(BlockID, policy = 256, allocator_type = BlockAlloc)]
pub struct BlockObj {
    pub(crate) head: Cell<EntityListNodeHead<BlockID>>,
    pub(crate) parent_func: Cell<Option<FuncID>>,
    pub(crate) body: Option<BlockObjBody>,
    pub(crate) dispose_mark: Cell<bool>,
}
pub(in crate::ir) type BlockRawPtr = PtrID<BlockObj, <BlockID as IPolicyPtrID>::PolicyT>;
pub(in crate::ir) type BlockIndex = IndexedID<BlockObj, <BlockID as IPolicyPtrID>::PolicyT>;
pub struct BlockObjBody {
    pub insts: EntityList<InstID>,
    pub phi_end: InstID,
    pub users: UserList,
    pub preds: PredList,
}
impl BlockObjBody {
    pub(crate) fn new(allocs: &IRAllocs) -> Self {
        let insts = EntityList::new(&allocs.insts);
        let phi_end = InstID::allocate(allocs, InstObj::new_phi_end());
        insts
            .push_back_id(phi_end, &allocs.insts)
            .expect("Failed to add phi_end to new BlockObjBody");
        let users = UserList::new(&allocs.uses);
        let preds = PredList::new(&allocs.jts);
        Self { insts, phi_end, users, preds }
    }
}
impl IEntityListNodeID for BlockID {
    fn obj_load_head(obj: &BlockObj) -> EntityListNodeHead<Self> {
        obj.head.get()
    }
    fn obj_store_head(obj: &BlockObj, head: EntityListNodeHead<Self>) {
        obj.head.set(head);
    }
    fn obj_is_sentinel(obj: &BlockObj) -> bool {
        obj.body.is_none()
    }
    fn new_sentinel_obj() -> BlockObj {
        BlockObj {
            head: Cell::new(EntityListNodeHead::none()),
            parent_func: Cell::new(None),
            body: None,
            dispose_mark: Cell::new(false),
        }
    }
    fn on_push_prev(self, prev: Self, alloc: &BlockAlloc) -> EntityListRes<Self> {
        if self == prev {
            return Err(EntityListError::RepeatedNode);
        }
        let parent = self.deref_alloc(alloc).get_parent_func();
        // It is legal to push a block without a parent function, so no assert here.
        prev.deref_alloc(alloc).parent_func.set(parent);
        Ok(())
    }
    fn on_push_next(self, next: Self, alloc: &BlockAlloc) -> EntityListRes<Self> {
        if self == next {
            return Err(EntityListError::RepeatedNode);
        }
        let parent = self.deref_alloc(alloc).get_parent_func();
        // It is legal to push a block without a parent function, so no assert here.
        next.deref_alloc(alloc).parent_func.set(parent);
        Ok(())
    }
    fn on_unplug(self, alloc: &BlockAlloc) -> EntityListRes<Self> {
        let curr_obj = self.deref_alloc(alloc);
        if curr_obj.body.is_none() {
            return Err(EntityListError::ItemFalselyDetached(self));
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
    pub fn new_uninit(allocs: &IRAllocs) -> Self {
        Self {
            head: Cell::new(EntityListNodeHead::none()),
            parent_func: Cell::new(None),
            body: Some(BlockObjBody::new(allocs)),
            dispose_mark: Cell::new(false),
        }
    }

    pub fn get_parent_func(&self) -> Option<FuncID> {
        self.parent_func.get()
    }
    pub(super) fn set_parent_func(&self, func: FuncID) {
        self.parent_func.set(Some(func));
    }
    pub(super) fn try_get_body(&self) -> Option<&BlockObjBody> {
        self.body.as_ref()
    }
    pub fn get_body(&self) -> &BlockObjBody {
        self.body
            .as_ref()
            .expect("Error: Attempted to access body of sentinel BlockObj")
    }
    pub fn get_preds(&self) -> &PredList {
        &self.get_body().preds
    }
    pub(super) fn add_pred(&self, allocs: &IRAllocs, jt_id: JumpTargetID) {
        self.get_body()
            .preds
            .push_back(jt_id, &allocs.jts)
            .expect("Failed to add JumpTarget to BlockObj preds");
    }

    pub fn try_get_terminator(&self, allocs: &IRAllocs) -> Option<InstID> {
        let back = self.get_body().insts.get_back_id(&allocs.insts)?;
        if back.is_terminator(allocs) { Some(back) } else { None }
    }
    pub fn get_terminator_inst(&self, allocs: &IRAllocs) -> InstID {
        self.try_get_terminator(allocs)
            .expect("Attempted to get terminator of BlockObj without terminator")
    }
    pub fn try_set_terminator_inst<'ir>(
        &self,
        allocs: &'ir IRAllocs,
        inst_id: InstID,
    ) -> TermiReplaceRes<'ir> {
        if !inst_id.is_terminator(allocs) {
            panic!("Attempted to set non-terminator {inst_id:?} as terminator of {self:p}");
        }
        let insts = &self.get_body().insts;
        let back = insts
            .get_back_id(&allocs.insts)
            .expect("Found empty list: BasicBlock inst list should have at least 1 inst (PhiEnd)");
        let old_terminator = if back.is_terminator(allocs) { Some(back) } else { None };
        if let Some(old_id) = old_terminator {
            insts.node_unplug(old_id, &allocs.insts)?;
        }
        insts.push_back_id(inst_id, &allocs.insts)?;
        let managed_old = old_terminator.map(|t| ManagedInst::new(allocs, t));
        Ok(managed_old)
    }
    pub fn set_terminator_inst<'ir>(
        &self,
        allocs: &'ir IRAllocs,
        termi: InstID,
    ) -> Option<ManagedInst<'ir>> {
        self.try_set_terminator_inst(allocs, termi)
            .expect("Failed to set terminator inst for BlockObj")
    }
    pub fn get_terminator(&self, allocs: &IRAllocs) -> TerminatorID {
        let back = self
            .get_body()
            .insts
            .get_back_id(&allocs.insts)
            .expect("Attempted to get terminator of BlockObj without terminator");
        TerminatorID::try_from_ir(allocs, back)
            .expect("Terminator InstID of BlockObj is not a valid TerminatorID")
    }
    pub fn try_set_terminator<'ir>(
        &self,
        allocs: &'ir IRAllocs,
        term_id: impl Into<TerminatorID>,
    ) -> TermiReplaceRes<'ir> {
        self.try_set_terminator_inst(allocs, term_id.into().into_ir())
    }
    pub fn set_terminator<'ir>(
        &self,
        allocs: &'ir IRAllocs,
        term_id: impl Into<TerminatorID>,
    ) -> Option<ManagedInst<'ir>> {
        self.try_set_terminator(allocs, term_id)
            .expect("Failed to set terminator for BlockObj")
    }

    pub fn try_get_succs<'ir>(&self, allocs: &'ir IRAllocs) -> Option<JumpTargets<'ir>> {
        let term_inst = self.try_get_terminator(allocs)?;
        term_inst.try_get_jts(allocs)
    }
    pub fn get_succs<'ir>(&self, allocs: &'ir IRAllocs) -> JumpTargets<'ir> {
        self.try_get_succs(allocs)
            .expect("Attempted to get JumpTargets of BlockObj without terminator")
    }
}

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
    fn is_zero_const(self, _: &IRAllocs) -> bool {
        false
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
    pub fn inner(self) -> BlockRawPtr {
        self.0
    }

    pub fn deref_ir(self, allocs: &IRAllocs) -> &BlockObj {
        self.inner().deref(&allocs.blocks)
    }
    pub fn deref_ir_mut(self, allocs: &mut IRAllocs) -> &mut BlockObj {
        self.inner().deref_mut(&mut allocs.blocks)
    }
    pub fn get_indexed(self, allocs: &IRAllocs) -> BlockIndex {
        let index = self
            .inner()
            .get_index(&allocs.blocks)
            .expect("Error: Attempted to get indexed ID of freed BlockID");
        BlockIndex::from(index)
    }

    pub fn get_parent_func(self, allocs: &IRAllocs) -> Option<FuncID> {
        self.deref_ir(allocs).parent_func.get()
    }
    pub fn set_parent_func(self, allocs: &IRAllocs, func: FuncID) {
        self.deref_ir(allocs).parent_func.set(Some(func));
    }

    pub fn get_body(self, allocs: &IRAllocs) -> &BlockObjBody {
        self.deref_ir(allocs).get_body()
    }
    pub fn get_insts(self, allocs: &IRAllocs) -> &EntityList<InstID> {
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
    pub fn try_get_terminator(self, allocs: &IRAllocs) -> Option<InstID> {
        self.deref_ir(allocs).try_get_terminator(allocs)
    }
    pub fn get_terminator_inst(self, allocs: &IRAllocs) -> InstID {
        self.deref_ir(allocs).get_terminator_inst(allocs)
    }
    pub fn try_set_terminator_inst(
        self,
        allocs: &IRAllocs,
        inst_id: InstID,
    ) -> TermiReplaceRes<'_> {
        self.deref_ir(allocs)
            .try_set_terminator_inst(allocs, inst_id)
    }
    pub fn set_terminator_inst(self, allocs: &IRAllocs, termi: InstID) -> Option<ManagedInst<'_>> {
        self.deref_ir(allocs).set_terminator_inst(allocs, termi)
    }
    pub fn try_get_succs<'ir>(self, allocs: &'ir IRAllocs) -> Option<JumpTargets<'ir>> {
        self.deref_ir(allocs).try_get_succs(allocs)
    }
    pub fn get_succs<'ir>(self, allocs: &'ir IRAllocs) -> JumpTargets<'ir> {
        self.deref_ir(allocs).get_succs(allocs)
    }

    pub fn new_uninit(allocs: &IRAllocs) -> Self {
        BlockObj::allocate(allocs, BlockObj::new_uninit(allocs))
    }
    pub fn new_with_terminator(allocs: &IRAllocs, terminator: impl ISubInstID) -> Self {
        let ret = Self::new_uninit(allocs);
        ret.set_terminator_inst(allocs, terminator.raw_into());
        ret
    }
    pub fn dispose(self, allocs: &IRAllocs) -> PoolAllocatedDisposeRes {
        BlockObj::dispose_id(self, allocs)
    }
}
