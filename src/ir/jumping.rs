use crate::{
    base::{MixRef, MixRefIter},
    ir::{
        BlockID, IRAllocs, ISubInst, ISubInstID, InstID, InstObj,
        inst::{
            BrInst, BrInstID, JumpInst, JumpInstID, RetInst, RetInstID, SwitchInst, SwitchInstID,
            UnreachableInst, UnreachableInstID,
        },
        module::allocs::{IPoolAllocated, PoolAllocatedDisposeRes},
    },
};
use mtb_entity_slab::{
    EntityAlloc, EntityListHead, EntityRingList, IEntityAllocID, IEntityRingListNode, PtrID,
};
use std::{
    cell::Cell,
    collections::{BTreeSet, HashSet},
    fmt::Debug,
};

/// 跳转目标的类型，用于区分不同的控制流转移
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JumpTargetKind {
    /// 无效/哨兵类型，用于链表哨兵节点
    None,
    /// 无条件跳转
    Jump,
    /// 条件分支的真分支
    BrThen,
    /// 条件分支的假分支
    BrElse,
    /// Switch 语句的默认分支
    SwitchDefault,
    /// Switch 语句的具体 case 分支，值为 case 常量
    SwitchCase(i64),

    // 非法值
    /// JumpTarget 被析构了, 不可访问
    Disposed,
}

/// 跳转目标对象，连接终结指令和目标基本块
///
/// 每个跳转目标表示控制流图中的一条边，包含：
/// - 跳转类型（无条件跳转、条件分支等）
/// - 源终结指令的引用
/// - 目标基本块的引用
///
/// 跳转目标通过弱引用链表连接，避免循环引用问题。
pub struct JumpTarget {
    node_head: Cell<EntityListHead<JumpTarget>>,
    /// 跳转目标的类型
    kind: Cell<JumpTargetKind>,
    /// 源终结指令的引用
    pub terminator: Cell<Option<InstID>>,
    /// 目标基本块的引用
    pub block: Cell<Option<BlockID>>,
}
impl IEntityRingListNode for JumpTarget {
    fn load_head(&self) -> EntityListHead<Self> {
        self.node_head.get()
    }
    fn store_head(&self, head: EntityListHead<Self>) {
        self.node_head.set(head);
    }

    fn is_sentinel(&self) -> bool {
        self.get_kind() == JumpTargetKind::None
    }
    fn new_sentinel() -> Self {
        JumpTarget {
            node_head: Cell::new(EntityListHead::none()),
            kind: Cell::new(JumpTargetKind::None),
            terminator: Cell::new(None),
            block: Cell::new(None),
        }
    }
    fn on_self_unplug(&self, _: JumpTargetID, _: &EntityAlloc<Self>) {
        self.block.set(None);
    }
}
impl JumpTarget {
    pub fn new(kind: JumpTargetKind) -> Self {
        use JumpTargetKind::*;
        assert_ne!(kind, Disposed, "Cannot allocate a disposed JumpTarget");
        JumpTarget {
            node_head: Cell::new(EntityListHead::none()),
            kind: Cell::new(kind),
            terminator: Cell::new(Option::None),
            block: Cell::new(Option::None),
        }
    }

    pub fn get_kind(&self) -> JumpTargetKind {
        self.kind.get()
    }
    pub fn is_disposed(&self) -> bool {
        self.kind.get() == JumpTargetKind::Disposed
    }
    pub(in crate::ir) fn mark_disposed(&self) {
        self.kind.set(JumpTargetKind::Disposed);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JumpTargetID(pub PtrID<JumpTarget>);

impl From<PtrID<JumpTarget>> for JumpTargetID {
    fn from(ptr: PtrID<JumpTarget>) -> Self {
        JumpTargetID(ptr)
    }
}
impl Into<PtrID<JumpTarget>> for JumpTargetID {
    fn into(self) -> PtrID<JumpTarget> {
        self.0
    }
}
impl Debug for JumpTargetID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JumpTargetID({:p})", self.inner())
    }
}
impl JumpTargetID {
    pub fn inner(self) -> PtrID<JumpTarget> {
        self.0
    }

