use crate::ir::{global::GlobalDisposeError, *};
use mtb_entity::{
    EntityAlloc, EntityAllocPolicy128, EntityAllocPolicy256, EntityAllocPolicy512,
    EntityAllocPolicy4096, IEntityAllocID, IEntityAllocatable, PtrID,
};
use std::{cell::RefCell, collections::VecDeque};

pub struct IRAllocs {
    pub exprs: EntityAlloc<ExprObj>,
    pub insts: EntityAlloc<InstObj>,
    pub globals: EntityAlloc<GlobalObj>,
    pub blocks: EntityAlloc<BlockObj>,
    pub uses: EntityAlloc<Use>,
    pub jts: EntityAlloc<JumpTarget>,
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
                    i.free(insts);
                }
                Expr(e) => {
                    e.free(exprs);
                }
                Global(g) => {
                    g.free(globals);
                }
                Use(u) => {
                    u.inner().free(uses);
                }
                JumpTarget(j) => {
                    j.inner().free(jts);
                }
            }
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

pub trait IPoolAllocated: IEntityAllocatable {
    type ModuleID: Copy;
    type MinRelatedPoolT: AsRef<IRAllocs>;

    const CLASS: PoolAllocatedClass;

    fn get_alloc(ir_allocs: &IRAllocs) -> &EntityAlloc<Self>;
    fn alloc_mut(ir_allocs: &mut IRAllocs) -> &mut EntityAlloc<Self>;

    fn make_module_id(raw: PtrID<Self>) -> Self::ModuleID;
    fn from_module_id(id: Self::ModuleID) -> PtrID<Self>;

    fn dispose_id(id: Self::ModuleID, pool: &Self::MinRelatedPoolT);
    fn obj_disposed(obj: &Self) -> bool;

    fn id_disposed(id: Self::ModuleID, ir_allocs: &IRAllocs) -> bool {
        let alloc = Self::get_alloc(ir_allocs);
        let Some(obj) = Self::from_module_id(id).try_deref(alloc) else {
            return true;
        };
        Self::obj_disposed(obj)
    }
}

impl IEntityAllocatable for BlockObj {
    /// Allocate 256 entities per allocation block.
    type AllocatePolicyT = EntityAllocPolicy256<Self>;
}
impl IPoolAllocated for BlockObj {
    type ModuleID = BlockID;
    type MinRelatedPoolT = IRAllocs;

    const CLASS: PoolAllocatedClass = PoolAllocatedClass::Block;

    fn get_alloc(ir_allocs: &IRAllocs) -> &EntityAlloc<Self> {
        &ir_allocs.blocks
    }
    fn alloc_mut(ir_allocs: &mut IRAllocs) -> &mut EntityAlloc<Self> {
        &mut ir_allocs.blocks
    }

    fn make_module_id(raw: PtrID<Self>) -> Self::ModuleID {
        BlockID(raw)
    }
    fn from_module_id(id: Self::ModuleID) -> PtrID<Self> {
        id.0
    }
    fn dispose_id(id: Self::ModuleID, ir_allocs: &IRAllocs) {
        id.dispose(ir_allocs);
    }

    fn obj_disposed(obj: &Self) -> bool {
        obj.is_disposed()
    }
}

impl IEntityAllocatable for InstObj {
    /// Allocate 512 entities per allocation block.
    type AllocatePolicyT = EntityAllocPolicy512<Self>;
}
impl IPoolAllocated for InstObj {
    type ModuleID = InstID;
    type MinRelatedPoolT = IRAllocs;

    const CLASS: PoolAllocatedClass = PoolAllocatedClass::Inst;

    fn get_alloc(ir_allocs: &IRAllocs) -> &EntityAlloc<Self> {
        &ir_allocs.insts
    }
    fn alloc_mut(ir_allocs: &mut IRAllocs) -> &mut EntityAlloc<Self> {
        &mut ir_allocs.insts
    }

