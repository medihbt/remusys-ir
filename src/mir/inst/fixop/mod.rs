use std::cell::Cell;

use crate::{
    base::slablist::SlabRefListNodeHead,
    mir::{
        inst::{MachineInstCommonBase, opcode::AArch64OP},
        operand::MachineOperand,
    },
};

pub mod branch;
pub mod data_process;
pub mod load_store;

pub struct FixOPInst {
    pub common: MachineInstCommonBase,
    operand_arr: [MachineOperand; 6],
    operand_len: usize,
}

impl FixOPInst {
    pub fn new(opcode: AArch64OP, noperands: usize) -> Self {
        assert!(noperands <= 6, "Too many operands for FixedOPsInst");
        Self {
            common: MachineInstCommonBase {
                self_head: Cell::new(SlabRefListNodeHead::new()),
                opcode,
            },
            operand_len: noperands,
            operand_arr: [MachineOperand::None; 6],
        }
    }

    pub fn get_operands(&self) -> &[MachineOperand] {
        &self.operand_arr[..self.operand_len]
    }
    pub fn operands_mut(&mut self) -> &mut [MachineOperand] {
        &mut self.operand_arr[..self.operand_len]
    }
    pub fn get_noperands(&self) -> usize {
        self.operand_len
    }
    pub fn get_opcode(&self) -> AArch64OP {
        self.common.opcode
    }
}
