use super::managing::{
    dispose_entity_list, global_common_dispose, inst_dispose, traceable_dispose, traceable_init_id,
    user_dispose, user_init_id,
};
use crate::ir::{
    BlockID, BlockObj, ExprID, ExprObj, FuncID, GlobalID, GlobalObj, ISubExpr, ISubExprID,
    ISubGlobal, ISubGlobalID, ISubInst, ISubInstID, InstID, InstObj, JumpTarget, JumpTargetID,
    JumpTargetKind, Module, Use, UseID, UseKind, UserID, UserList, ValueSSA,
    block::BlockAlloc,
    constant::expr::ExprAlloc,
    global::GlobalAlloc,
    inst::{InstAlloc, InstBackID},
    jumping::JumpTargetAlloc,
    module::managing::dispose_order_list,
    usedef::UseAlloc,
};
use mtb_entity_slab::{
    EntityAlloc, IAllocPolicy, IBasicEntityListID, IEntityAllocID, IEntityRingListNodeID,
    IPoliciedID, PtrID,
};
use std::{cell::RefCell, collections::VecDeque};
use thiserror::Error;

pub struct IRAllocs {
    pub exprs: ExprAlloc,
    pub insts: InstAlloc,
    pub globals: GlobalAlloc,
    pub blocks: BlockAlloc,
    pub uses: UseAlloc,
    pub jts: JumpTargetAlloc,
    pub disposed_queue: RefCell<VecDeque<PoolAllocatedID>>,
}

impl AsRef<IRAllocs> for IRAllocs {
    fn as_ref(&self) -> &IRAllocs {
        self
    }
}
impl AsMut<IRAllocs> for IRAllocs {
    fn as_mut(&mut self) -> &mut IRAllocs {
        self
    }
}

impl Default for IRAllocs {
    fn default() -> Self {
        Self::new()
    }
}

impl IRAllocs {
    pub fn new() -> Self {
        Self {
            exprs: EntityAlloc::new(),
            insts: EntityAlloc::new(),
            globals: EntityAlloc::new(),
            blocks: EntityAlloc::new(),
            uses: EntityAlloc::new(),
            jts: EntityAlloc::new(),
            disposed_queue: RefCell::new(VecDeque::new()),
        }
    }

    pub fn with_capacity(base_cap: usize) -> Self {
        Self {
            exprs: EntityAlloc::with_capacity(base_cap * 4),
            insts: EntityAlloc::with_capacity(base_cap * 4),
            globals: EntityAlloc::with_capacity(base_cap),
            blocks: EntityAlloc::with_capacity(base_cap * 2),
            uses: EntityAlloc::with_capacity(base_cap * 12),
            jts: EntityAlloc::with_capacity(base_cap * 2),
            disposed_queue: RefCell::new(VecDeque::with_capacity(base_cap)),
        }
    }

    pub fn num_total_allocated(&self) -> usize {
        let b = self.blocks.len();
        let i = self.insts.len();
        let e = self.exprs.len();
        let g = self.globals.len();
        let u = self.uses.len();
        let j = self.jts.len();
        b + i + e + g + u + j
    }

