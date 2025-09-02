use crate::{
    base::{
        INullableValue, SlabListError, SlabListNode, SlabListNodeHead, SlabListNodeRef,
        SlabListRes, SlabRef,
    },
    ir::{
        BlockRef, FuncRef, IRAllocs, IRAllocsEditable, IRAllocsReadable, IRWriter, IReferenceValue,
        ISubValueSSA, ITerminatorInst, ITraceableValue, IUser, IUserRef, InstKind, JumpTargets,
        ManagedInst, Opcode, OperandSet, Use, UserID, UserList, ValueSSA, ValueSSAError,
    },
    typing::{TypeMismatchError, ValTypeID},
};
use slab::Slab;
use std::{cell::Cell, fmt::Debug, rc::Rc};

pub(crate) mod usedef;

mod alloca;
mod amormw;
mod binop;
mod br;
mod call;
mod cast;
mod cmp;
mod gep;
mod jump;
mod load;
mod phi;
mod ret;
mod select;
mod store;
mod switch;

pub use self::{
    alloca::{Alloca, AllocaRef},
    amormw::{AmoOrdering, AmoRmw, AmoRmwBuilder, AmoRmwRef, SyncScope},
    binop::{BinOp, BinOpRef},
    br::{Br, BrRef},
    call::{CallOp, CallOpRef},
    cast::{CastOp, CastOpRef},
    cmp::{CmpOp, CmpOpRef},
    gep::{
        GEPBuilder, GEPIndexIter, GEPRef, GEPTypeIndexer, GEPTypeState, IndexPtr, IrGEPOffset,
        IrGEPOffsetIter,
    },
    jump::{Jump, JumpRef},
    load::{LoadInstRef, LoadOp},
    phi::{PhiError, PhiNode, PhiRef},
    ret::{Ret, RetRef},
    select::{SelectOp, SelectOpRef},
    store::{StoreOp, StoreOpRef},
    switch::{Switch, SwitchRef},
};

#[derive(Debug, Clone, Copy)]
pub enum InstError {
    OperandNull,
    OperandUninit,
    OperandOverflow,
    OperandTypeMismatch(TypeMismatchError, ValueSSA),
    OperandError(ValueSSAError),
    OperandNotComptimeConst(ValueSSA),

    InvalidCast,
    InvalidArgumentCount(usize, usize),
    DividedByZero,

    SelfNotAttached(InstRef),
    SelfAlreadyAttached(InstRef, BlockRef),
    ListError(SlabListError),
    ReplicatedTerminator(InstRef, InstRef),
}

#[derive(Debug)]
pub enum InstData {
    /// 指令链表的首尾引导结点, 不参与语义表达.
    ListGuideNode(InstCommon),

    /// 表示指令链表 “Phi 指令” 部分结束的结点, 不参与语义表达.
    PhiInstEnd(InstCommon),

    /// 表示 “所在基本块不可达”, 封死整个基本块的控制流.
    Unreachable(InstCommon),

    /// 终止函数控制流并返回一个值.
    Ret(Ret),

    /// 无条件跳转到指定基本块.
    Jump(Jump),

    /// 条件分支指令, 根据条件跳转到不同的基本块.
    Br(Br),

    /// Switch 语句, 根据条件跳转到不同的 case 分支.
    Switch(Switch),

    /// 在栈上分配一段固定大小的内存.
    Alloca(Alloca),

    /// 二元操作
    BinOp(BinOp),

    /// 函数调用指令
    Call(CallOp),

    /// 类型转换指令
    Cast(CastOp),

    /// Phi 指令, 根据前驱基本块选择一个值.
    Phi(PhiNode),

    /// 比较两个值的关系, 产生一个布尔值.
    Cmp(CmpOp),

    /// 根据索引计算指针偏移, 用于数组或结构体访问.
    GEP(IndexPtr),

    /// 选择指令, 根据条件选择两个值中的一个.
    Select(SelectOp),

    /// 加载内存中的值到寄存器.
    Load(LoadOp),

    /// 存储寄存器中的值到内存.
    Store(StoreOp),

    /// 原子操作: 读取 - 修改 - 写回
    AmoRmw(AmoRmw),
}

