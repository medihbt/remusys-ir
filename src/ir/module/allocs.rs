use crate::ir::*;
use mtb_entity::{
    EntityAlloc, EntityAllocPolicy128, EntityAllocPolicy256, EntityAllocPolicy512,
    EntityAllocPolicy4096, IEntityAllocatable, PtrID,
};

pub struct IRAllocs {
    pub exprs: EntityAlloc<ExprObj>,
    pub insts: EntityAlloc<InstObj>,
    pub globals: EntityAlloc<GlobalObj>,
    pub blocks: EntityAlloc<BlockObj>,
    pub uses: EntityAlloc<Use>,
    pub jts: EntityAlloc<JumpTarget>,
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
        }
    }
}

pub trait IPoolAllocated: IEntityAllocatable {
    type ModuleID: Copy;

    fn get_alloc(ir_allocs: &IRAllocs) -> &EntityAlloc<Self>;
    fn alloc_mut(ir_allocs: &mut IRAllocs) -> &mut EntityAlloc<Self>;

    fn make_module_id(raw: PtrID<Self>) -> Self::ModuleID;
    fn from_module_id(id: Self::ModuleID) -> PtrID<Self>;

    fn dispose_id(id: Self::ModuleID, ir_allocs: &IRAllocs);
}

impl IEntityAllocatable for BlockObj {
    /// Allocate 256 entities per allocation block.
    type AllocatePolicyT = EntityAllocPolicy256<Self>;
}
impl IPoolAllocated for BlockObj {
    type ModuleID = BlockID;

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
}

impl IEntityAllocatable for InstObj {
    /// Allocate 512 entities per allocation block.
    type AllocatePolicyT = EntityAllocPolicy512<Self>;
}
impl IPoolAllocated for InstObj {
    type ModuleID = InstID;

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
}

impl IEntityAllocatable for ExprObj {
    /// Allocate 256 entities per allocation block.
    type AllocatePolicyT = EntityAllocPolicy256<Self>;
}
impl IPoolAllocated for ExprObj {
    type ModuleID = ExprID;

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
}

impl IEntityAllocatable for GlobalObj {
    /// Allocate 128 entities per allocation block.
    type AllocatePolicyT = EntityAllocPolicy128<Self>;
}
impl IPoolAllocated for GlobalObj {
    type ModuleID = GlobalID;

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
    fn dispose_id(id: Self::ModuleID, ir_allocs: &IRAllocs) {
        id.dispose(ir_allocs);
    }
}

impl IEntityAllocatable for Use {
    /// Allocate 4096 entities per allocation block.
    type AllocatePolicyT = EntityAllocPolicy4096<Self>;
}
impl IPoolAllocated for Use {
    type ModuleID = UseID;

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
        id.deref_ir(allocs).dispose(allocs);
    }
}

impl IEntityAllocatable for JumpTarget {
    /// Allocate 256 entities per allocation block.
    type AllocatePolicyT = EntityAllocPolicy256<Self>;
}
impl IPoolAllocated for JumpTarget {
    type ModuleID = JumpTargetID;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PoolAllocatedValue {
    Block(BlockID),
    Inst(InstID),
    Expr(ExprID),
    Global(GlobalID),
    Use(UseID),
    JumpTarget(JumpTargetID),
}
