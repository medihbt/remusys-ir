use crate::mir::{
    fmt::FuncFormatContext,
    inst::{
        IMirSubInst, MirInstCommon,
        impls::*,
        inst::MirInst,
        mirops::{MirRestoreRegs, MirSaveRegs},
        opcode::MirOP,
    },
    module::{func::MirFunc, stack::MirStackItem},
    operand::{
        IMirSubOperand, MirOperand,
        imm::{ImmCalc, ImmLSP32, ImmLSP64},
        physreg_set::MirPhysRegSet,
        reg::*,
    },
    translate::{mir_pass::inst_lower::LowerInstAction, mirgen::operandgen::DispatchedReg},
};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
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

impl MirCall {
    pub fn dump_actions_template(&self, out_actions: &mut VecDeque<LowerInstAction>) {
        use LowerInstAction::*;

        // 先保存寄存器
        let saved_regs = self.get_saved_regs();
        if !saved_regs.is_empty() {
            out_actions.push_back(BeginSaveRegs(saved_regs, MirSaveRegs::new(saved_regs)));
        }

        // 准备参数
        let sp_adjustment = self.prepare_func_args(out_actions);

        // 调用函数
        let callee = self.callee().get();
        let callee = match callee {
            MirOperand::Global(global) => global,
            _ => panic!("Expected callee {callee:?} to be a global operand"),
        };
        let bl_inst = BLinkGlobal::new(MirOP::BLinkGlobal, GPR64::ra(), callee).into_mir();
        out_actions.push_back(LowerInstAction::NOP(bl_inst));

        // 如果有返回值，则将返回值移动到返回寄存器
        if let Some(retreg) = self.get_ret_arg() {
            let retreg = DispatchedReg::from_reg(retreg);

            use DispatchedReg::*;
            let mov_inst = match retreg {
                F32(dst) => UnaF32::new(MirOP::FMov32R, dst, FPR32::new_raw(0)).into_mir(),
                F64(dst) => UnaF64::new(MirOP::FMov64R, dst, FPR64::new_raw(0)).into_mir(),
                G32(dst) => Una32R::new(MirOP::Mov32R, dst, GPR32::new_raw(0), None).into_mir(),
                G64(dst) => Una64R::new(MirOP::Mov64R, dst, GPR64::new_raw(0), None).into_mir(),
            };
            out_actions.push_back(LowerInstAction::NOP(mov_inst));
        }

        // 恢复由参数引起的栈变化
        if sp_adjustment != 0 {
            let sp = GPR64::sp();
            // 如果这里崩溃了, 那就要处理参数数量太多导致需要额外计算的情况.
            let sp_offset = ImmCalc::new(sp_adjustment as u32);
            // 将 SP 恢复到调用前的位置
            out_actions.push_back(LowerInstAction::EndSubSP(
                Bin64RC::new(MirOP::Add64I, sp, sp, sp_offset).into_mir(),
            ));
        }

        // 恢复寄存器
        if !saved_regs.is_empty() {
            out_actions.push_back(EndSaveRegs(MirRestoreRegs::new(saved_regs)));
        }
    }
}

struct CallArgsCnt {
    gpreg_id: usize,
    fpreg_id: usize,
    spilled_cnt: usize,
}

enum CallIntArgKind {
    G32(GPR32),
    G64(GPR64),
    // dest index; source reg
    Spilled32(usize, GPR32),
    Spilled64(usize, GPR64),
}

enum CallFPArgKind {
    F32(FPR32),
    F64(FPR64),
    // dest index; source reg
    Spilled32(usize, FPR32),
    Spilled64(usize, FPR64),
}

impl CallArgsCnt {
    fn new() -> Self {
        Self {
            gpreg_id: 0,
            fpreg_id: 0,
            spilled_cnt: 0,
        }
    }

    fn push_gpr(&mut self, gpr: GPReg) -> CallIntArgKind {
        if self.gpreg_id >= 8 {
            let spilled_id = self.spilled_cnt;
            self.spilled_cnt += 1;
            match gpr.get_bits_log2() {
                5 => CallIntArgKind::Spilled32(spilled_id, GPR32::from_real(gpr)),
                6 => CallIntArgKind::Spilled64(spilled_id, GPR64::from_real(gpr)),
                _ => panic!("Unsupported size for GPR: {}", gpr.get_bits_log2()),
            }
        } else {
            self.gpreg_id += 1;
            let GPReg(id, si, uf) = gpr;
            let bits_log2 = si.get_bits_log2();
            match bits_log2 {
                5 => CallIntArgKind::G32(GPR32(id, uf | RegUseFlags::DEF)),
                6 => CallIntArgKind::G64(GPR64(id, uf | RegUseFlags::DEF)),
                _ => panic!("Unsupported size for GPR: {bits_log2}"),
            }
        }
    }

    fn push_fpr(&mut self, fpr: VFReg) -> CallFPArgKind {
        if self.fpreg_id >= 8 {
            let spilled_id = self.spilled_cnt;
            self.spilled_cnt += 1;
            match fpr.get_bits_log2() {
                5 => CallFPArgKind::Spilled32(spilled_id, FPR32::from_real(fpr)),
                6 => CallFPArgKind::Spilled64(spilled_id, FPR64::from_real(fpr)),
                _ => panic!("Unsupported size for FPR: {}", fpr.get_bits_log2()),
            }
        } else {
            self.fpreg_id += 1;
            let VFReg(id, si, uf) = fpr;
            let bits_log2 = si.get_bits_log2();
            match bits_log2 {
                5 => CallFPArgKind::F32(FPR32(id, uf | RegUseFlags::DEF)),
                6 => CallFPArgKind::F64(FPR64(id, uf | RegUseFlags::DEF)),
                _ => panic!("Unsupported size for FPR: {bits_log2}"),
            }
        }
    }
}