impl IUser for InstData {
    fn get_operands(&self) -> OperandSet<'_> {
        match self {
            InstData::ListGuideNode(_) => OperandSet::Fixed(&[]),
            InstData::PhiInstEnd(_) => OperandSet::Fixed(&[]),
            InstData::Unreachable(_) => OperandSet::Fixed(&[]),
            InstData::Ret(ret) => ret.get_operands(),
            InstData::Jump(_) => OperandSet::Fixed(&[]),
            InstData::Br(br) => br.get_operands(),
            InstData::Switch(switch) => switch.get_operands(),
            InstData::Alloca(_) => OperandSet::Fixed(&[]),
            InstData::BinOp(binop) => binop.get_operands(),
            InstData::Call(call) => call.get_operands(),
            InstData::Cast(cast_op) => cast_op.get_operands(),
            InstData::Phi(phi) => phi.get_operands(),
            InstData::Cmp(cmp_op) => cmp_op.get_operands(),
            InstData::GEP(gep) => gep.get_operands(),
            InstData::Select(select_op) => select_op.get_operands(),
            InstData::Load(load) => load.get_operands(),
            InstData::Store(store) => store.get_operands(),
            InstData::AmoRmw(amo) => amo.get_operands(),
        }
    }

    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        match self {
            InstData::ListGuideNode(_) => &mut [],
            InstData::PhiInstEnd(_) => &mut [],
            InstData::Unreachable(_) => &mut [],
            InstData::Ret(ret) => ret.operands_mut(),
            InstData::Jump(_) => &mut [],
            InstData::Br(br) => br.operands_mut(),
            InstData::Switch(switch) => switch.operands_mut(),
            InstData::Alloca(_) => &mut [],
            InstData::BinOp(binop) => binop.operands_mut(),
            InstData::Call(call) => call.operands_mut(),
            InstData::Cast(cast_op) => cast_op.operands_mut(),
            InstData::Phi(phi) => phi.operands_mut(),
            InstData::Cmp(cmp_op) => cmp_op.operands_mut(),
            InstData::GEP(gep) => gep.operands_mut(),
            InstData::Select(select_op) => select_op.operands_mut(),
            InstData::Load(load) => load.operands_mut(),
            InstData::Store(store) => store.operands_mut(),
            InstData::AmoRmw(amo) => amo.operands_mut(),
        }
    }
}