    fn make_module_id(raw: PtrID<Self>) -> Self::ModuleID {
        raw
    }
    fn from_module_id(id: Self::ModuleID) -> PtrID<Self> {
        id
    }
    fn dispose_id(id: Self::ModuleID, ir_allocs: &IRAllocs) {
        id.dispose(ir_allocs);
    }
    fn obj_disposed(obj: &Self) -> bool {
        obj.is_disposed()
    }
}

impl IEntityAllocatable for ExprObj {
    /// Allocate 256 entities per allocation block.
    type AllocatePolicyT = EntityAllocPolicy256<Self>;
}
impl IPoolAllocated for ExprObj {
    type ModuleID = ExprID;
    type MinRelatedPoolT = IRAllocs;

    const CLASS: PoolAllocatedClass = PoolAllocatedClass::Expr;

    fn get_alloc(ir_allocs: &IRAllocs) -> &EntityAlloc<Self> {
        &ir_allocs.exprs
    }
    fn alloc_mut(ir_allocs: &mut IRAllocs) -> &mut EntityAlloc<Self> {
        &mut ir_allocs.exprs
    }

    fn make_module_id(raw: PtrID<Self>) -> Self::ModuleID {
        raw
    }
    fn from_module_id(id: Self::ModuleID) -> PtrID<Self> {
        id
    }
    fn dispose_id(id: Self::ModuleID, ir_allocs: &IRAllocs) {
        id.dispose(ir_allocs);
    }
    fn obj_disposed(obj: &Self) -> bool {
        obj.is_disposed()
    }
}

impl IEntityAllocatable for GlobalObj {
    /// Allocate 128 entities per allocation block.
    type AllocatePolicyT = EntityAllocPolicy128<Self>;
}
impl IPoolAllocated for GlobalObj {
    type ModuleID = GlobalID;
    type MinRelatedPoolT = Module;

    const CLASS: PoolAllocatedClass = PoolAllocatedClass::Global;

    fn get_alloc(ir_allocs: &IRAllocs) -> &EntityAlloc<Self> {
        &ir_allocs.globals
    }
    fn alloc_mut(ir_allocs: &mut IRAllocs) -> &mut EntityAlloc<Self> {
        &mut ir_allocs.globals
    }

    fn make_module_id(raw: PtrID<Self>) -> Self::ModuleID {
        raw
    }
    fn from_module_id(id: Self::ModuleID) -> PtrID<Self> {
        id
    }
    fn dispose_id(id: Self::ModuleID, module: &Module) {
        match id.dispose(module) {
            Ok(()) => (),
            Err(GlobalDisposeError::AlreadyDisposed(_)) => {
                log::warn!("Double disposal detected for GlobalID {id:?} during module disposal.");
                // then do nothing
            }
            Err(e) => {
                panic!("Error during disposal of GlobalID {id:?}: {e:?}");
            }
        }
    }
    fn obj_disposed(obj: &Self) -> bool {
        obj.is_disposed()
    }
}

impl IEntityAllocatable for Use {
    /// Allocate 4096 entities per allocation block.
    type AllocatePolicyT = EntityAllocPolicy4096<Self>;
}
impl IPoolAllocated for Use {
    type ModuleID = UseID;
    type MinRelatedPoolT = IRAllocs;

    const CLASS: PoolAllocatedClass = PoolAllocatedClass::Use;

    fn get_alloc(ir_allocs: &IRAllocs) -> &EntityAlloc<Self> {
        &ir_allocs.uses
    }
    fn alloc_mut(ir_allocs: &mut IRAllocs) -> &mut EntityAlloc<Self> {
        &mut ir_allocs.uses
    }

    fn make_module_id(raw: PtrID<Self>) -> Self::ModuleID {
        UseID(raw)
    }
    fn from_module_id(id: Self::ModuleID) -> PtrID<Self> {
        id.0
    }
    fn dispose_id(id: Self::ModuleID, allocs: &IRAllocs) {
        id.dispose(allocs);
    }
    fn obj_disposed(obj: &Self) -> bool {
        obj.is_disposed()
    }
}

