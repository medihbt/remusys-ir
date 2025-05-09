use std::cell::Cell;

use slab::Slab;
use usedef::{UseData, UseRef};

use crate::{
    base::{
        slablist::{SlabRefList, SlabRefListError, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef}, slabref::SlabRef, NullableValue
    },
    impl_slabref,
    typing::{id::ValTypeID, TypeMismatchError},
};

use super::{ValueSSA, ValueSSAError, block::BlockRef, module::Module, opcode::Opcode};

pub mod binop;
pub mod callop;
pub mod cast;
pub mod cmp;
pub mod gep;
pub mod load_store;
pub mod phi;
pub mod sundury_inst;
pub mod terminator;
pub mod usedef;

mod checking;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstRef(usize);
impl_slabref!(InstRef, InstData);
impl SlabRefListNodeRef for InstRef {}

pub enum InstData {
    ListGuideNode(Cell<SlabRefListNodeHead>),
    PhiInstEnd(InstDataCommon),

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
    Phi(InstDataCommon, phi::PhiOp),

    /// Load a value from memory.
    Load(InstDataCommon, load_store::LoadOp),

    /// Store a value to memory.
    Store(InstDataCommon, load_store::StoreOp),

    /// Select a value from two options based on a condition.
    Select(InstDataCommon, sundury_inst::SelectOp),

    /// Binary operations (add, sub, mul, div, etc.).
    BinOp(InstDataCommon, binop::BinOp),

    /// Compare two values and produce a boolean result.
    Cmp(InstDataCommon, cmp::CmpOp),

    /// Cast a value from one type to another.
    Cast(InstDataCommon, cast::CastOp),

    /// Adjusts a pointer to an array or structure to the right position by indices.
    IndexPtr(InstDataCommon, gep::IndexPtrOp),

    /// Call a function and get the result.
    Call(InstDataCommon, callop::CallOp),

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
    pub self_ref: InstRef,
}

#[derive(Debug, Clone, Copy)]
pub struct InstDataInner {
    pub(super) _node_head: SlabRefListNodeHead,
    pub(super) _parent_bb: BlockRef,
}

pub enum InstError {
    OperandNull,
    OperandUninit,
    OperandOverflow,
    OperandTypeMismatch(TypeMismatchError, ValueSSA),
    OperandError(ValueSSAError),
    OperandNotComptimeConst(ValueSSA),

    InvalidCast(cast::CastError),
    InvalidArgumentCount(usize, usize),
    DividedByZero,

    SelfNotAttached(InstRef),
    SelfAlreadyAttached(InstRef, BlockRef),
    ListError(SlabRefListError),
}

trait InstDataUnique: Sized {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>);

    fn check_operands(&self, common: &InstDataCommon, module: &Module) -> Result<(), InstError>;
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
            _ => self.get_common_unwrap().inner.get()._node_head,
        }
    }

    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        match self {
            Self::ListGuideNode(cell) => cell.set(node_head),
            _ => self
                .get_common_unwrap()
                .inner
                .get()
                .insert_node_head(node_head)
                .assign_to(&self.get_common_unwrap().inner),
        }
    }
}

impl InstData {
    pub fn new_unreachable() -> Self {
        Self::Unreachable(InstDataCommon::new(Opcode::Unreachable, ValTypeID::Void, &mut Slab::new()))
    }
    pub fn new_phi_end() -> Self {
        Self::PhiInstEnd(InstDataCommon::new(Opcode::None, ValTypeID::Void, &mut Slab::new()))
    }

    pub fn get_common_unwrap(&self) -> &InstDataCommon {
        self.get_common().expect("Guide Node has no common data")
    }
    pub fn get_common(&self) -> Option<&InstDataCommon> {
        match self {
            Self::ListGuideNode(_) => None,
            Self::PhiInstEnd(common) => Some(common),
            Self::Unreachable(common) => Some(common),
            Self::Ret(common, ..) => Some(common),
            Self::Jump(common, ..) => Some(common),
            Self::Br(common, ..) => Some(common),
            Self::Switch(common, ..) => Some(common),
            Self::TailCall(common) => Some(common),
            Self::Phi(common, ..) => Some(common),
            Self::Load(common, ..) => Some(common),
            Self::Store(common, ..) => Some(common),
            Self::Select(common, ..) => Some(common),
            Self::BinOp(common, ..) => Some(common),
            Self::Cmp(common, ..) => Some(common),
            Self::Cast(common, ..) => Some(common),
            Self::IndexPtr(common, ..) => Some(common),
            Self::Call(common, ..) => Some(common),
            Self::DynCall(common) => Some(common),
            Self::Intrin(common) => Some(common),
        }
    }
    pub(super) fn common_mut(&mut self) -> Option<&mut InstDataCommon> {
        match self {
            Self::ListGuideNode(_) => None,
            Self::PhiInstEnd(common) => Some(common),
            Self::Unreachable(common) => Some(common),
            Self::Ret(common, ..) => Some(common),
            Self::Jump(common, ..) => Some(common),
            Self::Br(common, ..) => Some(common),
            Self::Switch(common, ..) => Some(common),
            Self::TailCall(common) => Some(common),
            Self::Phi(common, ..) => Some(common),
            Self::Load(common, ..) => Some(common),
            Self::Store(common, ..) => Some(common),
            Self::Select(common, ..) => Some(common),
            Self::BinOp(common, ..) => Some(common),
            Self::Cmp(common, ..) => Some(common),
            Self::Cast(common, ..) => Some(common),
            Self::IndexPtr(common, ..) => Some(common),
            Self::Call(common, ..) => Some(common),
            Self::DynCall(common, ..) => Some(common),
            Self::Intrin(common, ..) => Some(common),
        }
    }
    pub fn is_guide_node(&self) -> bool {
        matches!(self, Self::ListGuideNode(..) | Self::PhiInstEnd(..))
    }
    pub fn is_valid(&self) -> bool {
        !self.is_guide_node()
    }
    pub fn get_opcode(&self) -> Opcode {
        self.get_common_unwrap().opcode
    }
    pub fn get_value_type(&self) -> ValTypeID {
        self.get_common_unwrap().ret_type.clone()
    }

