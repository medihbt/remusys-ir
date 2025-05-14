use std::cell::Cell;

use slab::Slab;
use terminator::TerminatorInst;
use usedef::{UseData, UseRef};

use crate::{
    base::{
        NullableValue,
        slablist::{
            SlabRefList, SlabRefListError, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef,
        },
        slabref::SlabRef,
    },
    impl_slabref,
    typing::{TypeMismatchError, id::ValTypeID},
};

use super::{
    ValueSSA, ValueSSAError,
    block::{BlockRef, jump_target::JumpTargetData},
    module::{Module, ModuleAllocatorInner},
    opcode::Opcode,
};

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
pub mod visitor;

mod checking;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstRef(usize);
impl_slabref!(InstRef, InstData);
impl SlabRefListNodeRef for InstRef {
    fn on_node_push_next(
        curr: Self,
        next: Self,
        alloc: &Slab<InstData>,
    ) -> Result<(), SlabRefListError> {
        let curr_data = curr.to_slabref_unwrap(alloc);
        curr._node_attach_modify_parent(curr_data, next, alloc)
    }

    fn on_node_push_prev(
        curr: Self,
        prev: Self,
        alloc: &Slab<Self::RefObject>,
    ) -> Result<(), SlabRefListError> {
        let curr_data = curr.to_slabref_unwrap(alloc);
        curr._node_attach_modify_parent(curr_data, prev, alloc)
    }

    fn on_node_unplug(curr: Self, alloc: &Slab<Self::RefObject>) -> Result<(), SlabRefListError> {
        let curr_data = curr.to_slabref_unwrap(alloc);
        curr._node_detach_clean_parent(curr_data)
    }
}

pub enum InstData {
    /// Instruction list guide node containing a simple header and parent block.
    /// The guide node will be always attached to a block, so its parent block
    /// will be initialized when the block is allocated on `module.inner._alloc_block`.
    ListGuideNode(Cell<SlabRefListNodeHead>, Cell<BlockRef>),
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

    /// ## Parent Basic Block
    ///
    /// `None` if this instruction is not attached to any block, and `null` if
    /// this instruction is attached to a block that is not allocated to the module.
    ///
    /// ### Why use `Option`
    ///
    /// Sometimes we need to insert an instruction into a block that is not
    /// allocated to the module yet. Since the block has no ID, the `_parent_bb`
    /// is `null` instead of `None`.
    ///
    /// If we use `BlockRef` here, it is impossible to distinguish between the case
    /// above and the case where the instruction is not attached to any block.
    /// The checker may return an error unexpectedly.
    ///
    /// So we use `Option<BlockRef>`.
    pub(super) _parent_bb: Option<BlockRef>,
}

#[derive(Debug, Clone, Copy)]
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
    ReplicatedTerminator(InstRef, InstRef),
}

trait InstDataUnique: Sized {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>);

    fn check_operands(&self, common: &InstDataCommon, module: &Module) -> Result<(), InstError>;
}

impl InstDataInner {
    fn insert_node_head(mut self, node_head: SlabRefListNodeHead) -> Self {
        self._node_head = node_head;
        self
    }
    pub(super) fn insert_parent_bb(mut self, parent_bb: Option<BlockRef>) -> Self {
        self._parent_bb = parent_bb;
        self
    }
    pub(super) fn assign_to(&self, cell: &Cell<InstDataInner>) {
        cell.set(*self);
    }
}

impl SlabRefListNode for InstData {
    fn new_guide() -> Self {
        Self::ListGuideNode(
            Cell::new(SlabRefListNodeHead::new()),
            Cell::new(BlockRef::new_null()),
        )
    }

