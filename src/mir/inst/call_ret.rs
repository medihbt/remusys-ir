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

#[derive(Debug, Clone)]
pub struct MirReturn {
    pub common: MirInstCommon,
    operands_storj: [Cell<MirOperand>; 1],
    has_retval: bool,
}

impl MirReturn {
    pub fn new(has_retval: bool) -> Self {
        Self {
            common: MirInstCommon::new(MirOP::MirReturn),
            operands_storj: [Cell::new(MirOperand::None)],
            has_retval,
        }
    }

    pub fn set_retval(&mut self, retval: MirOperand) {
        self.operands_storj[0].set(retval);
        self.has_retval = true;
    }

    pub fn has_retval(&self) -> bool {
        self.has_retval
    }

    pub fn retval(&self) -> Option<&Cell<MirOperand>> {
        if self.has_retval {
            Some(&self.operands_storj[0])
        } else {
            None
        }
    }
    pub fn operands(&self) -> &[Cell<MirOperand>] {
        if self.has_retval {
            &self.operands_storj[..1]
        } else {
            &[]
        }
    }
}
