use crate::{
    impl_traceable_from_common,
    ir::{
        BlockID, IRAllocs, ISubValueSSA, ITraceableValue, IUser, JumpTargets, Opcode, OperandSet,
        UseID, UserList, ValueClass, ValueSSA,
        module::allocs::{IPoolAllocated, PoolAllocatedDisposeRes},
    },
    typing::{AggrType, TypeContext, ValTypeID},
};
use mtb_entity_slab::{
    EntityListError, EntityListNodeHead, EntityListRes, IEntityAllocID, IEntityListNodeID,
    IPoliciedID, IndexedID, entity_id,
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
mod extract;
mod insert;
mod phi;
mod select;

// aggregate field instructions
mod aggr_field_inst;

pub use self::{
    aggr_field_inst::{
        AggrFieldInstBuildErr, AggrFieldInstBuildRes, AggrFieldInstBuilderCommon,
        IAggrFieldInstBuildable,
    },
    alloca::{AllocaInst, AllocaInstID},
    amormw::{AmoOrdering, AmoRmwBuilder, AmoRmwInst, AmoRmwInstID, SyncScope},
    binop::{BinOPFlags, BinOPInst, BinOPInstID},
    br::{BrInst, BrInstID},
    call::{CallInst, CallInstBuilder, CallInstID},
    cast::{CastErr, CastInst, CastInstID},
    cmp::{CmpInst, CmpInstID},
    extract::{
        FieldExtractBuilder, FieldExtractInst, FieldExtractInstID, IndexExtractInst,
        IndexExtractInstID,
    },
    gep::{
        GEPInst, GEPInstBuilder, GEPInstID, GEPTypeIter, GEPTypeState, GEPTypeUnpack,
        GEPTypeUnpackRes, GEPUnpackErr,
    },
    insert::{
        FieldInsertBuilder, FieldInsertInst, FieldInsertInstID, IndexInsertInst, IndexInsertInstID,
    },
    jump::{JumpInst, JumpInstID},
    load::{LoadInst, LoadInstID},
    phi::{PhiInst, PhiInstDedup, PhiInstErr, PhiInstID, PhiInstRes},
    ret::{RetInst, RetInstID},
    select::{SelectInst, SelectInstID},
    store::{StoreInst, StoreInstID},
    switch::{SwitchInst, SwitchInstID},
    unreachable::{UnreachableInst, UnreachableInstID},
};

pub struct InstCommon {
    pub node_head: Cell<EntityListNodeHead<InstID>>,
    pub parent_bb: Cell<Option<BlockID>>,
    pub users: Option<UserList>,
    pub opcode: Opcode,
    pub ret_type: ValTypeID,
    pub(in crate::ir) disposed: Cell<bool>,
}
impl Clone for InstCommon {
    fn clone(&self) -> Self {
        Self {
            node_head: Cell::new(EntityListNodeHead::none()),
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
            node_head: Cell::new(EntityListNodeHead::none()),
            parent_bb: Cell::new(None),
            users: Some(UserList::new(&allocs.uses)),
            opcode: self.opcode,
            disposed: Cell::new(self.disposed.get()),
            ret_type: self.ret_type,
        }
    }

    pub fn new_sentinel() -> Self {
        Self {
            node_head: Cell::new(EntityListNodeHead::none()),
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
            node_head: Cell::new(EntityListNodeHead::none()),
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

pub trait ISubInst: IUser + Sized {
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

    fn is_terminator(&self) -> bool {
        false
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>>;
}
pub trait ISubInstID: Copy {
    type InstObjT: ISubInst + 'static;

    fn from_raw_ptr(ptr: <InstID as IPoliciedID>::BackID) -> Self;
    fn into_raw_ptr(self) -> <InstID as IPoliciedID>::BackID;

    fn raw_from(id: InstID) -> Self {
        Self::from_raw_ptr(id.into())
    }
    fn raw_into(self) -> InstID {
        InstID(self.into_raw_ptr())
    }

    fn try_from_instid(id: InstID, allocs: &IRAllocs) -> Option<Self> {
        let inst = id.0.deref(&allocs.insts);
        Self::InstObjT::try_from_ir_ref(inst).map(|_| Self::raw_from(id))
    }
    fn from_instid(id: InstID, allocs: &IRAllocs) -> Self {
        Self::try_from_instid(id, allocs).expect("Invalid sub-instruction ID")
    }

    fn try_deref_ir(self, allocs: &IRAllocs) -> Option<&Self::InstObjT> {
        let inst = self.into_raw_ptr().try_deref(&allocs.insts)?;
        if inst.is_disposed() {
            return None;
        }
        Self::InstObjT::try_from_ir_ref(inst)
    }
    fn try_deref_ir_mut(self, allocs: &mut IRAllocs) -> Option<&mut Self::InstObjT> {
        let inst = self.into_raw_ptr().deref_mut(&mut allocs.insts);
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
    fn get_indexed(self, allocs: &IRAllocs) -> InstIndex {
        let index = self
            .into_raw_ptr()
            .get_index(&allocs.insts)
            .expect("Error: Attempted to get indexed ID of freed InstID");
        InstIndex::from(index)
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
        let id = InstObj::allocate(allocs, obj.into_ir());
        Self::raw_from(id)
    }

    fn dispose(self, allocs: &IRAllocs) -> PoolAllocatedDisposeRes {
        InstObj::dispose_id(self.raw_into(), allocs)
    }
}
/// Implements `Debug` for a sub-instruction ID type -- showing target memory address.
#[macro_export]
macro_rules! _remusys_ir_subinst_id {
    ($IDType:ident, $ObjType:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $IDType(pub $crate::ir::inst::InstBackID);
        impl std::fmt::Debug for $IDType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let tyname = stringify!($IDType);
                let addr = self.into_raw_ptr();
                write!(f, "{tyname}({addr:p})",)
            }
        }
        impl $crate::ir::inst::ISubInstID for $IDType {
            type InstObjT = $ObjType;

            #[inline]
            fn from_raw_ptr(ptr: $crate::ir::inst::InstBackID) -> Self {
                $IDType(ptr)
            }
            #[inline]
            fn into_raw_ptr(self) -> $crate::ir::inst::InstBackID {
                self.0
            }
        }
    };
    ($IDType:ident, $ObjType:ident, terminator) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $IDType(pub $crate::ir::inst::InstBackID);
        impl std::fmt::Debug for $IDType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let tyname = stringify!($IDType);
                let addr = self.into_raw_ptr();
                write!(f, "{tyname}({addr:p})",)
            }
        }
        impl $crate::ir::inst::ISubInstID for $IDType {
            type InstObjT = $ObjType;

            #[inline]
            fn from_raw_ptr(ptr: $crate::ir::inst::InstBackID) -> Self {
                $IDType(ptr)
            }
            #[inline]
            fn into_raw_ptr(self) -> $crate::ir::inst::InstBackID {
                self.0
            }
            #[inline]
            fn is_terminator(self, _: &IRAllocs) -> bool {
                true
            }
        }
        impl $crate::ir::IValueConvert for $IDType {
            fn try_from_value(
                value: $crate::ir::ValueSSA,
                allocs: &$crate::ir::Module,
            ) -> Option<Self> {
                let inst_id = match value {
                    $crate::ir::ValueSSA::Inst(id) => id,
                    _ => return None,
                };
                Self::try_from_instid(inst_id, &allocs.allocs)
            }
            fn into_value(self) -> $crate::ir::ValueSSA {
                $crate::ir::ValueSSA::Inst(self.raw_into())
            }
        }
        impl $crate::ir::ITerminatorID for $IDType {}
    };
}

pub trait IAggregateInst: ISubInst {
    fn get_aggr_operand_type(&self) -> AggrType;
    fn get_elem_type(&self) -> ValTypeID;

    fn aggr_use(&self) -> UseID;
    fn get_aggr(&self, allocs: &IRAllocs) -> ValueSSA {
        self.aggr_use().get_operand(allocs)
    }
    fn set_aggr(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.aggr_use().set_operand(allocs, val);
    }
}

pub trait IAggrFieldInst: IAggregateInst {
    type DefaultBuilderT: IAggrFieldInstBuildable;

    fn get_field_indices(&self) -> &[u32];

    fn default_builder(aggr_type: AggrType) -> Self::DefaultBuilderT {
        Self::DefaultBuilderT::new(aggr_type)
    }
}

pub trait IAggrIndexInst: IAggregateInst {
    fn index_use(&self) -> UseID;

    fn get_index(&self, allocs: &IRAllocs) -> ValueSSA {
        self.index_use().get_operand(allocs)
    }
    fn set_index(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.index_use().set_operand(allocs, val);
    }

    fn new_uninit(allocs: &IRAllocs, tctx: &TypeContext, aggr_type: AggrType) -> Self;
}

#[entity_id(InstID, policy = 512, allocator_type = InstAlloc)]
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

    /// 从指针所示的内存中加载出一个 SSA 值.
    Load(LoadInst),

    /// 将一个 SSA 值存储到指针所示的存储区域.
    Store(StoreInst),

    // 其他指令
    /// 原子读取-修改-写入指令.
    AmoRmw(AmoRmwInst),

    /// 二元操作
    BinOP(BinOPInst),

    /// 调用一个函数.
    Call(CallInst),

    /// 类型转换指令.
    Cast(CastInst),

    /// 比较指令.
    Cmp(CmpInst),

    /// 根据变量索引从数组 / 向量中提取元素
    IndexExtract(IndexExtractInst),

    /// 根据常量索引列从数组 / 向量 / 结构体中提取元素
    FieldExtract(FieldExtractInst),

    /// 把数组 / 向量值 a 中的索引位 i 替换成元素 v 并返回新的数组 / 向量值。
    IndexInsert(IndexInsertInst),

    /// 把数组 / 结构体 / 向量聚合值 a 中的指定字段替换成元素 v 并返回新的聚合值。
    /// 字段位置通过常量索引链指定。
    FieldInsert(FieldInsertInst),

    /// Phi 节点：实现 SSA 形式中的 φ 函数
    Phi(PhiInst),

    /// 选择指令: 根据条件值选择两个操作数之一作为结果返回。
    Select(SelectInst),
}

