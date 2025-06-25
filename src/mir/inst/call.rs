use std::cell::Cell;

use crate::mir::{
    inst::{MirInstCommon, opcode::MirOP},
    operand::MirOperand,
};

/// Call pesudo instruction.
///
/// MIR syntax:
///
/// - `call <func-name>, %arg0, %arg1, ...`
#[derive(Debug, Clone)]
pub struct MirCall {
    pub(super) common: MirInstCommon,
    pub operands: Vec<Cell<MirOperand>>,
}

impl MirCall {
    pub fn new(callee: MirOperand, args: &[MirOperand]) -> Self {
        let mut operands = vec![Cell::new(callee)];
        operands.extend(args.iter().map(|x| Cell::new(x.clone())));
        Self {
            common: MirInstCommon::new(MirOP::Call),
            operands,
        }
    }

    pub fn callee(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn args(&self) -> &[Cell<MirOperand>] {
        &self.operands[1..]
    }
}
