use std::cell::Cell;

use crate::mir::{
    inst::{MirInstCommon, opcode::MirOP},
    operand::MirOperand,
};

pub trait IMirSubInst {
    fn get_common(&self) -> &MirInstCommon;

    fn out_operands(&self) -> &[Cell<MirOperand>];
    fn in_operands(&self) -> &[Cell<MirOperand>];

    fn accepts_opcode(opcode: MirOP) -> bool;

    fn new_empty(opcode: MirOP) -> Self;

    fn is_pseudo(&self) -> bool {
        false
    }
}
