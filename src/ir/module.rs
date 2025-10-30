use crate::ir::{
    BlockID, BlockObj, ExprID, ExprObj, GlobalID, GlobalObj, InstID, InstObj, Use, UseID,
};
use mtb_entity::{EntityAlloc, PtrID};

pub struct IRAllocs {
    pub exprs: EntityAlloc<ExprObj>,
    pub insts: EntityAlloc<InstObj>,
    pub globals: EntityAlloc<GlobalObj>,
    pub blocks: EntityAlloc<BlockObj>,
    pub uses: EntityAlloc<Use>,
}

impl IRAllocs {
    pub fn new() -> Self {
        Self {
            exprs: EntityAlloc::new(),
            insts: EntityAlloc::new(),
            globals: EntityAlloc::new(),
            blocks: EntityAlloc::new(),
            uses: EntityAlloc::new(),
        }
    }

    pub fn with_capacity(base_cap: usize) -> Self {
        Self {
            exprs: EntityAlloc::with_capacity(base_cap * 4),
            insts: EntityAlloc::with_capacity(base_cap * 4),
            globals: EntityAlloc::with_capacity(base_cap),
            blocks: EntityAlloc::with_capacity(base_cap * 2),
            uses: EntityAlloc::with_capacity(base_cap * 8),
        }
    }
}

pub trait IPoolAllocated: Sized {
    type ModuleID: Copy;

    fn get_alloc(ir_allocs: &IRAllocs) -> &EntityAlloc<Self>;
    fn alloc_mut(ir_allocs: &mut IRAllocs) -> &mut EntityAlloc<Self>;

    fn make_module_id(raw: PtrID<Self>) -> Self::ModuleID;
    fn from_module_id(id: Self::ModuleID) -> PtrID<Self>;
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
}