impl ISubInst for InstData {
    fn new_empty(_: Opcode) -> Self {
        InstData::ListGuideNode(InstCommon::new_empty())
    }

    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        Some(inst)
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        Some(inst)
    }
    fn into_ir(self) -> InstData {
        self
    }

    fn get_common(&self) -> &InstCommon {
        match self {
            InstData::ListGuideNode(common) => common,
            InstData::PhiInstEnd(common) => common,
            InstData::Unreachable(common) => common,
            InstData::Ret(ret) => ret.get_common(),
            InstData::Jump(jump) => jump.get_common(),
            InstData::Br(br) => br.get_common(),
            InstData::Switch(switch) => switch.get_common(),
            InstData::Alloca(alloca) => alloca.get_common(),
            InstData::BinOp(binop) => binop.get_common(),
            InstData::Call(call) => call.get_common(),
            InstData::Cast(cast_op) => cast_op.get_common(),
            InstData::Phi(phi) => phi.get_common(),
            InstData::Cmp(cmp_op) => cmp_op.get_common(),
            InstData::GEP(gep) => gep.get_common(),
            InstData::Select(select_op) => select_op.get_common(),
            InstData::Load(load) => load.get_common(),
            InstData::Store(store) => store.get_common(),
            InstData::AmoRmw(amo) => amo.get_common(),
        }
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        match self {
            InstData::ListGuideNode(common) => common,
            InstData::PhiInstEnd(common) => common,
            InstData::Unreachable(common) => common,
            InstData::Ret(ret) => ret.common_mut(),
            InstData::Jump(jump) => jump.common_mut(),
            InstData::Br(br) => br.common_mut(),
            InstData::Switch(switch) => switch.common_mut(),
            InstData::Alloca(alloca) => alloca.common_mut(),
            InstData::BinOp(binop) => binop.common_mut(),
            InstData::Call(call) => call.common_mut(),
            InstData::Cast(cast_op) => cast_op.common_mut(),
            InstData::Phi(phi) => phi.common_mut(),
            InstData::Cmp(cmp_op) => cmp_op.common_mut(),
            InstData::GEP(gep) => gep.common_mut(),
            InstData::Select(select_op) => select_op.common_mut(),
            InstData::Load(load) => load.common_mut(),
            InstData::Store(store) => store.common_mut(),
            InstData::AmoRmw(amo) => amo.common_mut(),
        }
    }

    fn is_terminator(&self) -> bool {
        use InstData::*;
        matches!(self, Unreachable(_) | Ret(_) | Jump(_) | Br(_) | Switch(_))
    }

    fn fmt_ir(&self, id: Option<usize>, writer: &IRWriter) -> std::io::Result<()> {
        writer.write_ref(self.get_self_ref(), "Inst");
        writer.write_users(self.users());
        match self {
            InstData::ListGuideNode(_) => Ok(()),
            InstData::PhiInstEnd(_) => writer.write_str("; Phi Inst End Node"),
            InstData::Unreachable(_) => writer.write_str("unreachable"),
            InstData::Ret(inst) => inst.fmt_ir(id, writer),
            InstData::Jump(inst) => inst.fmt_ir(id, writer),
            InstData::Br(inst) => inst.fmt_ir(id, writer),
            InstData::Switch(inst) => inst.fmt_ir(id, writer),
            InstData::Alloca(inst) => inst.fmt_ir(id, writer),
            InstData::BinOp(inst) => inst.fmt_ir(id, writer),
            InstData::Call(inst) => inst.fmt_ir(id, writer),
            InstData::Cast(inst) => inst.fmt_ir(id, writer),
            InstData::Phi(inst) => inst.fmt_ir(id, writer),
            InstData::Cmp(inst) => inst.fmt_ir(id, writer),
            InstData::GEP(inst) => inst.fmt_ir(id, writer),
            InstData::Select(inst) => inst.fmt_ir(id, writer),
            InstData::Load(inst) => inst.fmt_ir(id, writer),
            InstData::Store(inst) => inst.fmt_ir(id, writer),
            InstData::AmoRmw(inst) => inst.fmt_ir(id, writer),
        }
    }

    fn init_self_reference(&mut self, self_ref: InstRef) {
        match self {
            InstData::ListGuideNode(i) => i.self_ref = self_ref,
            InstData::PhiInstEnd(i) => i.self_ref = self_ref,
            InstData::Unreachable(i) => i.self_ref = self_ref,
            InstData::Ret(ret) => ret.init_self_reference(self_ref),
            InstData::Jump(jump) => jump.init_self_reference(self_ref),
            InstData::Br(br) => br.init_self_reference(self_ref),
            InstData::Switch(switch) => switch.init_self_reference(self_ref),
            InstData::Alloca(alloca) => alloca.init_self_reference(self_ref),
            InstData::BinOp(binop) => binop.init_self_reference(self_ref),
            InstData::Call(call) => call.init_self_reference(self_ref),
            InstData::Cast(cast_op) => cast_op.init_self_reference(self_ref),
            InstData::Phi(phi) => phi.init_self_reference(self_ref),
            InstData::Cmp(cmp_op) => cmp_op.init_self_reference(self_ref),
            InstData::GEP(gep) => gep.init_self_reference(self_ref),
            InstData::Select(select_op) => select_op.init_self_reference(self_ref),
            InstData::Load(load) => load.init_self_reference(self_ref),
            InstData::Store(store) => store.init_self_reference(self_ref),
            InstData::AmoRmw(amo) => amo.init_self_reference(self_ref),
        }
    }

    fn cleanup(&self) {
        match self {
            InstData::ListGuideNode(_) => {}
            InstData::PhiInstEnd(_) => {}
            InstData::Unreachable(_) => {}
            InstData::Ret(ret) => ret.cleanup(),
            InstData::Jump(jump) => jump.cleanup(),
            InstData::Br(br) => br.cleanup(),
            InstData::Switch(switch) => switch.cleanup(),
            InstData::Alloca(alloca) => alloca.cleanup(),
            InstData::BinOp(binop) => binop.cleanup(),
            InstData::Call(call) => call.cleanup(),
            InstData::Cast(cast_op) => cast_op.cleanup(),
            InstData::Phi(phi) => phi.cleanup(),
            InstData::Cmp(cmp_op) => cmp_op.cleanup(),
            InstData::GEP(gep) => gep.cleanup(),
            InstData::Select(select_op) => select_op.cleanup(),
            InstData::Load(load) => load.cleanup(),
            InstData::Store(store) => store.cleanup(),
            InstData::AmoRmw(amo) => amo.cleanup(),
        }
        log::debug!(
            "InstData cleanup: opcode {:?} ref {:?}",
            self.get_opcode(),
            self.get_self_ref()
        );
    }
}

