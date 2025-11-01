use crate::{
    base::MixRef,
    impl_traceable_from_common,
    ir::{
        BlockID, IRAllocs, ISubValueSSA, ITraceableValue, IUser, JumpTargets, Opcode, OperandSet,
        UseID, UserList, ValueClass, ValueSSA,
    },
    typing::ValTypeID,
};
use mtb_entity::{
    EntityAlloc, EntityListError, EntityListHead, IEntityAllocID, IEntityListNode, IndexedID,
    PtrID, PtrListRes,
};
use std::cell::Cell;

// basic block terminators
mod br;
mod jump;
mod ret;
mod switch;
mod unreachable;

// pointer and memory instructions
mod alloca;
mod gep;
mod load;
mod store;

// other instructions (atomic op; data processing; phi, etc.)
mod amormw;
mod binop;
mod call;
mod cast;
mod cmp;
mod phi;
mod select;

pub use self::{
    alloca::*, amormw::*, binop::*, br::*, call::*, cast::*, cmp::*, gep::*, jump::*, load::*,
    phi::*, ret::*, select::*, store::*, switch::*, unreachable::*,
};

pub struct InstCommon {
    pub node_head: Cell<EntityListHead<InstObj>>,
    pub parent_bb: Cell<Option<BlockID>>,
    pub users: Option<UserList>,
    pub opcode: Opcode,
    disposed: Cell<bool>,
    pub ret_type: ValTypeID,
}
impl Clone for InstCommon {
    fn clone(&self) -> Self {
        Self {
            node_head: Cell::new(EntityListHead::none()),
            parent_bb: Cell::new(self.parent_bb.get()),
            users: None,
            opcode: self.opcode,
            disposed: Cell::new(self.disposed.get()),
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
            disposed: Cell::new(self.disposed.get()),
            ret_type: self.ret_type,
        }
    }

    pub fn new_sentinel() -> Self {
        Self {
            node_head: Cell::new(EntityListHead::none()),
            parent_bb: Cell::new(None),
            users: None,
            opcode: Opcode::GuideNode,
            disposed: Cell::new(false),
            ret_type: ValTypeID::Void,
        }
    }
    pub fn is_sentinel(&self) -> bool {
        self.opcode == Opcode::GuideNode
    }

    pub fn new(opcode: Opcode, ret_ty: ValTypeID) -> Self {
        Self {
            node_head: Cell::new(EntityListHead::none()),
            parent_bb: Cell::new(None),
            users: None,
            opcode,
            disposed: Cell::new(false),
            ret_type: ret_ty,
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

    fn get_opcode(&self) -> Opcode {
        self.get_common().opcode
    }
    fn get_valtype(&self) -> ValTypeID {
        self.get_common().ret_type
    }
    fn get_parent(&self) -> Option<BlockID> {
        self.get_common().parent_bb.get()
    }
    fn set_parent(&self, parent: Option<BlockID>) {
        self.get_common().parent_bb.set(parent);
    }

    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self>;
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self>;
    fn try_from_ir(inst: InstObj) -> Option<Self>
    where
        Self: Sized;

    fn from_ir_ref(inst: &InstObj) -> &Self {
        Self::try_from_ir_ref(inst).expect("Invalid sub-instruction reference")
    }
    fn from_ir_mut(inst: &mut InstObj) -> &mut Self {
        Self::try_from_ir_mut(inst).expect("Invalid sub-instruction mutable reference")
    }
    fn from_ir(inst: InstObj) -> Self
    where
        Self: Sized,
    {
        Self::try_from_ir(inst).expect("Invalid sub-instruction")
    }

    fn into_ir(self) -> InstObj;

    fn is_terminator(&self) -> bool {
        false
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>>;

    fn dispose(&self, allocs: &IRAllocs) {
        if self.get_common().disposed.get() {
            return;
        }
        self._common_dispose(allocs);
    }
    fn _common_dispose(&self, allocs: &IRAllocs) {
        let common = self.get_common();
        assert!(!common.disposed.get(), "Instruction already disposed");
        common.disposed.set(true);
        self.user_dispose(allocs);
        if let Some(jt_list) = self.try_get_jts() {
            for &jt_id in jt_list.iter() {
                jt_id.deref(allocs).dispose(allocs);
            }
        }
    }

    fn inst_init_self_id(&self, self_id: InstID, allocs: &IRAllocs) {
        self._common_init_self_id(self_id, allocs)
    }
    fn _common_init_self_id(&self, self_id: InstID, allocs: &IRAllocs) {
        self.user_init_self_id(allocs, self_id);
        let jt_list = self.try_get_jts().unwrap_or(MixRef::Fix(&[]));
        for &jt_id in jt_list.iter() {
            jt_id.set_terminator(allocs, self_id);
        }
    }
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

    fn try_deref_ir(self, allocs: &IRAllocs) -> Option<&Self::InstObjT> {
        let inst = self.into_ir().try_deref(&allocs.insts)?;
        if inst.is_disposed() {
            return None;
        }
        Self::InstObjT::try_from_ir_ref(inst)
    }
    fn try_deref_ir_mut(self, allocs: &mut IRAllocs) -> Option<&mut Self::InstObjT> {
        let inst = self.into_ir().deref_mut(&mut allocs.insts);
        if inst.is_disposed() {
            return None;
        }
        Self::InstObjT::try_from_ir_mut(inst)
    }
    fn is_alive(self, allocs: &IRAllocs) -> bool {
        self.try_deref_ir(allocs).is_some()
    }
    fn deref_ir(self, allocs: &IRAllocs) -> &Self::InstObjT {
        self.try_deref_ir(allocs)
            .expect("Error: Attempted to deref freed InstID")
    }
    fn deref_ir_mut(self, allocs: &mut IRAllocs) -> &mut Self::InstObjT {
        self.try_deref_ir_mut(allocs)
            .expect("Error: Attempted to deref freed InstID")
    }
    fn get_indexed(self, allocs: &IRAllocs) -> IndexedID<InstObj> {
        self.into_ir()
            .as_indexed(&allocs.insts)
            .expect("Error: Attempted to get indexed ID of freed InstID")
    }

    fn get_common(self, allocs: &IRAllocs) -> &InstCommon {
        self.deref_ir(allocs).get_common()
    }
    fn common_mut(self, allocs: &mut IRAllocs) -> &mut InstCommon {
        self.deref_ir_mut(allocs).common_mut()
    }
    fn get_opcode(self, allocs: &IRAllocs) -> Opcode {
        self.deref_ir(allocs).get_opcode()
    }
    fn get_rettype(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).get_valtype()
    }
    fn get_parent(self, allocs: &IRAllocs) -> Option<BlockID> {
        self.deref_ir(allocs).get_parent()
    }
    fn set_parent(self, allocs: &IRAllocs, parent: Option<BlockID>) {
        self.deref_ir(allocs).set_parent(parent);
    }

    fn get_operands(self, allocs: &IRAllocs) -> OperandSet<'_> {
        self.deref_ir(allocs).get_operands()
    }
    fn operands_mut(self, allocs: &mut IRAllocs) -> &mut [UseID] {
        self.deref_ir_mut(allocs).operands_mut()
    }

    fn is_terminator(self, allocs: &IRAllocs) -> bool {
        self.deref_ir(allocs).is_terminator()
    }
    fn try_get_jts(self, allocs: &IRAllocs) -> Option<JumpTargets<'_>> {
        self.deref_ir(allocs).try_get_jts()
    }

