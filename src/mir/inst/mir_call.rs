use crate::mir::{
    fmt::FuncFormatContext,
    inst::{IMirSubInst, MirInstCommon, inst::MirInst, opcode::MirOP},
    module::func::MirFunc,
    operand::{IMirSubOperand, MirOperand, physreg_set::MirPhysRegSet, reg::RegOperand},
};
use std::{
    cell::{Cell, RefCell},
    fmt::Write,
    rc::Rc,
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
    saved_regs: Cell<MirPhysRegSet>,
}

impl MirCall {
    pub fn with_retreg(callee: MirOperand, ret_arg: RegOperand, args: &[MirOperand]) -> Self {
        let mut operands = vec![Cell::new(callee)];
        operands.push(Cell::new(ret_arg.into()));
        operands.extend(args.iter().map(|x| Cell::new(x.clone())));
        Self {
            common: MirInstCommon::new(MirOP::MirCall),
            operands,
            callee_func: RefCell::new(None),
            saved_regs: Cell::new(MirPhysRegSet::new_aapcs_caller()),
        }
    }
    pub fn with_return_void(callee: MirOperand, args: &[MirOperand]) -> Self {
        let mut operands = vec![Cell::new(callee)];
        operands.push(Cell::new(MirOperand::None));
        operands.extend(args.iter().map(|x| Cell::new(x.clone())));
        Self {
            common: MirInstCommon::new(MirOP::MirCall),
            operands,
            callee_func: RefCell::new(None),
            saved_regs: Cell::new(MirPhysRegSet::new_aapcs_caller()),
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

    pub fn ret_arg(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
    pub fn get_ret_arg(&self) -> Option<RegOperand> {
        match self.ret_arg().get() {
            MirOperand::GPReg(reg) => Some(RegOperand::from(reg)),
            MirOperand::VFReg(reg) => Some(RegOperand::from(reg)),
            MirOperand::None => None,
            _ => panic!("Expected return argument to be a register operand"),
        }
    }
    pub fn set_ret_arg(&self, ret_arg: RegOperand) {
        self.operands[1].set(ret_arg.into());
    }
    pub fn has_retval(&self) -> bool {
        !matches!(self.ret_arg().get(), MirOperand::None)
    }

    pub fn args(&self) -> &[Cell<MirOperand>] {
        &self.operands[2..]
    }

    pub fn get_saved_regs(&self) -> MirPhysRegSet {
        self.saved_regs.get()
    }
    pub fn set_saved_regs(&self, saved_regs: MirPhysRegSet) {
        self.saved_regs.set(saved_regs);
    }
    pub fn add_saved_reg<T>(&self, reg_operand: T)
    where
        RegOperand: From<T>,
    {
        let mut saved_regs = self.saved_regs.get();
        if saved_regs.save_reg(RegOperand::from(reg_operand)) {
            self.saved_regs.set(saved_regs);
        }
    }
    pub fn remove_saved_reg<T>(&self, reg_operand: T)
    where
        RegOperand: From<T>,
    {
        let mut saved_regs = self.saved_regs.get();
        if saved_regs.unsave_reg(RegOperand::from(reg_operand)) {
            self.saved_regs.set(saved_regs);
        }
    }
    pub fn restore_saved_args_to_aapcs(&self) {
        self.set_saved_regs(MirPhysRegSet::new_aapcs_caller());
    }
    pub fn has_saved_reg<T>(&self, reg_operand: T) -> bool
    where
        RegOperand: From<T>,
    {
        self.saved_regs
            .get()
            .has_saved_reg(RegOperand::from(reg_operand))
    }

    pub fn fmt_asm(&self, formatter: &mut FuncFormatContext) -> std::fmt::Result {
        write!(formatter, "mir.call @")?;
        let callee = self.callee().get();
        if let MirOperand::Global(global_ref) = callee {
            global_ref.fmt_asm(formatter)?;
        } else {
            return Err(std::fmt::Error);
        }
        formatter.write_str(" into %")?;
        self.ret_arg().get().fmt_asm(formatter)?;
        formatter.write_str(" with args (")?;
        for (i, arg) in self.args().iter().enumerate() {
            if i != 0 {
                formatter.write_str(", ")?;
            }
            arg.get().fmt_asm(formatter)?;
        }
        formatter.write_str(")")?;
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
            saved_regs: Cell::new(MirPhysRegSet::new_aapcs_caller()),
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