impl IEntityAllocatable for JumpTarget {
    /// Allocate 256 entities per allocation block.
    type AllocatePolicyT = EntityAllocPolicy256<Self>;
}
impl IPoolAllocated for JumpTarget {
    type ModuleID = JumpTargetID;
    type MinRelatedPoolT = IRAllocs;

    const CLASS: PoolAllocatedClass = PoolAllocatedClass::JumpTarget;

    fn get_alloc(ir_allocs: &IRAllocs) -> &EntityAlloc<Self> {
        &ir_allocs.jts
    }
    fn alloc_mut(ir_allocs: &mut IRAllocs) -> &mut EntityAlloc<Self> {
        &mut ir_allocs.jts
    }

    fn make_module_id(raw: PtrID<Self>) -> Self::ModuleID {
        JumpTargetID(raw)
    }
    fn from_module_id(id: Self::ModuleID) -> PtrID<Self> {
        id.0
    }
    fn dispose_id(id: Self::ModuleID, ir_allocs: &IRAllocs) {
        id.dispose(ir_allocs);
    }
    fn obj_disposed(obj: &Self) -> bool {
        obj.is_disposed()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PoolAllocatedID {
    Block(BlockID),
    Inst(InstID),
    Expr(ExprID),
    Global(GlobalID),
    Use(UseID),
    JumpTarget(JumpTargetID),
}
impl From<BlockID> for PoolAllocatedID {
    fn from(id: BlockID) -> Self {
        PoolAllocatedID::Block(id)
    }
}
impl From<PtrID<BlockObj>> for PoolAllocatedID {
    fn from(id: PtrID<BlockObj>) -> Self {
        PoolAllocatedID::Block(BlockID(id))
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
impl From<PtrID<Use>> for PoolAllocatedID {
    fn from(id: PtrID<Use>) -> Self {
        PoolAllocatedID::Use(UseID(id))
    }
}
impl From<JumpTargetID> for PoolAllocatedID {
    fn from(id: JumpTargetID) -> Self {
        PoolAllocatedID::JumpTarget(id)
    }
}
impl From<PtrID<JumpTarget>> for PoolAllocatedID {
    fn from(id: PtrID<JumpTarget>) -> Self {
        PoolAllocatedID::JumpTarget(JumpTargetID(id))
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
    pub fn dispose(self, module: &Module) {
        match self {
            PoolAllocatedID::Block(b) => {
                BlockObj::dispose_id(b, &module.allocs);
            }
            PoolAllocatedID::Inst(i) => {
                InstObj::dispose_id(i, &module.allocs);
            }
            PoolAllocatedID::Expr(e) => {
                ExprObj::dispose_id(e, &module.allocs);
            }
            PoolAllocatedID::Global(g) => {
                GlobalObj::dispose_id(g, module);
            }
            PoolAllocatedID::Use(u) => {
                Use::dispose_id(u, &module.allocs);
            }
            PoolAllocatedID::JumpTarget(j) => {
                JumpTarget::dispose_id(j, &module.allocs);
            }
        }
    }
    pub fn get_indexed(self, ir_allocs: &IRAllocs) -> Option<usize> {
        use PoolAllocatedID::*;
        match self {
            Block(b) => b.inner().as_indexed(&ir_allocs.blocks).map(|x| x.0),
            Inst(i) => i.as_indexed(&ir_allocs.insts).map(|x| x.0),
            Expr(e) => e.as_indexed(&ir_allocs.exprs).map(|x| x.0),
            Global(g) => g.as_indexed(&ir_allocs.globals).map(|x| x.0),
            Use(u) => u.0.as_indexed(&ir_allocs.uses).map(|x| x.0),
            JumpTarget(j) => j.0.as_indexed(&ir_allocs.jts).map(|x| x.0),
        }
    }
}