    pub fn deref_ir(self, allocs: &IRAllocs) -> &JumpTarget {
        self.0.deref(&allocs.jts)
    }

    pub fn get_kind(self, allocs: &IRAllocs) -> JumpTargetKind {
        self.deref_ir(allocs).get_kind()
    }
    pub fn get_terminator(self, allocs: &IRAllocs) -> Option<InstID> {
        self.deref_ir(allocs).terminator.get()
    }
    pub fn set_terminator(self, allocs: &IRAllocs, inst: InstID) {
        self.deref_ir(allocs).terminator.set(Some(inst));
    }

    pub fn get_block(self, allocs: &IRAllocs) -> Option<BlockID> {
        self.deref_ir(allocs).block.get()
    }
    pub fn raw_set_block(self, allocs: &IRAllocs, block: BlockID) {
        self.deref_ir(allocs).block.set(Some(block));
    }
    pub fn set_block(self, allocs: &IRAllocs, block: BlockID) {
        let jt_obj = self.deref_ir(allocs);
        if jt_obj.block.get() == Some(block) {
            return;
        }
        jt_obj
            .detach(&allocs.jts)
            .expect("Failed to detach JumpTarget from its previous block");
        jt_obj.block.set(Some(block));
        block.deref_ir(allocs).add_pred(allocs, self);
    }
    pub fn clean_block(self, allocs: &IRAllocs) {
        let obj = self.deref_ir(allocs);
        if let None = obj.block.get() {
            return;
        }
        obj.block.set(None);
        obj.detach(&allocs.jts)
            .expect("Failed to detach JumpTarget from its block");
    }

    pub fn new(allocs: &IRAllocs, kind: JumpTargetKind) -> Self {
        JumpTarget::allocate(allocs, JumpTarget::new(kind))
    }
    pub fn dispose(self, allocs: &IRAllocs) -> PoolAllocatedDisposeRes {
        JumpTarget::dispose_id(self, allocs)
    }
}

pub type PredList = EntityRingList<JumpTarget>;
pub type JumpTargets<'ir> = MixRef<'ir, [JumpTargetID]>;

#[derive(Clone)]
pub struct JumpTargetsBlockIter<'ir> {
    jts: MixRefIter<'ir, JumpTargetID>,
    allocs: &'ir IRAllocs,
}
impl<'ir> Iterator for JumpTargetsBlockIter<'ir> {
    type Item = Option<BlockID>;

    fn next(&mut self) -> Option<Self::Item> {
        self.jts.next().map(|jt_id| jt_id.get_block(self.allocs))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.jts.size_hint()
    }
}
impl<'ir> JumpTargetsBlockIter<'ir> {
    pub fn new(jts: JumpTargets<'ir>, allocs: &'ir IRAllocs) -> Self {
        JumpTargetsBlockIter { jts: jts.into_iter(), allocs }
    }
}

pub trait ITerminatorInst: ISubInst {
    fn get_jts(&self) -> JumpTargets<'_>;
    fn jts_mut(&mut self) -> &mut [JumpTargetID];
    fn terminates_function(&self) -> bool {
        self.get_jts().is_empty()
    }

    fn n_jump_targets(&self) -> usize {
        self.get_jts().len()
    }
    fn blocks_iter<'ir>(&'ir self, allocs: &'ir IRAllocs) -> JumpTargetsBlockIter<'ir> {
        JumpTargetsBlockIter::new(self.get_jts(), allocs)
    }

    fn dedup_dump_blocks(&self, allocs: &IRAllocs, sorts: bool) -> Vec<Option<BlockID>> {
        let self_iter = self.blocks_iter(allocs);
        if sorts {
            let blocks = BTreeSet::from_iter(self_iter);
            blocks.into_iter().collect()
        } else {
            let mut blocks = HashSet::new();
            let mut has_none = false;
            for b in self_iter {
                match b {
                    Some(b) => {
                        blocks.insert(b);
                    }
                    None => {
                        has_none = true;
                    }
                }
            }
            let mut res = Vec::with_capacity(blocks.len() + if has_none { 1 } else { 0 });
            if has_none {
                res.push(None);
            }
            res.extend(blocks.into_iter().map(Some));
            res
        }
    }