    fn load_node_head(&self) -> SlabRefListNodeHead {
        match self {
            Self::ListGuideNode(cell, _) => cell.get(),
            _ => self.get_common_unwrap().inner.get()._node_head,
        }
    }

    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        match self {
            Self::ListGuideNode(cell, _) => cell.set(node_head),
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
    pub fn new_unreachable(alloc_use: &mut Slab<UseData>) -> Self {
        Self::Unreachable(InstDataCommon::new(
            Opcode::Unreachable,
            ValTypeID::Void,
            alloc_use,
        ))
    }
    pub fn new_phi_end() -> Self {
        let common = InstDataCommon {
            inner: Cell::new(InstDataInner {
                _node_head: SlabRefListNodeHead::new(),
                _parent_bb: None,
            }),
            opcode: Opcode::None,
            operands: SlabRefList::new_guide(),
            ret_type: ValTypeID::Void,
            self_ref: InstRef::new_null(),
        };
        Self::PhiInstEnd(common)
    }

    pub fn get_common_unwrap(&self) -> &InstDataCommon {
        self.get_common().expect("Guide Node has no common data")
    }
    pub fn get_common(&self) -> Option<&InstDataCommon> {
        match self {
            Self::ListGuideNode(..) => None,
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
            Self::ListGuideNode(..) => None,
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

    pub fn get_parent_bb(&self) -> Option<BlockRef> {
        match self {
            // List guide node is always attached to a block,
            // so there is no possibility of `None`.
            Self::ListGuideNode(_, parent) => Some(parent.get()),

            // For other instructions, the parent block may be `None` or `null`.
            _ => self.get_common_unwrap().inner.get()._parent_bb,
        }
    }
    pub fn set_parent_bb(&self, parent_bb: Option<BlockRef>) {
        match self {
            Self::ListGuideNode(..) => panic!("Inst guide node parent is immutable"),
            _ => self
                .get_common_unwrap()
                .inner
                .get()
                .insert_parent_bb(parent_bb)
                .assign_to(&self.get_common_unwrap().inner),
        }
    }
    pub fn is_attached(&self) -> bool {
        self.get_parent_bb().is_some()
    }

    /// Checks if this instruction ends a control flow.
    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            Self::Unreachable(..) | Self::Ret(..) | Self::Br(..) | Self::Switch(..)
        )
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
            InstData::DynCall(..) => todo!("Dyncall not implemented and maybe will be removed"),
            InstData::Intrin(..) => todo!("Intrin not implemented and maybe will be removed"),
        }
    }
    pub(super) fn _inst_init_self_reference(
        &mut self,
        self_ref: InstRef,
        alloc_use: &Slab<UseData>,
    ) {
        let common = match self.common_mut() {
            Some(common) => common,
            None => return,
        };

        common.self_ref = self_ref;
        let mut opref = common.operands._head;
        while opref.is_nonnull() {
            opref.to_slabref_unwrap(alloc_use)._user.set(self_ref);
            opref = match opref.get_next_ref(alloc_use) {
                Some(next) => next,
                None => break,
            };
        }
    }

    pub fn as_terminator(&self) -> Option<(&InstDataCommon, &dyn terminator::TerminatorInst)> {
        match self {
            Self::Unreachable(_) => None,
            Self::Ret(common, ret) => Some((common, ret)),
            Self::Jump(common, jump) => Some((common, jump)),
            Self::Br(common, br) => Some((common, br)),
            Self::Switch(common, switch) => Some((common, switch)),
            _ => None,
        }
    }
    pub unsafe fn load_operand_view(&self) -> Option<SlabRefList<UseRef>> {
        match self {
            Self::ListGuideNode(..)
            | Self::PhiInstEnd(..)
            | Self::Unreachable(..)
            | Self::Jump(..) => None,
            _ => unsafe {
                Some(
                    self.get_common_unwrap()
                        .operands
                        .unsafe_load_readonly_view(),
                )
            },
        }
    }
}

