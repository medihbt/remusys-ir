use crate::{
    ir::{
        GlobalID, IRAllocs, IRManaged, ISubInst, ISubInstID, ISubValueSSA, ITraceableValue, InstID,
        InstObj, JumpTargetID, JumpTargets, ManagedInst, PredList, TerminatorID, UserList,
        ValueClass, ValueSSA,
    },
    typing::ValTypeID,
};
use mtb_entity::{
    EntityAlloc, EntityList, EntityListError, EntityListHead, IEntityAllocID, IEntityListNode,
    IndexedID, PtrID, PtrListRes,
};
use std::cell::Cell;

pub struct BlockObj {
    head: Cell<EntityListHead<BlockObj>>,
    parent_func: Cell<Option<GlobalID>>,
    body: Option<BlockObjBody>,
    dispose_mark: Cell<bool>,
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
        let phi_end = InstID::allocate(allocs, InstObj::new_phi_end());
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
            dispose_mark: Cell::new(false),
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
    pub fn new_uninit(allocs: &IRAllocs) -> Self {
        Self {
            head: Cell::new(EntityListHead::none()),
            parent_func: Cell::new(None),
            body: Some(BlockObjBody::new(allocs)),
            dispose_mark: Cell::new(false),
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
    pub(super) fn add_pred(&self, allocs: &IRAllocs, jt_id: JumpTargetID) {
        self.get_body()
            .preds
            .push_back_id(jt_id.inner(), &allocs.jts)
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
    pub fn set_terminator_inst<'alloc>(
        &self,
        allocs: &'alloc IRAllocs,
        inst_id: InstID,
    ) -> Option<ManagedInst<'alloc>> {
        if !inst_id.is_terminator(allocs) {
            panic!("Attempted to set non-terminator {inst_id:?} as terminator of {self:p}");
        }
        let insts = &self.get_body().insts;
        let old_terminator = insts.get_back_id(&allocs.insts);
        if old_terminator == Some(inst_id) {
            panic!("Attempted to set existing terminator {inst_id:?} as terminator of {self:p}");
        }
        if let Some(old_id) = old_terminator {
            insts
                .node_unplug(old_id, &allocs.insts)
                .expect("Failed to unplug old terminator InstID from BlockObj insts");
        }
        insts
            .push_back_id(inst_id, &allocs.insts)
            .expect("Failed to add new terminator InstID to BlockObj insts");
        old_terminator.map(|t| IRManaged::new(allocs, t))
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
    pub fn set_terminator<'allocs>(
        &self,
        allocs: &'allocs IRAllocs,
        term_id: TerminatorID,
    ) -> Option<ManagedInst<'allocs>> {
        self.set_terminator_inst(allocs, term_id.into_ir())
    }

    pub fn try_get_succs<'ir>(&self, allocs: &'ir IRAllocs) -> Option<JumpTargets<'ir>> {
        let term_inst = self.try_get_terminator(allocs)?;
        term_inst.try_get_jts(allocs)
    }
    pub fn get_succs<'ir>(&self, allocs: &'ir IRAllocs) -> JumpTargets<'ir> {
        self.try_get_succs(allocs)
            .expect("Attempted to get JumpTargets of BlockObj without terminator")
    }

    pub fn is_disposed(&self) -> bool {
        self.dispose_mark.get()
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
    pub fn try_get_terminator(self, allocs: &IRAllocs) -> Option<InstID> {
        self.deref_ir(allocs).try_get_terminator(allocs)
    }
    pub fn get_terminator_inst(self, allocs: &IRAllocs) -> InstID {
        self.deref_ir(allocs).get_terminator_inst(allocs)
    }
    pub fn set_terminator_inst(
        self,
        allocs: &IRAllocs,
        inst_id: InstID,
    ) -> Option<ManagedInst<'_>> {
        self.deref_ir(allocs).set_terminator_inst(allocs, inst_id)
    }
    pub fn try_get_succs<'ir>(self, allocs: &'ir IRAllocs) -> Option<JumpTargets<'ir>> {
        self.deref_ir(allocs).try_get_succs(allocs)
    }
    pub fn get_succs<'ir>(self, allocs: &'ir IRAllocs) -> JumpTargets<'ir> {
        self.deref_ir(allocs).get_succs(allocs)
    }

    pub fn allocate(allocs: &IRAllocs, mut obj: BlockObj) -> Self {
        if let None = obj.body {
            obj.body = Some(BlockObjBody::new(allocs));
        }
        let ptr_id = allocs.blocks.allocate(obj);
        let block_id = Self(ptr_id);
        block_id.get_body(allocs).init_self_id(block_id, allocs);
        block_id
    }
    pub fn new_uninit(allocs: &IRAllocs) -> Self {
        Self::allocate(allocs, BlockObj::new_uninit(allocs))
    }
    pub fn new_with_terminator(allocs: &IRAllocs, terminator: impl ISubInstID) -> Self {
        let ret = Self::new_uninit(allocs);
        ret.set_terminator_inst(allocs, terminator.into_ir());
        ret
    }
    pub fn dispose(self, allocs: &IRAllocs) {
        let Some(obj) = self.0.try_deref(&allocs.blocks) else {
            return;
        };
        if obj.dispose_mark.get() {
            return;
        }
        obj.dispose_mark.set(true);
        let Some(body) = &obj.body else {
            return;
        };
        for (inst_id, _) in body.insts.iter(&allocs.insts) {
            inst_id.dispose(allocs);
        }
        obj.parent_func.set(None);

        body.preds.clean(&allocs.jts);
        JumpTargetID(body.preds.sentinel).dispose(allocs);
        obj.traceable_dispose(allocs);
    }
}
