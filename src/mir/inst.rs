use std::cell::Cell;

use crate::{base::slablist::SlabRefListNodeHead, ir::cmp_cond::CmpCond};

use super::{opcode::MachineOpcode, operand::MachineOperand};

pub enum MachineInst {
    GuideNode(Cell<MachineInstInner>),

    Nop,

    /// aarch64 manual C3.1: Branch instructions
    BranchCond(MachineInstBranch),
    BConsistant,
    CmpBranch,
    TestBranch,

    Branch,
    BLink,

    BLinkReg,
    BranchReg,
    Ret,

    MRS, MSR,

    /// C3.2: Loads and Stores
    Ldr,
    Ldp,
    Str,
    Stp,

    /// C3.5: Data Processing
    AluUnaOp,
    AluBinOp,
    AluTriOp,
}

pub struct MachineInstCommon {
    pub list_head: Cell<MachineInstInner>,
    pub opcode:    MachineOpcode,
}

#[derive(Debug, Clone, Copy)]
pub struct MachineInstInner {
    pub list_head: SlabRefListNodeHead,
}

pub struct MachineInstBranch {
    pub common: MachineInstCommon,
    pub cond:   CmpCond,
    pub target: [Cell<MachineOperand>; 1],
}