    pub(crate) fn push_disposed(&self, id: impl Into<PoolAllocatedID>) {
        self.disposed_queue.borrow_mut().push_back(id.into());
    }
    pub fn free_disposed(&mut self) {
        let Self { exprs, insts, globals, blocks, uses, jts, disposed_queue } = self;
        let queue = disposed_queue.get_mut();
        while let Some(id) = queue.pop_front() {
            use PoolAllocatedID::*;
            match id {
                Block(b) => {
                    b.inner().free(blocks);
                }
                Inst(i) => {
                    i.into_raw_ptr().free(insts);
                }
                Expr(e) => {
                    e.into_raw_ptr().free(exprs);
                }
                Global(g) => {
                    g.into_raw_ptr().free(globals);
                }
                Use(u) => {
                    u.inner().free(uses);
                }
                JumpTarget(j) => {
                    j.inner().free(jts);
                }
            }
        }
        // After draining, optionally shrink the dispose queue's capacity so it doesn't keep
        // an excessive allocation. Heuristic:
        // - Soft target scales with module size (total allocated entities / 8),
        //   clamped within [QUEUE_SOFT_TARGET_MIN, QUEUE_SOFT_TARGET_MAX].
        // - Hard cap bounds the retained capacity unconditionally.
        // - Shrink only when capacity is far above the target or hard cap to reduce churn.
        let num_total_allocated = {
            let b = blocks.len();
            let i = insts.len();
            let e = exprs.len();
            let g = globals.len();
            let u = uses.len();
            let j = jts.len();
            b + i + e + g + u + j
        };
        const QUEUE_HARD_CAP: usize = 16 * 1024; // absolute upper bound to retain
        const QUEUE_SOFT_TARGET_MIN: usize = 256; // never shrink below this
        const QUEUE_SOFT_TARGET_MAX: usize = 8 * 1024; // typical soft ceiling

        let mut target = num_total_allocated / 8; // scale with module size
        target = target.clamp(QUEUE_SOFT_TARGET_MIN, QUEUE_SOFT_TARGET_MAX);
        let cap = queue.capacity();
        if cap > QUEUE_HARD_CAP || cap > (target.saturating_mul(2)) {
            let new_cap = target.min(QUEUE_HARD_CAP);
            *queue = VecDeque::with_capacity(new_cap);
        }
    }