impl InstDataCommon {
    pub fn new(opcode: Opcode, ret_type: ValTypeID, alloc_use: &mut Slab<UseData>) -> Self {
        Self {
            inner: Cell::new(InstDataInner {
                _node_head: SlabRefListNodeHead::new(),
                _parent_bb: None,
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

    fn remove_use(&self, alloc_use: &Slab<UseData>, use_ref: UseRef) {
        self.operands
            .unplug_node(alloc_use, use_ref)
            .expect("Failed to remove use reference from instruction");
    }
}

impl InstRef {
    fn _node_attach_modify_parent(
        self,
        self_data: &InstData,
        next: InstRef,
        alloc_inst: &Slab<InstData>,
    ) -> Result<(), SlabRefListError> {
        let curr_parent = match self_data.get_parent_bb() {
            Some(p) => p,
            None => return Err(SlabRefListError::SelfNotInList(self.get_handle())),
        };
        let next_data = next.to_slabref_unwrap(alloc_inst);
        if let Some(_) = next_data.get_parent_bb() {
            return Err(SlabRefListError::PluggedItemAttached(next.get_handle()));
        }
        next_data.set_parent_bb(Some(curr_parent));
        Ok(())
    }
    fn _node_detach_clean_parent(self, self_data: &InstData) -> Result<(), SlabRefListError> {
        match self_data.get_parent_bb() {
            Some(_) => {
                self_data.set_parent_bb(None);
                Ok(())
            }
            None => Err(SlabRefListError::UnpluggedItemAttached(self.get_handle())),
        }
    }

    pub fn add_next_inst(&self, module: &Module, next: InstRef) -> Result<(), InstError> {
        let self_data = module.get_inst(*self);
        let parent = match self_data.get_parent_bb() {
            Some(p) => p,
            None => {
                return Err(InstError::ListError(SlabRefListError::SelfNotInList(
                    self.get_handle(),
                )));
            }
        };
        module
            .get_block(parent)
            .instructions
            .node_add_next(&module.borrow_value_alloc().alloc_inst, *self, next)
            .map_err(InstError::ListError)
    }
    pub fn add_prev_inst(&self, module: &Module, prev: InstRef) -> Result<(), InstError> {
        let self_data = module.get_inst(*self);
        let parent = match self_data.get_parent_bb() {
            Some(p) => p,
            None => {
                return Err(InstError::ListError(SlabRefListError::SelfNotInList(
                    self.get_handle(),
                )));
            }
        };
        module
            .get_block(parent)
            .instructions
            .node_add_prev(&module.borrow_value_alloc().alloc_inst, *self, prev)
            .map_err(InstError::ListError)
    }
    pub fn detach_self(&self, module: &Module) -> Result<(), InstError> {
        let self_data = module.get_inst(*self);
        let parent = match self_data.get_parent_bb() {
            Some(p) => p,
            None => {
                return Err(InstError::ListError(SlabRefListError::SelfNotInList(
                    self.get_handle(),
                )));
            }
        };
        module
            .get_block(parent)
            .instructions
            .unplug_node(&module.borrow_value_alloc().alloc_inst, *self)
            .map_err(InstError::ListError)
    }

    /// Finalize the instruction by removing all operands and jump targets.
    pub fn finalize_with_module(&self, module: &Module) {
        let alloc_value = module.borrow_value_alloc();
        let use_alloc = module.borrow_use_alloc();
        let jt_alloc = module.borrow_jt_alloc();
        let self_data = self.to_slabref_unwrap(&alloc_value.alloc_inst);

        // Clean up jump targets of the terminators.
        let operands_view = match self_data {
            InstData::Jump(_, jump) => {
                jump.set_block(module, BlockRef::new_null());
                return;
            }
            InstData::Br(_, br) => {
                br.if_true.set_block(module, BlockRef::new_null());
                br.if_false.set_block(module, BlockRef::new_null());
                br.set_cond(module, ValueSSA::new_null());
                return;
            }
            InstData::Switch(_, switch) => {
                switch.set_cond(module, ValueSSA::new_null());
                switch.set_default(module, BlockRef::new_null());
                for (_, pred) in &*switch.borrow_cases() {
                    pred.set_block(module, BlockRef::new_null());
                }
                return;
            }
            InstData::Unreachable(..) | InstData::PhiInstEnd(..) | InstData::ListGuideNode(..) => {
                return;
            }
            _ => unsafe {
                self_data
                    .get_common_unwrap()
                    .operands
                    .unsafe_load_readonly_view()
            },
        };

        if operands_view.is_empty() {
            return;
        }

        // Clean up operands of the instruction.
        for (useref, data) in operands_view.view(&use_alloc) {
            let operand = data.get_operand();
            data.set_operand_nordfg(ValueSSA::new_null());
            if operand.is_null() {
                continue;
            }
            module.operand_del_use(operand, useref).unwrap();
        }
    }
}
