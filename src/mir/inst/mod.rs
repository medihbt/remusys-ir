use std::cell::Cell;

use crate::base::slablist::SlabRefListNodeHead;
use opcode::AArch64OP;

pub mod opcode;
pub mod fixop;

pub enum MachineInst {
    GuideNode(MachineInstCommonBase), // A guide node for the instruction list, used for padding.
    NOP(MachineInstCommonBase),       // No operation, used for padding.
    Hint,
    Barrier,

    Branch,
    BranchCond,

    Load,
    Store,
}

pub struct MachineInstCommonBase {
    pub self_head: Cell<SlabRefListNodeHead>,
    pub opcode:    AArch64OP,
}