    pub fn get_parent_bb(&self) -> BlockRef {
        match self {
            Self::ListGuideNode(_) => panic!("Invalid InstData variant"),
            _ => self.get_common_unwrap().inner.get()._parent_bb,
        }
    }
    pub fn set_parent_bb(&self, parent_bb: BlockRef) {
        match self {
            Self::ListGuideNode(_) => panic!("Invalid InstData variant"),
            _ => self
                .get_common_unwrap()
                .inner
                .get()
                .insert_parent_bb(parent_bb)
                .assign_to(&self.get_common_unwrap().inner),
        }
    }

    /// Checks if this instruction ends a control flow.
    pub fn is_terminator(&self) -> bool {
        matches!(self, Self::Unreachable(_))
    }

    pub(super) fn check_operands(&self, module: &Module) -> Result<(), InstError> {
        match self {
            InstData::ListGuideNode(..)
            | InstData::PhiInstEnd(..)
            | InstData::Unreachable(..)
            | InstData::Jump(..) => Ok(()),
            InstData::Ret(c, r) => r.check_operands(c, module),
            InstData::Br(c, b) => b.check_operands(c, module),
            InstData::Switch(c, s) => s.check_operands(c, module),
            InstData::TailCall(..) => todo!("TailCall Not Implemented and maybe will be removed"),
            InstData::Phi(c, phi) => phi.check_operands(c, module),
            InstData::Load(c, ldr) => ldr.check_operands(c, module),
            InstData::Store(c, str) => str.check_operands(c, module),
            InstData::Select(c, s) => s.check_operands(c, module),
            InstData::BinOp(c, b) => b.check_operands(c, module),
            InstData::Cmp(c, cmp) => cmp.check_operands(c, module),
            InstData::Cast(c, cast) => cast.check_operands(c, module),
            InstData::IndexPtr(c, gep) => gep.check_operands(c, module),
            InstData::Call(c, call) => call.check_operands(c, module),
            InstData::DynCall(c, ..) => todo!("Dyncall not implemented and maybe will be removed"),
            InstData::Intrin(c, ..) => todo!("Intrin not implemented and maybe will be removed"),
        }
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
            self_ref: InstRef::new_null(),
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

impl InstRef {
    fn _node_attach_check(&self, module: &Module, to_attach: InstRef) -> Result<BlockRef, InstError> {
        let parent = module.get_inst(*self).get_parent_bb();
        if parent.is_null() {
            return Err(InstError::SelfNotAttached(*self));
        }
        if to_attach.is_null() {
            return Err(InstError::OperandNull);
        }
        let to_attach_bb = module.get_inst(to_attach).get_parent_bb();
        if to_attach_bb.is_nonnull() {
            return Err(InstError::SelfAlreadyAttached(to_attach, to_attach_bb));
        }
        Ok(parent)
    }
    fn _detach_check(&self, module: &Module) -> Result<BlockRef, InstError> {
        let parent = module.get_inst(*self).get_parent_bb();
        if parent.is_null() {
            return Err(InstError::SelfNotAttached(*self));
        }
        Ok(parent)
    }

    pub fn add_next_inst(&self, module: &Module, next: InstRef) -> Result<(), InstError> {
        let parent = self._node_attach_check(module, next)?;
        module.get_block(parent)
            .insructions
            .node_add_next(&module.borrow_value_alloc()._alloc_inst, *self, next)
            .map_err(InstError::ListError)
    }
    pub fn add_prev_inst(&self, module: &Module, prev: InstRef) -> Result<(), InstError> {
        let parent = self._node_attach_check(module, prev)?;
        module.get_block(parent)
            .insructions
            .node_add_prev(&module.borrow_value_alloc()._alloc_inst, *self, prev)
            .map_err(InstError::ListError)
    }
    pub fn detach_self(&self, module: &Module) -> Result<(), InstError> {
        let parent = self._detach_check(module)?;
        module.get_block(parent)
            .insructions
            .unplug_node(&module.borrow_value_alloc()._alloc_inst, *self)
            .map_err(InstError::ListError)
    }
}
