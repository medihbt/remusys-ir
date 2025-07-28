use crate::mir::{
    fmt::FuncFormatContext,
    inst::{IMirSubInst, MirInstCommon, inst::MirInst, opcode::MirOP},
    operand::{IMirSubOperand, MirOperand},
};
use std::{cell::Cell, fmt::Write};

#[derive(Debug, Clone)]
pub struct MirReturn {
    pub common: MirInstCommon,
    operands_storage: [Cell<MirOperand>; 1],
    has_retval: Cell<bool>,
}

impl MirReturn {
    pub fn new(has_retval: bool) -> Self {
        Self {
            common: MirInstCommon::new(MirOP::MirReturn),
            operands_storage: [Cell::new(MirOperand::None)],
            has_retval: Cell::new(has_retval),
        }
    }

    pub fn set_retval(&self, retval: MirOperand) {
        self.operands_storage[0].set(retval);
        self.has_retval.set(true);
    }

    pub fn has_retval(&self) -> bool {
        self.has_retval.get()
    }

    pub fn retval(&self) -> Option<&Cell<MirOperand>> {
        if self.has_retval() { Some(&self.operands_storage[0]) } else { None }
    }
    pub fn operands(&self) -> &[Cell<MirOperand>] {
        if self.has_retval() { &self.operands_storage[..1] } else { &[] }
    }

    pub fn fmt_asm(&self, formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        if let Some(retval) = self.retval() {
            write!(formatter, "mir.return ")?;
            retval.get().fmt_asm(formatter)?;
        } else {
            write!(formatter, "mir.return")?;
        }
        Ok(())
    }
}

impl IMirSubInst for MirReturn {
    fn get_common(&self) -> &MirInstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self.common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &[]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        self.operands()
    }

    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirReturn)
    }
    fn new_empty(_: MirOP) -> Self {
        Self::new(false)
    }

    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirReturn(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirReturn(self)
    }
}

pub use super::mir_call::MirCall;
pub use super::reg_restore::{MirRestoreHostRegs, MirRestoreRegs};
pub use super::reg_save::MirSaveRegs;
pub use super::switch::MirSwitch;