    fn has_multiple_blocks(&self, allocs: &IRAllocs) -> bool {
        let mut first = None;
        for b in self.blocks_iter(allocs) {
            match first {
                Some(f) if f != b => return true,
                None => first = Some(b),
                _ => continue,
            }
        }
        false
    }
}

pub trait ITerminatorID: ISubInstID<InstObjT: ITerminatorInst> {
    fn get_jts(self, allocs: &IRAllocs) -> JumpTargets<'_> {
        self.deref_ir(allocs).get_jts()
    }
    fn jts_mut(self, allocs: &mut IRAllocs) -> &mut [JumpTargetID] {
        self.deref_ir_mut(allocs).jts_mut()
    }
    fn blocks_iter(self, allocs: &IRAllocs) -> JumpTargetsBlockIter<'_> {
        self.deref_ir(allocs).blocks_iter(allocs)
    }
    fn dedup_dump_blocks(self, allocs: &IRAllocs, sorts: bool) -> Vec<Option<BlockID>> {
        self.deref_ir(allocs).dedup_dump_blocks(allocs, sorts)
    }
    fn has_multiple_blocks(self, allocs: &IRAllocs) -> bool {
        self.deref_ir(allocs).has_multiple_blocks(allocs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TerminatorID {
    Unreachable(UnreachableInstID),
    Ret(RetInstID),
    Jump(JumpInstID),
    Br(BrInstID),
    Switch(SwitchInstID),
}
#[derive(Clone, Copy)]
pub enum TerminatorObj<'ir> {
    Unreachable(&'ir UnreachableInst),
    Ret(&'ir RetInst),
    Jump(&'ir JumpInst),
    Br(&'ir BrInst),
    Switch(&'ir SwitchInst),
}
impl TerminatorID {
    pub fn try_from_ir(allocs: &IRAllocs, inst_id: impl ISubInstID) -> Option<Self> {
        use TerminatorID::*;
        let inst_id = inst_id.into_instid();
        match inst_id.deref_ir(allocs) {
            InstObj::Unreachable(_) => Some(Unreachable(UnreachableInstID(inst_id))),
            InstObj::Ret(_) => Some(Ret(RetInstID(inst_id))),
            InstObj::Jump(_) => Some(Jump(JumpInstID(inst_id))),
            InstObj::Br(_) => Some(Br(BrInstID(inst_id))),
            InstObj::Switch(_) => Some(Switch(SwitchInstID(inst_id))),
            _ => None,
        }
    }
    pub fn into_ir(self) -> InstID {
        use TerminatorID::*;
        match self {
            Unreachable(id) => id.into_instid(),
            Ret(id) => id.into_instid(),
            Jump(id) => id.into_instid(),
            Br(id) => id.into_instid(),
            Switch(id) => id.into_instid(),
        }
    }

    pub fn deref_ir(self, allocs: &IRAllocs) -> TerminatorObj<'_> {
        use TerminatorID::*;
        match self {
            Unreachable(id) => TerminatorObj::Unreachable(id.deref_ir(allocs)),
            Ret(id) => TerminatorObj::Ret(id.deref_ir(allocs)),
            Jump(id) => TerminatorObj::Jump(id.deref_ir(allocs)),
            Br(id) => TerminatorObj::Br(id.deref_ir(allocs)),
            Switch(id) => TerminatorObj::Switch(id.deref_ir(allocs)),
        }
    }

    pub fn get_jts(self, allocs: &IRAllocs) -> JumpTargets<'_> {
        match self.deref_ir(allocs) {
            TerminatorObj::Unreachable(inst) => inst.get_jts(),
            TerminatorObj::Ret(inst) => inst.get_jts(),
            TerminatorObj::Jump(inst) => inst.get_jts(),
            TerminatorObj::Br(inst) => inst.get_jts(),
            TerminatorObj::Switch(inst) => inst.get_jts(),
        }
    }
    pub fn blocks_iter(self, allocs: &IRAllocs) -> JumpTargetsBlockIter<'_> {
        JumpTargetsBlockIter::new(self.get_jts(allocs), allocs)
    }
}
