use std::cell::Cell;

use slab::Slab;
use usedef::{UseData, UseRef};

use crate::{
    base::{
        NullableValue,
        slablist::{SlabRefList, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef},
        slabref::SlabRef,
    },
    impl_slabref,
    typing::id::ValTypeID,
};

use super::{block::BlockRef, module::Module, opcode::Opcode};

pub mod binop;
pub mod callop;
pub mod cmp;
pub mod gep;
pub mod load_store;
pub mod terminator;
pub mod usedef;
pub mod sundury_inst;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstRef(usize);
impl_slabref!(InstRef, InstData);
impl SlabRefListNodeRef for InstRef {}

pub enum InstData {
    ListGuideNode(Cell<SlabRefListNodeHead>),

    // Terminator instructions. These instructions are put at the end of a block and
    // transfer control to another block or return from a function.
    /// Mark this block as unreachable.
    Unreachable(InstDataCommon),

    /// Return from a function, sometimes with a value.
    Ret(InstDataCommon, terminator::Ret),

    /// Jump to another block unconditionally.
    Jump(InstDataCommon, terminator::Jump),

    /// Branch to one of two blocks based on a condition.
    Br(InstDataCommon, terminator::Br),

    /// Switch to one of multiple blocks based on a value.
    Switch(InstDataCommon, terminator::Switch),

    /// Call a function while transferring control to the callee.
    TailCall(InstDataCommon),

    // Non-terminator instructions. These instructions are put in the middle of a block
    // and do not transfer control to another block or return from a function.
    /// PHI Node. This instruction is used to select a value based on the control flow.
    Phi(InstDataCommon),

    /// Load a value from memory.
    Load(InstDataCommon),

    /// Store a value to memory.
    Store(InstDataCommon),

    /// Select a value from two options based on a condition.
    Select(InstDataCommon),

    /// Binary operations (add, sub, mul, div, etc.).
    BinOp(InstDataCommon),

    /// Compare two values and produce a boolean result.
    Cmp(InstDataCommon),

    /// Cast a value from one type to another.
    Cast(InstDataCommon),

    /// Adjusts a pointer to an array or structure to the right position by indices.
    IndexPtr(InstDataCommon),

    /// Call a function and get the result.
    Call(InstDataCommon),

    /// Call a value and get the result.
    DynCall(InstDataCommon),

    /// Call an intrinsic function and get the result.
    Intrin(InstDataCommon),
}

pub struct InstDataCommon {
    pub inner: Cell<InstDataInner>,
    pub opcode: Opcode,
    pub operands: SlabRefList<UseRef>,
    pub ret_type: ValTypeID,
}

#[derive(Debug, Clone, Copy)]
pub struct InstDataInner {
    pub(super) _node_head: SlabRefListNodeHead,
    pub(super) _parent_bb: BlockRef,
}

trait InstDataUnique {
    fn update_build_common(
        &mut self,
        common: InstDataCommon,
        mut_module: &Module,
    ) -> InstDataCommon;
}

impl InstDataInner {
    fn insert_node_head(self, node_head: SlabRefListNodeHead) -> Self {
        Self {
            _node_head: node_head,
            _parent_bb: self._parent_bb,
        }
    }
    fn insert_parent_bb(self, parent_bb: BlockRef) -> Self {
        Self {
            _node_head: self._node_head,
            _parent_bb: parent_bb,
        }
    }
    fn assign_to(&self, cell: &Cell<InstDataInner>) {
        cell.set(*self);
    }
}

impl SlabRefListNode for InstData {
    fn new_guide() -> Self {
        Self::ListGuideNode(Cell::new(SlabRefListNodeHead::new()))
    }

    fn load_node_head(&self) -> SlabRefListNodeHead {
        match self {
            Self::ListGuideNode(cell) => cell.get(),
            _ => self.get_common().inner.get()._node_head,
        }
    }

    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        match self {
            Self::ListGuideNode(cell) => cell.set(node_head),
            _ => self
                .get_common()
                .inner
                .get()
                .insert_node_head(node_head)
                .assign_to(&self.get_common().inner),
        }
    }
}

impl InstData {
    pub fn get_common(&self) -> &InstDataCommon {
        match self {
            Self::ListGuideNode(_) => panic!("Invalid InstData variant"),
            Self::Unreachable(common) => common,
            Self::Ret(common, _) => common,
            Self::Jump(common, _) => common,
            Self::Br(common, _) => common,
            Self::Switch(common, _) => common,
            Self::TailCall(common) => common,
            Self::Phi(common) => common,
            Self::Load(common) => common,
            Self::Store(common) => common,
            Self::Select(common) => common,
            Self::BinOp(common) => common,
            Self::Cmp(common) => common,
            Self::Cast(common) => common,
            Self::IndexPtr(common) => common,
            Self::Call(common) => common,
            Self::DynCall(common) => common,
            Self::Intrin(common) => common,
        }
    }
    pub fn get_opcode(&self) -> Opcode {
        self.get_common().opcode
    }
    pub fn get_value_type(&self) -> ValTypeID {
        self.get_common().ret_type.clone()
    }

    pub fn get_parent_bb(&self) -> BlockRef {
        match self {
            Self::ListGuideNode(_) => panic!("Invalid InstData variant"),
            _ => self.get_common().inner.get()._parent_bb,
        }
    }
    pub fn set_parent_bb(&self, parent_bb: BlockRef) {
        match self {
            Self::ListGuideNode(_) => panic!("Invalid InstData variant"),
            _ => self
                .get_common()
                .inner
                .get()
                .insert_parent_bb(parent_bb)
                .assign_to(&self.get_common().inner),
        }
    }

    /// Checks if this instruction ends a control flow.
    pub fn is_terminator(&self) -> bool {
        matches!(self, Self::Unreachable(_))
    }
}

impl InstDataCommon {
    pub fn new(opcode: Opcode, ret_type: ValTypeID, alloc_use: &mut Slab<UseData>) -> Self {
        Self {
            inner: Cell::new(InstDataInner {
                _node_head: SlabRefListNodeHead::new(),
                _parent_bb: BlockRef::new_null(),
            }),
            opcode,
            operands: SlabRefList::from_slab(alloc_use),
            ret_type,
        }
    }

    fn add_use(&self, alloc_use: &mut Slab<UseData>, use_data: UseData) -> UseRef {
        let use_ref = alloc_use.insert(use_data);
        self.operands
            .push_back_ref(alloc_use, UseRef::from_handle(use_ref))
            .expect("Failed to add use reference to instruction");
        UseRef::from_handle(use_ref)
    }
    fn alloc_use(&self, alloc_use: &mut Slab<UseData>) -> UseRef {
        self.add_use(alloc_use, UseData::new_guide())
    }

    fn remove_use(&self, alloc_use: &mut Slab<UseData>, use_ref: UseRef) {
        self.operands
            .unplug_node(alloc_use, use_ref)
            .expect("Failed to remove use reference from instruction");
    }
}
