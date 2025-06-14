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

#[derive(Debug, Clone)]
pub struct FixOPInst {
    pub common: MachineInstCommonBase,
    operand_arr: [Cell<MachineOperand>; 6],
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
            operand_arr: [const { Cell::new(MachineOperand::None) }; 6],
        }
    }

    pub fn operands(&self) -> &[Cell<MachineOperand>] {
        &self.operand_arr[..self.operand_len]
    }
    pub fn get_noperands(&self) -> usize {
        self.operand_len
    }
    pub fn get_opcode(&self) -> AArch64OP {
        self.common.opcode
    }
}