    fn allocate(allocs: &IRAllocs, obj: Self::InstObjT) -> Self {
        let mut obj = obj.into_ir();
        if obj.get_common().users.is_none() && !obj.is_sentinel() {
            obj.common_mut().users = Some(UserList::new(&allocs.uses));
        }
        let id = allocs.insts.allocate(obj);
        id.deref_ir(allocs).inst_init_self_id(id, allocs);
        Self::raw_from_ir(id)
    }

    fn dispose(self, allocs: &IRAllocs) {
        let Some(obj) = self.try_deref_ir(allocs) else {
            return;
        };
        obj.dispose(allocs);
    }
    fn delete(self, allocs: &mut IRAllocs) {
        self.dispose(allocs);
        let IRAllocs { insts, uses, jts, .. } = allocs;
        let obj = self.into_ir().deref(insts);
        for use_id in obj.get_operands() {
            use_id.inner().free(uses);
        }
        if let Some(users_sentinel) = obj.try_get_users() {
            users_sentinel.sentinel.free(uses);
        }
        if let Some(jt) = obj.try_get_jts() {
            for &jt in jt.iter() {
                jt.inner().free(jts);
            }
        }
        self.into_ir().free(&mut allocs.insts);
    }
}

pub enum InstObj {
    /// 指令链表的首尾引导结点, 不参与语义表达.
    GuideNode(InstCommon),

    /// 表示指令链表 “Phi 指令” 部分结束的结点, 不参与语义表达.
    PhiInstEnd(InstCommon),

    // 基本块终结指令
    /// 表示 “所在基本块不可达”, 封死整个基本块的控制流.
    Unreachable(UnreachableInst),

    /// 结束函数控制流, 并返回一个值
    Ret(RetInst),

    /// 无条件跳转到指定基本块.
    Jump(JumpInst),

    /// 条件分支跳转到两个指定基本块之一.
    Br(BrInst),

    /// 多路分支跳转到多个指定基本块之一.
    Switch(SwitchInst),

    // 指针与内存相关指令
    /// 在栈上分配内存.
    Alloca(AllocaInst),

    /// 获取复合类型元素地址的指针计算指令.
    GEP(GEPInst),

    // 其他指令
    /// 调用一个函数.
    Call(CallInst),
}
pub type InstID = PtrID<InstObj>;