pub type InstIndex = IndexedID<InstObj, <InstID as IPoliciedID>::PolicyT>;
pub type InstBackID = <InstID as IPoliciedID>::BackID;

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
            Load(load) => load.get_operands(),
            Store(store) => store.get_operands(),

            // Other instructions
            AmoRmw(amormw) => amormw.get_operands(),
            BinOP(binop) => binop.get_operands(),
            Call(call) => call.get_operands(),
            Cast(cast) => cast.get_operands(),
            Cmp(cmp) => cmp.get_operands(),
            IndexExtract(e) => e.get_operands(),
            FieldExtract(e) => e.get_operands(),
            IndexInsert(e) => e.get_operands(),
            FieldInsert(e) => e.get_operands(),
            Phi(phi) => phi.get_operands(),
            Select(select) => select.get_operands(),
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
            Load(load) => load.operands_mut(),
            Store(store) => store.operands_mut(),
            // Other instructions
            AmoRmw(amormw) => amormw.operands_mut(),
            BinOP(binop) => binop.operands_mut(),
            Call(call) => call.operands_mut(),
            Cast(cast) => cast.operands_mut(),
            Cmp(cmp) => cmp.operands_mut(),
            IndexExtract(e) => e.operands_mut(),
            FieldExtract(e) => e.operands_mut(),
            IndexInsert(e) => e.operands_mut(),
            FieldInsert(e) => e.operands_mut(),
            Phi(phi) => phi.operands_mut(),
            Select(select) => select.operands_mut(),
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
            Load(load) => load.get_common(),
            Store(store) => store.get_common(),
            // Other instructions
            AmoRmw(amormw) => amormw.get_common(),
            BinOP(binop) => binop.get_common(),
            Call(call) => call.get_common(),
            Cast(cast) => cast.get_common(),
            Cmp(cmp) => cmp.get_common(),
            IndexExtract(e) => e.get_common(),
            FieldExtract(e) => e.get_common(),
            IndexInsert(e) => e.get_common(),
            FieldInsert(e) => e.get_common(),
            Phi(phi) => phi.get_common(),
            Select(select) => select.get_common(),
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
            Load(load) => load.common_mut(),
            Store(store) => store.common_mut(),
            // Other instructions
            AmoRmw(amormw) => amormw.common_mut(),
            BinOP(binop) => binop.common_mut(),
            Call(call) => call.common_mut(),
            Cast(cast) => cast.common_mut(),
            Cmp(cmp) => cmp.common_mut(),
            IndexExtract(e) => e.common_mut(),
            FieldExtract(e) => e.common_mut(),
            IndexInsert(e) => e.common_mut(),
            FieldInsert(e) => e.common_mut(),
            Phi(phi) => phi.common_mut(),
            Select(select) => select.common_mut(),
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
        matches!(self, Unreachable(_) | Ret(_) | Jump(_) | Br(_) | Switch(_))
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
}
impl IEntityListNodeID for InstID {
    fn obj_load_head(obj: &InstObj) -> EntityListNodeHead<Self> {
        obj.get_common().node_head.get()
    }
    fn obj_store_head(obj: &InstObj, head: EntityListNodeHead<Self>) {
        obj.get_common().node_head.set(head);
    }

