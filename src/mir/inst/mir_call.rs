use crate::mir::{
    fmt::FuncFormatContext,
    inst::{IMirSubInst, MirInstCommon, inst::MirInst, opcode::MirOP},
    module::func::MirFunc,
    operand::{
        IMirSubOperand, MirOperand,
        reg::{RegID, RegOperand, RegUseFlags, SubRegIndex},
    },
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
    saved_regs: Cell<MirCallerSavedRegs>,
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
            saved_regs: Cell::new(MirCallerSavedRegs::new_aapcs()),
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
            saved_regs: Cell::new(MirCallerSavedRegs::new_aapcs()),
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

    pub fn get_saved_regs(&self) -> MirCallerSavedRegs {
        self.saved_regs.get()
    }
    pub fn set_saved_regs(&self, saved_regs: MirCallerSavedRegs) {
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
        self.set_saved_regs(MirCallerSavedRegs::new_aapcs());
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
            saved_regs: Cell::new(MirCallerSavedRegs::new_aapcs()),
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

#[derive(Debug, Clone, Copy)]
pub struct MirCallerSavedRegs {
    pub gpr_bitset: u32,
    pub fpr_bitset: u32,
}

impl MirCallerSavedRegs {
    pub fn new_empty() -> Self {
        MirCallerSavedRegs {
            gpr_bitset: 0,
            fpr_bitset: 0,
        }
    }

    /// 根据 AAPCS64 ABI, 调用者需要保存的寄存器包括:
    ///
    /// #### GPRs
    ///
    /// - x0-x7: 可以用于传参/传递返回值
    /// - x8: 间接结果位置寄存器
    /// - x9-x15: 调用者保存的临时变量寄存器
    /// - x16,x17,x18: 平台特定寄存器
    /// - x30: 返回地址寄存器 -- 一般来说, 当函数中有 call 时, x30 在函数入口点就会保存。
    ///
    /// #### FPRs
    ///
    /// - v0-v7: 可以用于传参/传递返回值
    /// - v16-v31: 调用者保存的临时变量寄存器
    pub const fn new_aapcs() -> Self {
        MirCallerSavedRegs {
            gpr_bitset: 0b01000000_00000111_11111111_11111111,
            fpr_bitset: 0b11111111_11111111_00000000_11111111,
        }
    }

    /// 尝试保存一个通用寄存器.
    /// 如果寄存器已经被保存或者寄存器 ID 是虚的, 则返回 `false`。否则保存寄存器并返回 `true` .
    pub const fn save_gpr(&mut self, pos: RegID) -> bool {
        let RegID::Phys(id) = pos else {
            return false; // 不能保存虚寄存器
        };
        if self.gpr_bitset & (1 << id) != 0 {
            return false; // 寄存器已经被保存
        }
        self.gpr_bitset |= 1 << id;
        true
    }

    /// 尝试保存一个浮点寄存器.
    /// 如果寄存器已经被保存或者寄存器 ID 是虚的,
    /// 则返回 `false`。否则保存寄存器并返回 `true`
    pub const fn save_fpr(&mut self, pos: RegID) -> bool {
        let RegID::Phys(id) = pos else {
            return false; // 不能保存虚寄存器
        };
        if self.fpr_bitset & (1 << id) != 0 {
            return false; // 寄存器已经被保存
        }
        self.fpr_bitset |= 1 << id;
        true
    }

    pub const fn save_reg(&mut self, reg_operand: RegOperand) -> bool {
        let RegOperand(id, _, _, is_fp) = reg_operand;
        if is_fp {
            self.save_fpr(RegID::Phys(id))
        } else {
            self.save_gpr(RegID::Phys(id))
        }
    }

    pub const fn unsave_gpr(&mut self, pos: RegID) -> bool {
        let RegID::Phys(id) = pos else {
            return false; // 不能取消保存虚寄存器
        };
        if self.gpr_bitset & (1 << id) == 0 {
            return false; // 寄存器没有被保存
        }
        self.gpr_bitset &= !(1 << id);
        true
    }
    pub const fn unsave_fpr(&mut self, pos: RegID) -> bool {
        let RegID::Phys(id) = pos else {
            return false; // 不能取消保存虚寄存器
        };
        if self.fpr_bitset & (1 << id) == 0 {
            return false; // 寄存器没有被保存
        }
        self.fpr_bitset &= !(1 << id);
        true
    }
    pub const fn unsave_reg(&mut self, reg_operand: RegOperand) -> bool {
        let RegOperand(id, _, _, is_fp) = reg_operand;
        if is_fp {
            self.unsave_fpr(RegID::Phys(id))
        } else {
            self.unsave_gpr(RegID::Phys(id))
        }
    }

    pub const fn insert_saved_gpr(mut self, pos: RegID) -> Self {
        self.save_gpr(pos);
        self
    }
    pub const fn insert_saved_fpr(mut self, pos: RegID) -> Self {
        self.save_fpr(pos);
        self
    }
    pub const fn insert_saved_reg(mut self, reg_operand: RegOperand) -> Self {
        self.save_reg(reg_operand);
        self
    }

    pub const fn has_saved_gpr(&self, pos: RegID) -> bool {
        let RegID::Phys(id) = pos else {
            return false; // 不能检查虚寄存器
        };
        self.gpr_bitset & (1 << id) != 0
    }
    pub const fn has_saved_fpr(&self, pos: RegID) -> bool {
        let RegID::Phys(id) = pos else {
            return false; // 不能检查虚寄存器
        };
        self.fpr_bitset & (1 << id) != 0
    }
    pub const fn has_saved_reg(&self, reg_operand: RegOperand) -> bool {
        let RegOperand(id, _, _, is_fp) = reg_operand;
        if is_fp {
            self.has_saved_fpr(RegID::Phys(id))
        } else {
            self.has_saved_gpr(RegID::Phys(id))
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.gpr_bitset == 0 && self.fpr_bitset == 0
    }
    pub const fn num_gprs(&self) -> u32 {
        self.gpr_bitset.count_ones()
    }
    pub const fn num_fprs(&self) -> u32 {
        self.fpr_bitset.count_ones()
    }
    pub const fn num_regs(&self) -> u32 {
        self.num_gprs() + self.num_fprs()
    }

    pub const fn iter(self) -> MirCallSavedRegIter {
        MirCallSavedRegIter {
            gpr_bitset: self.gpr_bitset,
            fpr_bitset: self.fpr_bitset,
            index: 0,
        }
    }
    pub fn dump_regs(&self) -> Vec<RegOperand> {
        self.iter().collect()
    }
}

impl IntoIterator for MirCallerSavedRegs {
    type Item = RegOperand;
    type IntoIter = MirCallSavedRegIter;

    fn into_iter(self) -> MirCallSavedRegIter {
        MirCallSavedRegIter {
            gpr_bitset: self.gpr_bitset,
            fpr_bitset: self.fpr_bitset,
            index: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MirCallSavedRegIter {
    pub gpr_bitset: u32,
    pub fpr_bitset: u32,
    pub index: u8,
}

impl Iterator for MirCallSavedRegIter {
    type Item = RegOperand;

    fn next(&mut self) -> Option<RegOperand> {
        while self.index < 64 {
            let (bitset, is_fp, id) = if self.index < 32 {
                (self.gpr_bitset, false, self.index)
            } else {
                (self.fpr_bitset, true, self.index - 32)
            };
            self.index += 1;

            if bitset & (1 << id) != 0 {
                let sub_index = SubRegIndex::new(6, 0);
                let reg_operand = RegOperand(id as u32, sub_index, RegUseFlags::KILL, is_fp);
                return Some(reg_operand);
            }
        }
        None // 超出寄存器范围或没有更多寄存器
    }
}
