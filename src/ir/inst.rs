use crate::{
    ir::{BlockID, IRAllocs, ITraceableValue, IUser, Opcode, OperandSet, UseID, UserList},
    typing::ValTypeID,
};
use mtb_entity::{
    EntityAlloc, EntityListError, EntityListHead, IEntityAllocID, IEntityListNode, PtrID,
    PtrListRes,
};
use std::cell::Cell;

pub struct InstCommon {
    pub node_head: Cell<EntityListHead<InstObj>>,
    pub parent_bb: Cell<Option<BlockID>>,
    pub users: Option<UserList>,
    pub opcode: Opcode,
    pub ret_type: ValTypeID,
}
impl Clone for InstCommon {
    fn clone(&self) -> Self {
        Self {
            node_head: Cell::new(EntityListHead::none()),
            parent_bb: Cell::new(self.parent_bb.get()),
            users: None,
            opcode: self.opcode,
            ret_type: self.ret_type,
        }
    }
}
impl InstCommon {
    pub fn deep_cloned(&self, allocs: &IRAllocs) -> Self {
        Self {
            node_head: Cell::new(EntityListHead::none()),
            parent_bb: Cell::new(None),
            users: Some(UserList::new(&allocs.uses)),
            opcode: self.opcode,
            ret_type: self.ret_type,
        }
    }

    pub fn new_sentinal() -> Self {
        Self {
            node_head: Cell::new(EntityListHead::none()),
            parent_bb: Cell::new(None),
            users: None,
            opcode: Opcode::GuideNode,
            ret_type: ValTypeID::Void,
        }
    }

    pub fn get_parent(&self) -> Option<BlockID> {
        self.parent_bb.get()
    }
    pub fn set_parent(&self, parent: Option<BlockID>) {
        self.parent_bb.set(parent);
    }
}

pub trait ISubInst: IUser {
    fn get_common(&self) -> &InstCommon;
    fn common_mut(&mut self) -> &mut InstCommon;

    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self>;
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self>;
    fn try_from_ir(inst: InstObj) -> Option<Self>;

    fn from_ir_ref(inst: &InstObj) -> &Self {
        Self::try_from_ir_ref(inst).expect("Invalid sub-instruction reference")
    }
    fn from_ir_mut(inst: &mut InstObj) -> &mut Self {
        Self::try_from_ir_mut(inst).expect("Invalid sub-instruction mutable reference")
    }
    fn from_ir(inst: InstObj) -> Self {
        Self::try_from_ir(inst).expect("Invalid sub-instruction")
    }

    fn into_ir(self) -> InstObj;
}
pub trait ISubInstID: Copy {
    type InstObjT: ISubInst + 'static;

    fn raw_from_ir(id: InstID) -> Self;
    fn into_ir(self) -> InstID;

    fn try_from_ir(id: InstID, allocs: &IRAllocs) -> Option<Self> {
        let inst = id.deref(&allocs.insts);
        Self::InstObjT::try_from_ir_ref(inst).map(|_| Self::raw_from_ir(id))
    }
    fn from_ir(id: InstID, allocs: &IRAllocs) -> Self {
        Self::try_from_ir(id, allocs).expect("Invalid sub-instruction ID")
    }

    fn deref_ir(self, allocs: &IRAllocs) -> &Self::InstObjT {
        let inst = self.into_ir().deref(&allocs.insts);
        Self::InstObjT::from_ir_ref(inst)
    }
    fn deref_ir_mut(self, allocs: &mut IRAllocs) -> &mut Self::InstObjT {
        let inst = self.into_ir().deref_mut(&mut allocs.insts);
        Self::InstObjT::from_ir_mut(inst)
    }

    fn get_common(self, allocs: &IRAllocs) -> &InstCommon {
        self.deref_ir(allocs).get_common()
    }
    fn common_mut(self, allocs: &mut IRAllocs) -> &mut InstCommon {
        self.deref_ir_mut(allocs).common_mut()
    }

