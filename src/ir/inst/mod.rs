pub mod instructions;
pub mod callop;
pub mod jump_targets;
pub mod phi;
pub mod terminator;
pub mod usedef;

use std::cell::{Cell, RefCell};

use slab::Slab;
use terminator::*;
use usedef::{UseData, UseRef};

use crate::{
    base::{
        slablist::{SlabRefList, SlabRefListError, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef},
        slabref::SlabRef,
    },
    typing::id::ValTypeID,
};

use super::{Module, block::BlockRef, opcode::Opcode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A reference to an instruction.
pub struct InstRef(pub(crate) usize);

pub enum Inst {
    /// Head or tail node of the instruction list, used to mark the beginning
    /// or end of the instruction list.
    ///
    /// NOTE: This is NOT a valid instruction.
    ListGuideNode(Cell<SlabRefListNodeHead>),

    // Terminator instructions. These instructions are put at the end of a block and
    // transfer control to another block or return from a function.
    /// Mark this block as unreachable.
    Unreachable (InstCommon, Unreachable),
    /// Return from a function, sometimes with a value.
    Ret         (InstCommon, Cell<Ret>),
    /// Jump to another block unconditionally.
    Jump        (InstCommon, Jump),
    /// Branch to one of two blocks based on a condition.
    Br          (InstCommon, Br),
    /// Branch to one of many blocks based on a condition.
    Switch      (InstCommon, Switch),
    /// Call a function and transfer control to the callee.
    /// Return type of the tail call instruction should be same as
    /// the callee function return type.
    TailCall    (InstCommon, TailCallOp),

    // Non-terminator instructions. These instructions are put in the middle of a block
    // and do not transfer control to another block or return from a function.
    /// PHI Node. This instruction is used to select a value based on the control flow.
    Phi         (InstCommon, RefCell<phi::PhiNode>),

    /// Select a value based on the control flow.
    BinSelect   (InstCommon, instructions::BinSelect),

    /// Binary operation. This instruction is used to perform a binary operation on two
    /// values.
    BinOp       (InstCommon, instructions::BinOp),

    /// Compare operation. This instruction is used to perform a comparison operation
    /// on two values.
    /// The result of the comparison is a boolean value.
    Cmp         (InstCommon, instructions::BinOp),

    /// Cast operation. This instruction is used to perform a cast operation on a value.
    /// The result of the cast operation is a value of a different type.
    Cast        (InstCommon, instructions::CastOp),

    /// GetElemtPtr operation. This instruction adjusts the pointer to point to
    /// the right position in the array or struct.
    IndexPtr    (InstCommon, instructions::IndexPtrOp),

    /// Static call operation with a known function target.
    Call        (InstCommon, callop::CallOp),

    /// Dynamic call operation with its callee being anything that holds a
    /// function pointer.
    DynCall     (InstCommon, callop::CallOp),

    /// Loads value from a pointer.
    Load        (InstCommon, instructions::LoadOp),

    /// Store value to a pointer.
    Store       (InstCommon, instructions::StoreOp),
}

pub trait InstDataTrait {
    /// Initialize the common data after initializing the opcode-dependent
    /// data of the instruction.
    ///
    /// This function should be called after the opcode-dependent data of the
    /// instruction has been initialized.
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        InstCommon::new(opcode, ty, parent, module)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InstMutInner {
    node_head: SlabRefListNodeHead,
    parent:    BlockRef,
}

pub struct InstCommon {
    /// The opcode of the instruction.
    pub opcode: Opcode,

    /// The type of the instruction.
    pub ty: ValTypeID,

    /// The operands of the instruction.
    pub operands: SlabRefList<UseRef>,

    /// The common mutable part of the instruction.
    pub(crate) inner: Cell<InstMutInner>,
}

impl SlabRef for InstRef {
    type RefObject = Inst;

    fn from_handle(handle: usize) -> Self {
        Self(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl SlabRefListNode for Inst {
    fn new_guide() -> Self {
        Inst::ListGuideNode(Cell::new(SlabRefListNodeHead::new()))
    }

    fn load_node_head(&self) -> SlabRefListNodeHead {
        match self {
            Inst::ListGuideNode(node_head) => node_head.get(),
            _ => self.get_common().inner.get().node_head,
        }
    }

    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        match self {
            Inst::ListGuideNode(_) => panic!("Cannot store node head to ListGuideNode"),
            _ => {
                let inner = &self.get_common().inner;
                inner.get()
                     .insert_head(node_head)
                     .set_into(inner);
            }
        }
    }
}

impl SlabRefListNodeRef for InstRef {}

impl Inst {
    pub fn get_common(&self) -> &InstCommon {
        match self {
            Inst::ListGuideNode(_) => panic!(
                "Inst::ListGuideNode is head or tail of instruction list, NOT valid instruction"
            ),

            Inst::Unreachable(common, _) => common,
            Inst::Ret(common, _) => common,
            Inst::Jump(common, _) => common,
            Inst::Br(common, _) => common,
            Inst::Switch(common, _) => common,
            Inst::TailCall(common, _) => common,

            Inst::Phi(common, _) => common,

            Inst::BinSelect(common, _) => common,
            Inst::BinOp(common, _) => common,
            Inst::Cmp(common, _) => common,
            Inst::Cast(common, _) => common,
            Inst::IndexPtr(common, _) => common,
            Inst::Call(common, _) => common,
            Inst::DynCall(common, _) => common,
            Inst::Load(common, _) => common,
            Inst::Store(common, _) => common,
        }
    }

    pub fn get_opcode(&self) -> Opcode {
        self.get_common().opcode
    }
    pub fn get_ty(&self) -> ValTypeID {
        self.get_common().ty.clone()
    }

    pub fn get_parent(&self) -> BlockRef {
        self.get_common().inner.get().parent
    }
    pub fn set_parent(&self, parent: BlockRef) {
        self.get_common().inner.get()
            .insert_parent(parent)
            .set_into(&self.get_common().inner);
    }

    pub fn is_terminator(&self) -> bool {
        matches!(self,
            Inst::Unreachable(..) | Inst::Ret(..) |
            Inst::Jump(..) | Inst::Br(..) | Inst::Switch(..) |
            Inst::TailCall(..)
        )
    }
}

impl InstMutInner {
    pub fn new(parent: BlockRef) -> Self {
        Self {
            node_head: SlabRefListNodeHead::new(),
            parent,
        }
    }
    pub fn insert_head(&self, node_head: SlabRefListNodeHead) -> Self {
        Self {
            node_head,
            parent: self.parent,
        }
    }
    pub fn insert_parent(&self, parent: BlockRef) -> Self {
        Self {
            node_head: self.node_head,
            parent,
        }
    }
    pub fn set_into(self, value: &Cell<Self>) {
        value.set(self);
    }
}

impl InstCommon {
    pub fn new(opcode: Opcode, ty: ValTypeID, parent: BlockRef, module: &mut Module) -> Self {
        Self {
            opcode, ty,
            operands: SlabRefList::from_slab(&mut module._alloc_use),
            inner: Cell::new(InstMutInner::new(parent)),
        }
    }

    pub(super) fn add_use(&self, use_data: UseData, alloc: &mut Slab<UseData>) -> UseRef {
        let useref = UseRef::from_handle(alloc.insert(use_data));
        self.operands
            .push_back_ref(alloc, useref.clone())
            .expect("Failed to add use reference to instruction");
        useref
    }

    #[allow(dead_code)]
    pub(super) fn remove_use(&self, use_ref: UseRef, alloc: &Slab<UseData>) -> Result<UseRef, SlabRefListError> {
        self.operands
            .unplug_node(alloc, use_ref.clone())
            .map(|_| use_ref)
    }
}
