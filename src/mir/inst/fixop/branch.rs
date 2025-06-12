use crate::mir::{
    inst::{opcode::AArch64OP, BrCondFlag, MachineInstCommonBase},
    operand::MachineOperand,
};

pub struct CondBr {
    pub common: MachineInstCommonBase,
    pub oprnds: [MachineOperand; 2],
    pub cond: BrCondFlag,
}
pub struct UncondBr {
    pub common: MachineInstCommonBase,
    pub oprnds: [MachineOperand; 1],
}
pub struct BLink {
    pub common: MachineInstCommonBase,
    pub oprnds: [MachineOperand; 2],
}
pub struct BrRegCond {
    pub common: MachineInstCommonBase,
    pub oprnds: [MachineOperand; 2],
}

impl CondBr {
    pub fn get_label(&self) -> &MachineOperand {
        &self.oprnds[0]
    }
    pub fn get_implicit_pstate(&self) -> &MachineOperand {
        &self.oprnds[1]
    }
}
impl BLink {
    pub fn get_target(&self) -> &MachineOperand {
        &self.oprnds[0]
    }
    pub fn get_link_reg(&self) -> &MachineOperand {
        &self.oprnds[1]
    }
}
impl BrRegCond {
    pub fn get_cond_reg(&self) -> &MachineOperand {
        &self.oprnds[0]
    }
    pub fn get_target(&self) -> &MachineOperand {
        &self.oprnds[1]
    }

    pub fn is_cbnz(&self) -> bool { matches!(self.common.opcode, AArch64OP::CBNZ) }
    pub fn is_cbz (&self) -> bool { matches!(self.common.opcode, AArch64OP::CBZ) }
    pub fn is_tbnz(&self) -> bool { matches!(self.common.opcode, AArch64OP::TBNZ) }
    pub fn is_tbz (&self) -> bool { matches!(self.common.opcode, AArch64OP::TBZ) }
}