    fn obj_is_sentinel(obj: &InstObj) -> bool {
        matches!(obj, InstObj::GuideNode(_))
    }
    fn new_sentinel_obj() -> InstObj {
        InstObj::GuideNode(InstCommon::new_sentinel())
    }

    fn on_push_prev(self, prev: Self, alloc: &InstAlloc) -> EntityListRes<Self> {
        if self == prev {
            return Err(EntityListError::RepeatedNode);
        }
        let parent = self.deref_alloc(alloc).get_parent();
        // Parent block CAN BE None here. e.g. when parent block has not allocated
        // into IRAllocs yet.
        prev.deref_alloc(alloc).set_parent(parent);
        Ok(())
    }
    fn on_push_next(self, next: Self, alloc: &InstAlloc) -> EntityListRes<Self> {
        if self == next {
            return Err(EntityListError::RepeatedNode);
        }
        let parent = self.deref_alloc(alloc).get_parent();
        // Parent block CAN BE None here. e.g. when parent block has not allocated
        // into IRAllocs yet.
        next.deref_alloc(alloc).set_parent(parent);
        Ok(())
    }
    fn on_unplug(self, alloc: &InstAlloc) -> EntityListRes<Self> {
        // Parent block CAN BE None here. e.g. when parent block has not allocated
        // into IRAllocs yet.
        self.deref_alloc(alloc).set_parent(None);
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

impl std::fmt::Pointer for InstID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.into_raw_ptr().fmt(f)
    }
}
/// InstID should be implemented manually because of the macro_rules! subinst_id
/// contains `Debug` implementation which conflicts with the auto-derived one.
impl ISubInstID for InstID {
    type InstObjT = InstObj;

    #[inline]
    fn from_raw_ptr(ptr: <InstID as IPoliciedID>::BackID) -> Self {
        Self(ptr)
    }
    #[inline]
    fn into_raw_ptr(self) -> <InstID as IPoliciedID>::BackID {
        self.0
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
    fn is_zero_const(self, _: &IRAllocs) -> bool {
        false
    }

    fn can_trace(self) -> bool {
        true
    }
    fn try_get_users(self, allocs: &IRAllocs) -> Option<&UserList> {
        self.deref_ir(allocs).try_get_users()
    }
}