    fn get_operands(self, allocs: &IRAllocs) -> OperandSet<'_> {
        self.deref_ir(allocs).get_operands()
    }
    fn operands_mut(self, allocs: &mut IRAllocs) -> &mut [UseID] {
        self.deref_ir_mut(allocs).operands_mut()
    }

    fn new(allocs: &IRAllocs, obj: Self::InstObjT) -> Self {
        let mut obj = obj.into_ir();
        if obj.get_common().users.is_none() && !obj.is_sentinal() {
            obj.common_mut().users = Some(UserList::new(&allocs.uses));
        }
        let id = allocs.insts.allocate(obj);
        Self::raw_from_ir(id)
    }
}

#[derive(Clone)]
pub enum InstObj {
    /// 指令链表的首尾引导结点, 不参与语义表达.
    GuideNode(InstCommon),

    /// 表示指令链表 “Phi 指令” 部分结束的结点, 不参与语义表达.
    PhiInstEnd(InstCommon),

    /// 表示 “所在基本块不可达”, 封死整个基本块的控制流.
    Unreachable(InstCommon),
}
pub type InstID = PtrID<InstObj>;

impl IUser for InstObj {
    fn get_operands(&self) -> OperandSet<'_> {
        use InstObj::*;
        match self {
            GuideNode(_) | PhiInstEnd(_) | Unreachable(_) => OperandSet::Fixed(&[]),
        }
    }

    fn operands_mut(&mut self) -> &mut [UseID] {
        use InstObj::*;
        match self {
            GuideNode(_) | PhiInstEnd(_) | Unreachable(_) => &mut [],
        }
    }
}
impl ITraceableValue for InstObj {
    fn try_get_users(&self) -> Option<&UserList> {
        self.get_common().users.as_ref()
    }

    fn users(&self) -> &UserList {
        self.get_common()
            .users
            .as_ref()
            .expect("Detected sentinal instruction")
    }

    fn has_single_reference_semantics(&self) -> bool {
        true
    }
}
impl ISubInst for InstObj {
    fn get_common(&self) -> &InstCommon {
        use InstObj::*;
        match self {
            GuideNode(c) | PhiInstEnd(c) | Unreachable(c) => c,
        }
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        use InstObj::*;
        match self {
            GuideNode(c) | PhiInstEnd(c) | Unreachable(c) => c,
        }
    }

    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        Some(inst)
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        Some(inst)
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        Some(inst)
    }

    fn into_ir(self) -> InstObj {
        self
    }
}
impl IEntityListNode for InstObj {
    fn load_head(&self) -> EntityListHead<Self> {
        self.get_common().node_head.get()
    }
    fn store_head(&self, head: EntityListHead<Self>) {
        self.get_common().node_head.set(head);
    }

    fn is_sentinal(&self) -> bool {
        matches!(self, InstObj::GuideNode(_))
    }
    fn new_sentinal() -> Self {
        InstObj::GuideNode(InstCommon::new_sentinal())
    }

    fn on_push_next(
        curr: PtrID<Self>,
        next: PtrID<Self>,
        alloc: &EntityAlloc<Self>,
    ) -> PtrListRes<Self> {
        if curr == next {
            return Err(EntityListError::RepeatedNode);
        }
        let parent = curr.deref(alloc).get_common().parent_bb.get();
        assert_ne!(parent, None, "Pushing inst without parent block");
        next.deref(alloc).get_common().parent_bb.set(parent);
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
        let parent = curr.deref(alloc).get_common().parent_bb.get();
        assert_ne!(parent, None, "Pushing inst without parent block");
        prev.deref(alloc).get_common().parent_bb.set(parent);
        Ok(())
    }
    fn on_unplug(curr: PtrID<Self>, alloc: &EntityAlloc<Self>) -> PtrListRes<Self> {
        let parent_bb = curr.deref(alloc).get_common().parent_bb.get();
        assert_ne!(parent_bb, None, "Unplugging inst without parent block");
        curr.deref(alloc).get_common().parent_bb.set(None);
        Ok(())
    }
}

impl ISubInstID for InstID {
    type InstObjT = InstObj;

    fn raw_from_ir(id: InstID) -> Self {
        id
    }
    fn into_ir(self) -> InstID {
        self
    }
}