    pub fn num_pending_disposed(&self) -> usize {
        self.disposed_queue.borrow().len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PoolAllocatedClass {
    Block,
    Inst,
    Expr,
    Global,
    Use,
    JumpTarget,
}

#[derive(Debug, Clone, Copy, Error)]
pub enum PoolAllocatedDisposeErr {
    #[error("Entity already disposed")]
    AlreadyDisposed,

    #[error("Disposing a global in reading symbol table: {0}")]
    SymtabBorrowError(&'static std::panic::Location<'static>),
}
pub type PoolAllocatedDisposeRes<T = ()> = Result<T, PoolAllocatedDisposeErr>;

pub(crate) trait IPoolAllocated: Sized {
    type PolicyT: IAllocPolicy;
    type PtrID: IPoliciedID<ObjectT = Self, PolicyT = Self::PolicyT> + Into<PoolAllocatedID>;
    type MinRelatedPoolT: AsRef<IRAllocs>;

    const _CLASS: PoolAllocatedClass;

    fn get_alloc(allocs: &IRAllocs) -> &EntityAlloc<Self, Self::PolicyT>;
    fn _alloc_mut(allocs: &mut IRAllocs) -> &mut EntityAlloc<Self, Self::PolicyT>;

    fn init_self_id(&self, id: Self::PtrID, allocs: &IRAllocs);
    fn allocate(allocs: &IRAllocs, obj: Self) -> Self::PtrID;

    fn obj_disposed(&self) -> bool;
    fn id_is_live(id: Self::PtrID, allocs: &IRAllocs) -> bool {
        let alloc = Self::get_alloc(allocs);
        let ptr = id.into_backend();
        let Some(obj) = ptr.try_deref(alloc) else {
            return false;
        };
        !obj.obj_disposed()
    }

    fn dispose_obj(&self, id: Self::PtrID, pool: &Self::MinRelatedPoolT)
    -> PoolAllocatedDisposeRes;
    fn dispose_id(id: Self::PtrID, pool: &Self::MinRelatedPoolT) -> PoolAllocatedDisposeRes {
        let alloc = Self::get_alloc(pool.as_ref());
        let ptr = id.into_backend();
        let Some(obj) = ptr.try_deref(alloc) else {
            return Err(PoolAllocatedDisposeErr::AlreadyDisposed);
        };
        obj.dispose_obj(id, pool)?;
        pool.as_ref().push_disposed(id);
        Ok(())
    }
}

impl IPoolAllocated for BlockObj {
    type PtrID = BlockID;
    type PolicyT = <BlockID as IPoliciedID>::PolicyT;
    type MinRelatedPoolT = IRAllocs;

    const _CLASS: PoolAllocatedClass = PoolAllocatedClass::Block;

    fn get_alloc(ir_allocs: &IRAllocs) -> &EntityAlloc<Self> {
        &ir_allocs.blocks
    }
    fn _alloc_mut(ir_allocs: &mut IRAllocs) -> &mut EntityAlloc<Self> {
        &mut ir_allocs.blocks
    }

    fn allocate(allocs: &IRAllocs, obj: Self) -> BlockID {
        let alloc = &allocs.blocks;
        let ptr = PtrID::allocate_from(alloc, obj);
        ptr.deref(alloc).init_self_id(BlockID(ptr), allocs);
        BlockID(ptr)
    }
    fn init_self_id(&self, id: BlockID, allocs: &IRAllocs) {
        traceable_init_id(self, ValueSSA::Block(id), allocs);
        let Some(body) = &self.body else {
            return;
        };
        body.preds.sentinel.raw_set_block(allocs, id);
        body.insts
            .forall_with_sentinel(&allocs.insts, |_, i| {
                i.set_parent(Some(id));
                Ok(())
            })
            .unwrap();
    }

    fn obj_disposed(&self) -> bool {
        self.dispose_mark.get()
    }
    fn dispose_obj(&self, id: BlockID, allocs: &IRAllocs) -> PoolAllocatedDisposeRes {
        if self.obj_disposed() {
            return Err(PoolAllocatedDisposeErr::AlreadyDisposed);
        }
        self.dispose_mark.set(true);

        if let Some(parent) = self.get_parent_func()
            && let Some(bbs) = parent.get_body(allocs)
        {
            bbs.blocks
                .node_unplug(id, &allocs.blocks)
                .expect("Block not found in parent function's block list");
        }
        let Some(body) = &self.body else {
            return Ok(());
        };
        dispose_order_list::<InstObj>(&body.insts, allocs)?;
        traceable_dispose(self, allocs)?;

        // clean up predecessors
        body.preds.clean(&allocs.jts);
        body.preds.sentinel.dispose(allocs)?;
        Ok(())
    }
}

impl IPoolAllocated for InstObj {
    type PtrID = InstID;
    type PolicyT = <InstID as IPoliciedID>::PolicyT;
    type MinRelatedPoolT = IRAllocs;

    const _CLASS: PoolAllocatedClass = PoolAllocatedClass::Inst;

    fn get_alloc(ir_allocs: &IRAllocs) -> &EntityAlloc<Self, Self::PolicyT> {
        &ir_allocs.insts
    }
    fn _alloc_mut(ir_allocs: &mut IRAllocs) -> &mut EntityAlloc<Self, Self::PolicyT> {
        &mut ir_allocs.insts
    }

    fn init_self_id(&self, id: InstID, allocs: &IRAllocs) {
        user_init_id(self, id.into(), allocs);
        if let Some(jt) = self.try_get_jts() {
            for &jt_id in jt.iter() {
                jt_id.set_terminator(allocs, id);
            }
        }
        use InstObj::*;
        // Intentionally keep an exhaustive list of variants whose init needs no extra work.
        // Avoiding a wildcard arm ensures compile errors when new variants are added,
        // so we must consciously review whether they require special wiring here.
        match self {
            GuideNode(_) | PhiInstEnd(_) | Unreachable(_) | Ret(_) | Jump(_) | Br(_)
            | Alloca(_) | GEP(_) | Load(_) | Store(_) | AmoRmw(_) | BinOP(_) | Call(_)
            | Cast(_) | Cmp(_) | IndexExtract(_) | FieldExtract(_) | IndexInsert(_)
            | FieldInsert(_) | Select(_) => { /* do nothing */ }
            Switch(_) => { /* do nothing */ }
            Phi(phi) => phi.self_id.set(Some(id)),
        }
    }
    fn allocate(allocs: &IRAllocs, mut obj: Self) -> InstID {
        if !InstID::obj_is_sentinel(&obj) && obj.common_mut().users.is_none() {
            obj.common_mut().users = Some(UserList::new(&allocs.uses));
        }
        let alloc = &allocs.insts;
        let ptr = InstBackID::allocate_from(alloc, obj);
        ptr.deref(alloc).init_self_id(InstID(ptr), allocs);
        InstID(ptr)
    }

    fn obj_disposed(&self) -> bool {
        self.get_common().disposed.get()
    }
    fn dispose_obj(&self, id: InstID, allocs: &IRAllocs) -> PoolAllocatedDisposeRes {
        inst_dispose(self, id, allocs)?;
        use InstObj::*;
        // Same rationale as in init_self_id: list all variants explicitly to stay exhaustive
        // and require review on new variants.
        match self {
            GuideNode(_) | PhiInstEnd(_) | Unreachable(_) | Ret(_) | Jump(_) | Br(_)
            | Alloca(_) | GEP(_) | Load(_) | Store(_) | AmoRmw(_) | BinOP(_) | Call(_)
            | Cast(_) | Cmp(_) | IndexExtract(_) | FieldExtract(_) | IndexInsert(_)
            | FieldInsert(_) | Select(_) => { /* do nothing */ }
            Switch(_) => { /* do nothing */ }
            Phi(phi) => phi.self_id.set(None),
        }
        Ok(())
    }
}

impl IPoolAllocated for ExprObj {
    type PtrID = ExprID;
    type PolicyT = <ExprID as IPoliciedID>::PolicyT;
    type MinRelatedPoolT = IRAllocs;

    const _CLASS: PoolAllocatedClass = PoolAllocatedClass::Expr;

    fn get_alloc(ir_allocs: &IRAllocs) -> &EntityAlloc<Self> {
        &ir_allocs.exprs
    }
    fn _alloc_mut(ir_allocs: &mut IRAllocs) -> &mut EntityAlloc<Self> {
        &mut ir_allocs.exprs
    }

    fn init_self_id(&self, id: ExprID, allocs: &IRAllocs) {
        user_init_id(self, id.into(), allocs);
    }
    fn allocate(allocs: &IRAllocs, mut obj: Self) -> ExprID {
        if obj.common_mut().users.is_none() {
            obj.common_mut().users = Some(UserList::new(&allocs.uses));
        }
        let alloc = &allocs.exprs;
        let ptr = PtrID::allocate_from(alloc, obj);
        ptr.deref(alloc).init_self_id(ExprID(ptr), allocs);
        ExprID(ptr)
    }
    fn obj_disposed(&self) -> bool {
        self.get_common().dispose_mark.get()
    }

    fn dispose_obj(&self, _: ExprID, allocs: &IRAllocs) -> PoolAllocatedDisposeRes {
        if self.obj_disposed() {
            return Err(PoolAllocatedDisposeErr::AlreadyDisposed);
        }
        self.get_common().dispose_mark.set(true);
        user_dispose(self, allocs)
    }
}

impl IPoolAllocated for GlobalObj {
    type PtrID = GlobalID;
    type PolicyT = <GlobalID as IPoliciedID>::PolicyT;
    type MinRelatedPoolT = Module;

    const _CLASS: PoolAllocatedClass = PoolAllocatedClass::Global;

    fn get_alloc(ir_allocs: &IRAllocs) -> &GlobalAlloc {
        &ir_allocs.globals
    }
    fn _alloc_mut(ir_allocs: &mut IRAllocs) -> &mut GlobalAlloc {
        &mut ir_allocs.globals
    }

    fn init_self_id(&self, id: GlobalID, allocs: &IRAllocs) {
        user_init_id(self, UserID::Global(id), allocs);
        let f = match self {
            GlobalObj::Var(_) => return,
            GlobalObj::Func(f) => f,
        };
        let func_id = FuncID::raw_from(id);
        for arg in &f.args {
            arg.func.set(Some(func_id));
            let arg_val = ValueSSA::FuncArg(func_id, arg.index);
            traceable_init_id(arg, arg_val, allocs);
        }
        let Some(body) = &f.body else {
            return;
        };
        body.blocks
            .forall_with_sentinel(&allocs.blocks, |_, b| {
                b.set_parent_func(func_id);
                Ok(())
            })
            .unwrap();
    }
    fn allocate(allocs: &IRAllocs, mut obj: Self) -> GlobalID {
        if obj.common_mut().users.is_none() {
            obj.common_mut().users = Some(UserList::new(&allocs.uses));
        }
        let alloc = &allocs.globals;
        let ptr = PtrID::allocate_from(alloc, obj);
        ptr.deref(alloc).init_self_id(GlobalID(ptr), allocs);
        GlobalID(ptr)
    }

    fn obj_disposed(&self) -> bool {
        self.get_common().dispose_mark.get()
    }
    fn dispose_obj(&self, id: GlobalID, pool: &Module) -> PoolAllocatedDisposeRes {
        global_common_dispose(self, id, pool)?;
        let func = match self {
            GlobalObj::Var(_) => return Ok(()),
            GlobalObj::Func(gf) => gf,
        };
        for arg in &func.args {
            arg.func.set(None);
            traceable_dispose(arg, &pool.allocs)?;
        }
        let Some(body) = &func.body else {
            return Ok(());
        };
        dispose_entity_list::<BlockObj>(&body.blocks, &pool.allocs)
    }
}

impl IPoolAllocated for Use {
    type PtrID = UseID;
    type PolicyT = <UseID as IPoliciedID>::PolicyT;
    type MinRelatedPoolT = IRAllocs;

    const _CLASS: PoolAllocatedClass = PoolAllocatedClass::Use;

    fn get_alloc(ir_allocs: &IRAllocs) -> &UseAlloc {
        &ir_allocs.uses
    }
    fn _alloc_mut(ir_allocs: &mut IRAllocs) -> &mut UseAlloc {
        &mut ir_allocs.uses
    }

    fn init_self_id(&self, _: UseID, _: &IRAllocs) {}
    fn allocate(allocs: &IRAllocs, obj: Self) -> UseID {
        assert_ne!(
            obj.get_kind(),
            UseKind::DisposedUse,
            "Cannot allocate a disposed Use"
        );
        let ptr = PtrID::allocate_from(&allocs.uses, obj);
        ptr.deref(&allocs.uses).init_self_id(UseID(ptr), allocs);
        UseID(ptr)
    }

    fn obj_disposed(&self) -> bool {
        self.get_kind() == UseKind::DisposedUse
    }
    fn dispose_obj(&self, uid: UseID, allocs: &IRAllocs) -> PoolAllocatedDisposeRes {
        if self.obj_disposed() {
            return Err(PoolAllocatedDisposeErr::AlreadyDisposed);
        }
        self.mark_disposed();
        uid.detach(&allocs.uses).expect("Use dispose detach failed");
        self.user.set(None);
        self.operand.set(ValueSSA::None);
        Ok(())
    }
}

impl IPoolAllocated for JumpTarget {
    type PtrID = JumpTargetID;
    type PolicyT = <JumpTargetID as IPoliciedID>::PolicyT;
    type MinRelatedPoolT = IRAllocs;

    const _CLASS: PoolAllocatedClass = PoolAllocatedClass::JumpTarget;

    fn get_alloc(ir_allocs: &IRAllocs) -> &JumpTargetAlloc {
        &ir_allocs.jts
    }
    fn _alloc_mut(ir_allocs: &mut IRAllocs) -> &mut JumpTargetAlloc {
        &mut ir_allocs.jts
    }

    fn init_self_id(&self, _: JumpTargetID, _: &IRAllocs) {}
    fn allocate(allocs: &IRAllocs, obj: Self) -> JumpTargetID {
        let ptr = PtrID::allocate_from(&allocs.jts, obj);
        ptr.deref(&allocs.jts)
            .init_self_id(JumpTargetID(ptr), allocs);
        JumpTargetID(ptr)
    }
    fn obj_disposed(&self) -> bool {
        self.get_kind() == JumpTargetKind::Disposed
    }
    fn dispose_obj(&self, jtid: JumpTargetID, allocs: &IRAllocs) -> PoolAllocatedDisposeRes {
        if self.obj_disposed() {
            return Err(PoolAllocatedDisposeErr::AlreadyDisposed);
        }
        self.mark_disposed();
        jtid.detach(&allocs.jts)
            .expect("JumpTarget dispose detach failed");
        self.terminator.set(None);
        self.block.set(None);
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PoolAllocatedID {
    Block(BlockID),
    Inst(InstID),
    Expr(ExprID),
    Global(GlobalID),
    Use(UseID),
    JumpTarget(JumpTargetID),
}
impl std::fmt::Debug for PoolAllocatedID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PoolAllocatedID::*;
        match self {
            Block(b) => write!(f, "BlockID({:p})", b.inner()),
            Inst(i) => write!(f, "InstID({:p})", i.into_raw_ptr()),
            Expr(e) => write!(f, "ExprID({:p})", e.into_raw_ptr()),
            Global(g) => write!(f, "GlobalID({:p})", g.into_raw_ptr()),
            Use(u) => write!(f, "UseID({:p})", u.0),
            JumpTarget(j) => write!(f, "JumpTargetID({:p})", j.0),
        }
    }
}
impl From<BlockID> for PoolAllocatedID {
    fn from(id: BlockID) -> Self {
        PoolAllocatedID::Block(id)
    }
}
impl From<InstID> for PoolAllocatedID {
    fn from(id: InstID) -> Self {
        PoolAllocatedID::Inst(id)
    }
}
impl From<ExprID> for PoolAllocatedID {
    fn from(id: ExprID) -> Self {
        PoolAllocatedID::Expr(id)
    }
}
impl From<GlobalID> for PoolAllocatedID {
    fn from(id: GlobalID) -> Self {
        PoolAllocatedID::Global(id)
    }
}
impl From<UseID> for PoolAllocatedID {
    fn from(id: UseID) -> Self {
        PoolAllocatedID::Use(id)
    }
}
impl From<JumpTargetID> for PoolAllocatedID {
    fn from(id: JumpTargetID) -> Self {
        PoolAllocatedID::JumpTarget(id)
    }
}
impl PoolAllocatedID {
    pub fn get_class(&self) -> PoolAllocatedClass {
        match self {
            PoolAllocatedID::Block(_) => PoolAllocatedClass::Block,
            PoolAllocatedID::Inst(_) => PoolAllocatedClass::Inst,
            PoolAllocatedID::Expr(_) => PoolAllocatedClass::Expr,
            PoolAllocatedID::Global(_) => PoolAllocatedClass::Global,
            PoolAllocatedID::Use(_) => PoolAllocatedClass::Use,
            PoolAllocatedID::JumpTarget(_) => PoolAllocatedClass::JumpTarget,
        }
    }
    pub fn dispose(self, module: &Module) -> PoolAllocatedDisposeRes {
        match self {
            PoolAllocatedID::Block(b) => BlockObj::dispose_id(b, &module.allocs),
            PoolAllocatedID::Inst(i) => InstObj::dispose_id(i, &module.allocs),
            PoolAllocatedID::Expr(e) => ExprObj::dispose_id(e, &module.allocs),
            PoolAllocatedID::Global(g) => GlobalObj::dispose_id(g, module),
            PoolAllocatedID::Use(u) => Use::dispose_id(u, &module.allocs),
            PoolAllocatedID::JumpTarget(j) => JumpTarget::dispose_id(j, &module.allocs),
        }
    }
    pub fn get_entity_index(self, allocs: &IRAllocs) -> usize {
        use PoolAllocatedID::*;
        match self {
            Block(b) => b.get_entity_index(allocs),
            Inst(i) => i.get_entity_index(allocs),
            Expr(e) => e.get_entity_index(allocs),
            Global(g) => g.get_entity_index(allocs),
            Use(u) => u.get_entity_index(allocs),
            JumpTarget(j) => j.get_entity_index(allocs),
        }
    }
    pub fn try_get_entity_index(self, allocs: &IRAllocs) -> Option<usize> {
        use PoolAllocatedID::*;
        match self {
            Block(b) => b.try_get_entity_index(allocs),
            Inst(i) => i.try_get_entity_index(allocs),
            Expr(e) => e.try_get_entity_index(allocs),
            Global(g) => g.try_get_entity_index(allocs),
            Use(u) => u.try_get_entity_index(allocs),
            JumpTarget(j) => j.try_get_entity_index(allocs),
        }
    }
}