impl SlabListNode for InstData {
    fn new_guide() -> Self {
        InstData::ListGuideNode(InstCommon::new_empty())
    }
    fn load_node_head(&self) -> SlabListNodeHead {
        self.get_common().inner.get().node_head
    }
    fn store_node_head(&self, node_head: SlabListNodeHead) {
        let mut inner = self.get_common().inner.get();
        inner.node_head = node_head;
        self.get_common().inner.set(inner);
    }
}

impl ITraceableValue for InstData {
    fn users(&self) -> &UserList {
        &self.get_common().users
    }

    fn has_single_reference_semantics(&self) -> bool {
        true
    }
}

impl InstData {
    pub fn is_guide_node(&self) -> bool {
        matches!(self, InstData::ListGuideNode(_) | InstData::PhiInstEnd(_))
    }

    pub fn new_unreachable() -> Self {
        InstData::Unreachable(InstCommon::new(Opcode::Unreachable, ValTypeID::Void))
    }
    pub fn new_phi_inst_end() -> Self {
        InstData::PhiInstEnd(InstCommon::new(Opcode::PhiEnd, ValTypeID::Void))
    }

    pub fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        match self {
            InstData::Br(br) => Some(br.get_jts()),
            InstData::Switch(switch) => Some(switch.get_jts()),
            InstData::Jump(jump) => Some(jump.get_jts()),
            _ => None,
        }
    }

    fn basic_cleanup(inst: &impl ISubInst) {
        // 清理指令的用户列表
        inst.get_common().users.clear();
        // 清空所有操作数的引用
        for operand in &inst.get_operands() {
            operand.clean_operand();
        }
    }

    fn basic_init_self_reference(self_ref: InstRef, inst: &mut impl ISubInst) {
        inst.common_mut().self_ref = self_ref;
        for user in &inst.get_common().users {
            user.operand.set(ValueSSA::Inst(self_ref));
        }
        for operand in inst.operands_mut() {
            operand.user.set(UserID::Inst(self_ref));
        }
    }
}

pub trait ISubInst: Debug + Sized + IUser {
    fn new_empty(opcode: Opcode) -> Self;

    fn try_from_ir(inst: &InstData) -> Option<&Self>;
    fn from_ir(inst: &InstData) -> &Self {
        Self::try_from_ir(inst).expect("Expected a valid instruction data")
    }

    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self>;
    fn from_ir_mut(inst: &mut InstData) -> &mut Self {
        Self::try_from_ir_mut(inst).expect("Expected a valid instruction data")
    }

    fn into_ir(self) -> InstData;

    fn get_common(&self) -> &InstCommon;
    fn common_mut(&mut self) -> &mut InstCommon;

    fn get_opcode(&self) -> Opcode {
        self.get_common().opcode
    }
    fn get_self_ref(&self) -> InstRef {
        self.get_common().self_ref
    }
    fn get_parent_bb(&self) -> BlockRef {
        self.get_common().inner.get().parent_bb
    }
    fn set_parent_bb(&self, parent: BlockRef) {
        let mut inner = self.get_common().inner.get();
        inner.parent_bb = parent;
        self.get_common().inner.set(inner);
    }
    fn get_parent_func(&self, allocs: &IRAllocs) -> FuncRef {
        let parent_bb = self.get_parent_bb();
        let parent_block = parent_bb.to_data(&allocs.blocks);
        FuncRef(parent_block.get_parent_func())
    }
    fn get_valtype(&self) -> ValTypeID {
        self.get_common().ret_type
    }

    fn is_terminator(&self) -> bool;

    fn get_prev(&self) -> InstRef {
        InstRef::from_handle(self.get_common().inner.get().node_head.prev)
    }
    fn get_next(&self) -> InstRef {
        InstRef::from_handle(self.get_common().inner.get().node_head.next)
    }

    fn init_self_reference(&mut self, self_ref: InstRef) {
        InstData::basic_init_self_reference(self_ref, self);
    }

    fn fmt_ir(&self, id: Option<usize>, writer: &IRWriter) -> std::io::Result<()>;

