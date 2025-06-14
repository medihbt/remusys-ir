use std::cell::Cell;

use crate::mir::{
    inst::{BrCondFlag, MachineInstCommonBase, opcode::AArch64OP},
    operand::MachineOperand,
};

#[derive(Debug, Clone)]
pub struct CondBr {
    pub common: MachineInstCommonBase,
    pub operands: [Cell<MachineOperand>; 2],
    pub cond: BrCondFlag,
}
#[derive(Debug, Clone)]
pub struct UncondBr {
    pub common: MachineInstCommonBase,
    pub operands: [Cell<MachineOperand>; 1],
}
#[derive(Debug, Clone)]
pub struct BLink {
    pub common: MachineInstCommonBase,
    pub operands: [Cell<MachineOperand>; 2],
}
#[derive(Debug, Clone)]
pub struct BrRegCond {
    pub common: MachineInstCommonBase,
    pub operands: [Cell<MachineOperand>; 2],
}

impl CondBr {
    pub fn get_label(&self) -> &Cell<MachineOperand> {
        &self.operands[0]
    }
    pub fn get_implicit_pstate(&self) -> &Cell<MachineOperand> {
        &self.operands[1]
    }
}
impl BLink {
    pub fn get_target(&self) -> &Cell<MachineOperand> {
        &self.operands[0]
    }
    pub fn get_link_reg(&self) -> &Cell<MachineOperand> {
        &self.operands[1]
    }
}
impl BrRegCond {
    pub fn get_cond_reg(&self) -> &Cell<MachineOperand> {
        &self.operands[0]
    }
    pub fn get_target(&self) -> &Cell<MachineOperand> {
        &self.operands[1]
    }

    pub fn is_cbnz(&self) -> bool {
        matches!(self.common.opcode, AArch64OP::CBNZ)
    }
    pub fn is_cbz(&self) -> bool {
        matches!(self.common.opcode, AArch64OP::CBZ)
    }
    pub fn is_tbnz(&self) -> bool {
        matches!(self.common.opcode, AArch64OP::TBNZ)
    }
    pub fn is_tbz(&self) -> bool {
        matches!(self.common.opcode, AArch64OP::TBZ)
    }
}