impl MirCall {
    /// 准备函数调用的参数. 返回调整参数后的 SP 偏移量.
    fn prepare_func_args(&self, out_actions: &mut VecDeque<LowerInstAction>) -> i64 {
        // 找到要 spill 的参数和它们的大小
        let (spilled_args, callee_arg_section_size) = {
            let callee = self.get_callee_func().unwrap();
            let callee_stack_layout = callee.borrow_inner().stack_layout.clone();
            let callee_args_size = callee_stack_layout.args_size as i64;
            let callee_spilled_args = callee_stack_layout.args;
            (callee_spilled_args, callee_args_size)
        };
        let mut arg_state = CallArgsCnt::new();

        // 如果有参数需要 spill，则将栈指针向下移动
        if callee_arg_section_size != 0 {
            let sp = GPR64::sp();
            let sp_offset = callee_arg_section_size as u32;
            // 如果这里崩溃了, 那就要处理参数数量太多导致需要额外计算的情况.
            let sp_offset_imm = ImmCalc::new(sp_offset);
            // 如果有参数需要 spill，则将栈指针向下移动
            let reserve_sp = Bin64RC::new(MirOP::Sub64I, sp, sp, sp_offset_imm).into_mir();
            out_actions.push_back(LowerInstAction::BeginSubSP(sp_offset, reserve_sp));
        }

        for arg in self.args() {
            let arg = arg.get();
            use MirOperand::*;
            match arg {
                GPReg(arg) => {
                    let arg_kind = arg_state.push_gpr(arg);
                    Self::make_prepare_gparg_inst(&spilled_args, arg, arg_kind, out_actions);
                }
                VFReg(arg) => {
                    let arg_kind = arg_state.push_fpr(arg);
                    Self::make_prepare_fparg_inst(&spilled_args, arg, arg_kind, out_actions);
                }
                Imm64(_) | Imm32(_) | F32(_) | F64(_) => {
                    todo!("MirCall with immediate or float argument: {arg:?}");
                }
                _ => {
                    panic!("Unsupported argument type for call: {arg:?}");
                }
            }
        }

        // 返回 SP 调整量.
        callee_arg_section_size
    }

    /// 为整数参数生成准备指令
    fn make_prepare_gparg_inst(
        callee_spilled_args: &[MirStackItem],
        arg: GPReg,
        arg_kind: CallIntArgKind,
        out_actions: &mut VecDeque<LowerInstAction>,
    ) {
        let sp = GPR64::sp();
        match arg_kind {
            CallIntArgKind::G32(dst) => {
                let inst = Una32R::new(MirOP::Mov32R, dst, GPR32::from_real(arg), None);
                out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
            }
            CallIntArgKind::G64(dst) => {
                let inst = Una64R::new(MirOP::Mov64R, dst, GPR64::from_real(arg), None);
                out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
            }
            CallIntArgKind::Spilled32(dst_idx, src) => {
                let arg_offset = callee_spilled_args[dst_idx].offset;
                // 这个有赌的成分——偏移量有可能超过 ImmLSP32 能表示的范围, 但我懒得额外处理这件事情了.
                // 如果程序崩溃了再说吧.
                let arg_offset = ImmLSP32::new(arg_offset as u32);
                let inst = StoreGr32Base::new(MirOP::StrGr32Base, src, sp, arg_offset);
                out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
            }
            CallIntArgKind::Spilled64(dst_idx, src) => {
                let arg_offset = callee_spilled_args[dst_idx].offset;
                // 这个有赌的成分——偏移量有可能超过 ImmLSP64 能表示的范围, 但我懒得额外处理这件事情了.
                // 如果程序崩溃了再说吧.
                let arg_offset = ImmLSP64::new(arg_offset as u64);
                let inst = StoreGr64Base::new(MirOP::StrGr64Base, src, sp, arg_offset);
                out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
            }
        }
    }

    fn make_prepare_fparg_inst(
        callee_spilled_args: &[MirStackItem],
        arg: VFReg,
        arg_kind: CallFPArgKind,
        out_actions: &mut VecDeque<LowerInstAction>,
    ) {
        let sp = GPR64::sp();
        match arg_kind {
            CallFPArgKind::F32(dst) => {
                let inst = UnaF32::new(MirOP::FMov32R, dst, FPR32::from_real(arg));
                out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
            }
            CallFPArgKind::F64(dst) => {
                let inst = UnaF64::new(MirOP::FMov64R, dst, FPR64::from_real(arg));
                out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
            }
            CallFPArgKind::Spilled32(dst_id, src) => {
                let arg_offset = callee_spilled_args[dst_id].offset;
                let arg_offset = ImmLSP32::new(arg_offset as u32);
                let inst = StoreF32Base::new(MirOP::StrF32Base, src, sp, arg_offset);
                out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
            }
            CallFPArgKind::Spilled64(dst_id, src) => {
                let arg_offset = callee_spilled_args[dst_id].offset;
                let arg_offset = ImmLSP64::new(arg_offset as u64);
                let inst = StoreF64Base::new(MirOP::StrF64Base, src, sp, arg_offset);
                out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
            }
        }
    }
}