    /// 清空所有与自身有关的引用, 用于 RAII 风格的清理.
    /// 注意: 这不会从基本块中移除自己.
    fn cleanup(&self) {
        InstData::basic_cleanup(self);
    }
}

#[derive(Debug)]
pub struct InstCommon {
    pub inner: Cell<InstInner>,
    pub users: UserList,
    pub opcode: Opcode,
    pub self_ref: InstRef,
    pub ret_type: ValTypeID,
}

impl Clone for InstCommon {
    fn clone(&self) -> Self {
        Self {
            inner: Cell::new(self.inner.get()),
            users: UserList::new_empty(),
            self_ref: self.self_ref,
            opcode: self.opcode,
            ret_type: self.ret_type,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InstInner {
    pub node_head: SlabListNodeHead,
    pub parent_bb: BlockRef,
}

impl InstCommon {
    pub fn new_empty() -> Self {
        Self {
            inner: Cell::new(InstInner {
                node_head: SlabListNodeHead::new(),
                parent_bb: BlockRef::new_null(),
            }),
            users: UserList::new_empty(),
            self_ref: InstRef::new_null(),
            opcode: Opcode::GuideNode,
            ret_type: ValTypeID::Void,
        }
    }
    pub fn new(opcode: Opcode, ret_type: ValTypeID) -> Self {
        Self {
            inner: Cell::new(InstInner {
                node_head: SlabListNodeHead::new(),
                parent_bb: BlockRef::new_null(),
            }),
            users: UserList::new_empty(),
            self_ref: InstRef::new_null(),
            opcode,
            ret_type,
        }
    }

    pub fn get_parent(&self) -> BlockRef {
        self.inner.get().parent_bb
    }
    pub fn set_parent(&self, parent: BlockRef) {
        let mut inner = self.inner.get();
        inner.parent_bb = parent;
        self.inner.set(inner);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstRef(usize);

impl SlabRef for InstRef {
    type RefObject = InstData;
    fn from_handle(handle: usize) -> Self {
        InstRef(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl Debug for InstRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InstRef({})", self.0)
    }
}

impl SlabListNodeRef for InstRef {
    fn on_node_push_next(curr: Self, next: Self, alloc: &Slab<InstData>) -> SlabListRes {
        curr.node_attach_set_parent(next, alloc)
    }
    fn on_node_push_prev(curr: Self, prev: Self, alloc: &Slab<InstData>) -> SlabListRes {
        curr.node_attach_set_parent(prev, alloc)
    }
    fn on_node_unplug(curr: Self, alloc: &Slab<InstData>) -> SlabListRes {
        curr.to_inst(alloc).set_parent_bb(BlockRef::new_null());
        Ok(())
    }
}

impl IReferenceValue for InstRef {
    type ValueDataT = InstData;

    fn to_value_data<'a>(self, allocs: &'a IRAllocs) -> &'a Self::ValueDataT
    where
        Self::ValueDataT: 'a,
    {
        self.to_inst(&allocs.insts)
    }

    fn to_value_data_mut<'a>(self, allocs: &'a mut IRAllocs) -> &'a mut Self::ValueDataT
    where
        Self::ValueDataT: 'a,
    {
        self.to_data_mut(&mut allocs.insts)
    }
}

impl ISubValueSSA for InstRef {
    fn try_from_ir(value: ValueSSA) -> Option<Self> {
        match value {
            ValueSSA::Inst(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_ir(self) -> ValueSSA {
        ValueSSA::Inst(self)
    }

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID {
        self.to_inst(&allocs.insts).get_common().ret_type
    }

    fn try_gettype_noalloc(self) -> Option<ValTypeID> {
        None
    }

    fn is_zero(&self, _: &IRAllocs) -> bool {
        false
    }

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        let id = writer.borrow_numbers().inst_get_number(*self);
        self.to_data(&writer.allocs.insts).fmt_ir(id, writer)
    }
}

impl ISubInstRef for InstRef {
    type InstDataT = InstData;

    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        inst_ref
    }
    fn into_raw(self) -> InstRef {
        self
    }
}

impl IUserRef for InstRef {}

impl InstRef {
    fn node_attach_set_parent(self, to_attach: Self, alloc: &Slab<InstData>) -> SlabListRes {
        let parent = self.to_inst(alloc).get_parent_bb();
        // 这里就不处理 parent == null 的情况了.
        // 当基本块对象刚刚构造、还没放到堆上时, parent 就可能是 null. 此时也有可能会 push 一些指令上去.
        let data = to_attach.to_inst(alloc);
        if data.get_parent_bb().is_nonnull() {
            return Err(SlabListError::PluggedItemAttached(to_attach.get_handle()));
        }
        data.set_parent_bb(parent);
        Ok(())
    }

    pub fn from_alloc(alloc: &mut Slab<InstData>, mut data: InstData) -> Self {
        let ret = Self::from_handle(alloc.vacant_key());
        data.init_self_reference(ret);
        alloc.insert(data);
        ret
    }

    pub fn new(allocs: &mut impl IRAllocsEditable, data: InstData) -> Self {
        let allocs = allocs.get_allocs_mutref();
        Self::from_alloc(&mut allocs.insts, data)
    }

    /// 如果自己在指令列表里, 就把自己移除掉.
    pub fn detach_self<'a>(
        self,
        allocs: &'a impl IRAllocsReadable,
    ) -> Result<ManagedInst<'a>, InstError> {
        let allocs = allocs.get_allocs_ref();
        let (parent, opcode) = {
            let data = self.to_inst(&allocs.insts);
            (data.get_parent_bb(), data.get_opcode())
        };
        if parent.is_null() {
            log::debug!("Trying to unplug an instruction {self:?} (opcode {opcode:?}) NOT in list");
            return Err(InstError::SelfNotAttached(self));
        }
        parent
            .insts_from_alloc(&allocs.blocks)
            .unplug_node(&allocs.insts, self)
            .map_err(InstError::ListError)?;
        Ok(ManagedInst::new(self, allocs))
    }

    pub fn get_parent(self, allocs: &impl IRAllocsReadable) -> BlockRef {
        self.to_inst(&allocs.get_allocs_ref().insts).get_parent_bb()
    }
    pub fn get_parent_from_alloc(self, alloc: &Slab<InstData>) -> BlockRef {
        self.to_inst(alloc).get_parent_bb()
    }

    pub fn get_parent_func(self, allocs: &impl IRAllocsReadable) -> FuncRef {
        let allocs = allocs.get_allocs_ref();
        let parent = self.to_inst(&allocs.insts).get_parent_bb();
        let func = parent.to_data(&allocs.blocks).get_parent_func();
        FuncRef(func)
    }
}

pub trait ISubInstRef: Sized + Clone {
    type InstDataT: ISubInst;

    fn from_raw_nocheck(inst_ref: InstRef) -> Self;
    fn into_raw(self) -> InstRef;

    fn try_from_inst(inst: InstRef, alloc: &Slab<InstData>) -> Option<Self> {
        let inst_data = inst.to_inst(alloc);
        match Self::InstDataT::try_from_ir(inst_data) {
            Some(_) => Some(Self::from_raw_nocheck(inst)),
            None => None,
        }
    }
    fn from_inst(inst: InstRef, alloc: &Slab<InstData>) -> Self {
        Self::try_from_inst(inst, alloc).expect("Expected a valid instruction reference")
    }

    fn as_inst(self, alloc: &Slab<InstData>) -> Option<&Self::InstDataT> {
        self.into_raw()
            .as_data(alloc)
            .and_then(|data| Self::InstDataT::try_from_ir(data))
    }
    fn as_inst_mut(self, alloc: &mut Slab<InstData>) -> Option<&mut Self::InstDataT> {
        self.into_raw()
            .as_data_mut(alloc)
            .and_then(|data| Self::InstDataT::try_from_ir_mut(data))
    }
    fn to_inst(self, alloc: &Slab<InstData>) -> &Self::InstDataT {
        let Some(data) = self.clone().as_inst(alloc) else {
            let raw = self.into_raw();
            panic!("Expected a valid instruction data reference for {raw:?}");
        };
        data
    }
    fn to_inst_mut(self, alloc: &mut Slab<InstData>) -> &mut Self::InstDataT {
        let Some(data) = self.clone().as_inst_mut(alloc) else {
            let raw = self.into_raw();
            panic!("Expected a valid instruction data reference for {raw:?}");
        };
        data
    }

    fn get_opcode(self, alloc: &Slab<InstData>) -> Opcode {
        self.to_inst(alloc).get_common().opcode
    }
    fn get_kind(self, allocs: &impl IRAllocsReadable) -> InstKind {
        self.get_opcode(&allocs.get_allocs_ref().insts).get_kind()
    }
}
