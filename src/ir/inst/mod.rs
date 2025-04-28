pub mod instructions;
pub mod phi;
pub mod terminator;
pub mod usedef;

use std::cell::RefCell;

use slab::Slab;
use terminator::*;
use usedef::{UseData, UseRef};

use crate::{base::slabref::SlabRef, typing::id::ValTypeID};

use super::{Module, block::BlockRef, opcode::Opcode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A reference to an instruction.
pub struct InstRef(pub(crate) usize);

pub enum Inst {
    // Terminator instructions. These instructions are put at the end of a block and
    // transfer control to another block or return from a function.
    /// Mark this block as unreachable.
    Unreachable(InstCommon, Unreachable),
    /// Return from a function, sometimes with a value.
    Ret(InstCommon, Ret),
    /// Jump to another block unconditionally.
    Jump(InstCommon, Jump),
    /// Branch to one of two blocks based on a condition.
    Br(InstCommon, Br),
    /// Branch to one of many blocks based on a condition.
    Switch(InstCommon, Switch),
    /// Call a function and transfer control to the callee.
    /// Return type of the tail call instruction should be same as
    /// the callee function return type.
    TailCall(InstCommon, TailCallOp),

    // Non-terminator instructions. These instructions are put in the middle of a block
    // and do not transfer control to another block or return from a function.
    /// PHI Node. This instruction is used to select a value based on the control flow.
    Phi(InstCommon, phi::PhiNode),

    /// Select a value based on the control flow.
    BinSelect(InstCommon, instructions::BinSelect),

    /// Binary operation. This instruction is used to perform a binary operation on two
    /// values.
    BinOp(InstCommon, instructions::BinOp),

    /// Compare operation. This instruction is used to perform a comparison operation
    /// on two values.
    /// The result of the comparison is a boolean value.
    Cmp(InstCommon, instructions::BinOp),

    /// Cast operation. This instruction is used to perform a cast operation on a value.
    /// The result of the cast operation is a value of a different type.
    Cast(InstCommon, instructions::CastOp),

    /// GetElemtPtr operation. This instruction adjusts the pointer to point to
    /// the right position in the array or struct.
    IndexPtr(InstCommon, instructions::IndexPtrOp),

    /// Static call operation with a known function target.
    Call(InstCommon, instructions::CallOp),

    /// Dynamic call operation with its callee being anything that holds a
    /// function pointer.
    DynCall(InstCommon, instructions::CallOp),
    
    /// Loads value from a pointer.
    Load(InstCommon, instructions::LoadOp),

    /// Store value to a pointer.
    Store(InstCommon, instructions::StoreOp),
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

pub struct InstCommon {
    /// The opcode of the instruction.
    pub opcode: Opcode,

    /// The type of the instruction.
    pub ty: ValTypeID,
    /// The parent block of the instruction.
    pub parent: BlockRef,

    /// Next instruction in the block.
    pub next: Option<InstRef>,
    /// Previous instruction in the block.
    pub prev: Option<InstRef>,

    /// The operands of the instruction.
    pub op_begin: UseRef,
    pub op_end:   UseRef,
}

impl SlabRef for InstRef {
    type Item = RefCell<Inst>;

    fn from_handle(handle: usize) -> Self {
        Self(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl Inst {
    pub fn get_common(&self) -> &InstCommon {
        match self {
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
    pub fn common_mut(&mut self) -> &mut InstCommon {
        match self {
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
        self.get_common().parent
    }
}

impl InstCommon {
    pub fn new(opcode: Opcode, ty: ValTypeID, parent: BlockRef, module: &mut Module) -> Self {
        let ret = Self {
            opcode,
            ty,
            parent,
            next: None,
            prev: None,
            op_begin: module.alloc_use(UseData::new(InstRef::new_nil())),
            op_end: module.alloc_use(UseData::new(InstRef::new_nil())),
        };

        ret.op_begin.modify_slabref(&mut module._alloc_use, |v| {
            v.prev = None;
            v.next = Some(ret.op_end);
        });
        ret.op_end.modify_slabref(&mut module._alloc_use, |v| {
            v.prev = Some(ret.op_begin);
            v.next = None;
        });

        ret
    }

    fn get_use_back(&self, alloc: &Slab<UseData>) -> UseRef {
        self.op_end
            .read_slabref(alloc, |v| v.prev)
            .unwrap()
            .unwrap()
    }

    /// Adding a 'use' Step1: add Use data
    fn _add_use_modify(&self, use_data: &mut UseData, alloc: &Slab<UseData>) {
        use_data.prev = Some(self.get_use_back(alloc));
        use_data.next = Some(self.op_end);
    }

    /// Adding a 'use' Step2: modify the previous use
    fn _add_use_fill(&self, use_ref: UseRef, alloc: &mut Slab<UseData>) {
        self.get_use_back(alloc).modify_slabref(alloc, |v| {
            v.next = Some(use_ref);
        });
        self.op_end.modify_slabref(alloc, |v| {
            v.prev = Some(use_ref);
        });
    }

    /// Adding a 'use'
    fn add_use(&self, mut use_data: UseData, alloc: &mut Slab<UseData>) -> UseRef {
        self._add_use_modify(&mut use_data, alloc);
        let use_ref = UseRef::from_handle(alloc.insert(use_data));
        self._add_use_fill(use_ref, alloc);
        use_ref
    }
}
