use std::{
    cell::{Cell, RefCell},
    fmt::Write,
    rc::Rc,
};

use crate::mir::{
    fmt::FuncFormatContext,
    inst::{IMirSubInst, MirInstCommon, inst::MirInst, opcode::MirOP},
    module::func::MirFunc,
    operand::{IMirSubOperand, MirOperand},
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
    callee_func: RefCell<Option<Rc<MirFunc>>>,
}

impl MirCall {
    pub fn new(callee: MirOperand, args: &[MirOperand]) -> Self {
        let mut operands = vec![Cell::new(callee)];
        operands.extend(args.iter().map(|x| Cell::new(x.clone())));
        Self {
            common: MirInstCommon::new(MirOP::MirCall),
            operands,
            callee_func: RefCell::new(None),
        }
    }
    pub fn get_callee_func(&self) -> Option<Rc<MirFunc>> {
        self.callee_func.borrow().clone()
    }
    pub fn set_callee_func(&self, func: Rc<MirFunc>) {
        self.callee_func.replace(Some(func));
    }
    pub fn callee(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }

    pub fn args(&self) -> &[Cell<MirOperand>] {
        &self.operands[1..]
    }

    pub fn fmt_asm(&self, formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        write!(formatter, "mir.call ")?;
        let callee = self.callee().get();
        if let MirOperand::Global(global_ref) = callee {
            global_ref.fmt_asm(formatter)?;
        } else {
            return Err(std::fmt::Error);
        }
        for (i, arg) in self.args().iter().enumerate() {
            if i != 0 {
                formatter.write_str(", ")?;
            }
            arg.get().fmt_asm(formatter)?;
        }
        Ok(())
    }
}

impl IMirSubInst for MirCall {
    fn get_common(&self) -> &MirInstCommon {
        &self.common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &[]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self.operands
    }

    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirCall)
    }
    fn new_empty(_: MirOP) -> Self {
        Self {
            common: MirInstCommon::new(MirOP::MirCall),
            operands: Vec::new(),
            callee_func: RefCell::new(None),
        }
    }

    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirCall(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirCall(self)
    }
}

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
        if self.has_retval() {
            Some(&self.operands_storage[0])
        } else {
            None
        }
    }
    pub fn operands(&self) -> &[Cell<MirOperand>] {
        if self.has_retval() {
            &self.operands_storage[..1]
        } else {
            &[]
        }
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

pub use super::switch::MirSwitch;
