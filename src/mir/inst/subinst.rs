use std::cell::Cell;

use crate::mir::{
    inst::{MirInstCommon, inst::MirInst, opcode::MirOP},
    operand::MirOperand,
};

pub trait IMirSubInst {
    fn get_common(&self) -> &MirInstCommon;
    fn common_mut(&mut self) -> &mut MirInstCommon;

    fn get_opcode(&self) -> MirOP {
        self.get_common().opcode
    }
    fn opcode_is(&self, opcode: MirOP) -> bool {
        self.get_common().opcode == opcode
    }

    fn out_operands(&self) -> &[Cell<MirOperand>];
    fn in_operands(&self) -> &[Cell<MirOperand>];

    fn accepts_opcode(opcode: MirOP) -> bool;

    fn new_empty(opcode: MirOP) -> Self;

    fn is_pseudo(&self) -> bool {
        false
    }

    fn from_mir(mir_inst: &MirInst) -> Option<&Self>;
    fn into_mir(self) -> MirInst;
}