impl IUser for InstObj {
    fn get_operands(&self) -> OperandSet<'_> {
        use InstObj::*;
        match self {
            GuideNode(_) | PhiInstEnd(_) | Unreachable(_) => OperandSet::Fixed(&[]),
            Ret(ret) => ret.get_operands(),
            Jump(jump) => jump.get_operands(),
            Br(br) => br.get_operands(),
            Switch(switch) => switch.get_operands(),

            // Pointer and memory instructions
            Alloca(alloca) => alloca.get_operands(),
            GEP(gep) => gep.get_operands(),

            // Other instructions
            Call(call) => call.get_operands(),
        }
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        use InstObj::*;
        match self {
            GuideNode(_) | PhiInstEnd(_) | Unreachable(_) => &mut [],
            // Basic block terminators
            Ret(ret) => ret.operands_mut(),
            Jump(jump) => jump.operands_mut(),
            Br(br) => br.operands_mut(),
            Switch(switch) => switch.operands_mut(),
            // Pointer and memory instructions
            Alloca(alloca) => alloca.operands_mut(),
            GEP(gep) => gep.operands_mut(),
            // Other instructions
            Call(call) => call.operands_mut(),
        }
    }
}
impl_traceable_from_common!(InstObj, true);
impl ISubInst for InstObj {
    fn get_common(&self) -> &InstCommon {
        use InstObj::*;
        match self {
            GuideNode(c) | PhiInstEnd(c) => c,
            // Basic block terminators
            Unreachable(c) => c.get_common(),
            Ret(ret) => ret.get_common(),
            Jump(jump) => jump.get_common(),
            Br(br) => br.get_common(),
            Switch(switch) => switch.get_common(),
            // Pointer and memory instructions
            Alloca(alloca) => alloca.get_common(),
            GEP(gep) => gep.get_common(),
            // Other instructions
            Call(call) => call.get_common(),
        }
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        use InstObj::*;
        match self {
            GuideNode(c) | PhiInstEnd(c) => c,
            // Basic block terminators
            Unreachable(c) => c.common_mut(),
            Ret(ret) => ret.common_mut(),
            Jump(jump) => jump.common_mut(),
            Br(br) => br.common_mut(),
            Switch(switch) => switch.common_mut(),
            // Pointer and memory instructions
            Alloca(alloca) => alloca.common_mut(),
            GEP(gep) => gep.common_mut(),
            // Other instructions
            Call(call) => call.common_mut(),
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

    fn is_terminator(&self) -> bool {
        use InstObj::*;
        match self {
            Unreachable(_) | Ret(_) | Jump(_) | Br(_) | Switch(_) => true,
            _ => false,
        }
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        use InstObj::*;
        match self {
            Unreachable(_) => Some(JumpTargets::Fix(&[])),
            Ret(ret) => ret.try_get_jts(),
            Jump(jump) => jump.try_get_jts(),
            Br(br) => br.try_get_jts(),
            Switch(switch) => switch.try_get_jts(),
            _ => None,
        }
    }

    fn dispose(&self, allocs: &IRAllocs) {
        use InstObj::*;
        if self.is_disposed() {
            return;
        }
        match self {
            GuideNode(_) | PhiInstEnd(_) | Unreachable(_) => self._common_dispose(allocs),
            Ret(ret) => ret.dispose(allocs),
            Jump(jump) => jump.dispose(allocs),
            Br(br) => br.dispose(allocs),
            Switch(switch) => switch.dispose(allocs),
            // Pointer and memory instructions
            Alloca(alloca) => alloca.dispose(allocs),
            GEP(gep) => gep.dispose(allocs),
            // Other instructions
            Call(call) => call.dispose(allocs),
        }
    }
}
impl IEntityListNode for InstObj {
    fn load_head(&self) -> EntityListHead<Self> {
        self.get_common().node_head.get()
    }
    fn store_head(&self, head: EntityListHead<Self>) {
        self.get_common().node_head.set(head);
    }

    fn is_sentinel(&self) -> bool {
        matches!(self, InstObj::GuideNode(_))
    }
    fn new_sentinel() -> Self {
        InstObj::GuideNode(InstCommon::new_sentinel())
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
impl InstObj {
    pub fn new_phi_end() -> Self {
        InstObj::PhiInstEnd(InstCommon::new(Opcode::PhiEnd, ValTypeID::Void))
    }
    pub fn new_unreachable() -> Self {
        Self::Unreachable(UnreachableInst::new())
    }

    pub fn is_disposed(&self) -> bool {
        self.get_common().disposed.get()
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
impl ISubValueSSA for InstID {
    fn get_class(self) -> ValueClass {
        ValueClass::Inst
    }
    fn try_from_ir(ir: ValueSSA) -> Option<Self> {
        match ir {
            ValueSSA::Inst(id) => Some(id),
            _ => None,
        }
    }
    fn into_ir(self) -> ValueSSA {
        ValueSSA::Inst(self)
    }

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).get_valtype()
    }

    fn can_trace(self) -> bool {
        true
    }
    fn try_get_users(self, allocs: &IRAllocs) -> Option<&UserList> {
        self.deref_ir(allocs).try_get_users()
    }
}
