use crate::mir::{
    fmt::FuncFormatContext,
    inst::{IMirSubInst, MirInstCommon, impls::*, inst::MirInst, opcode::MirOP},
    module::stack::MirStackLayout,
    operand::{
        IMirSubOperand, MirOperand,
        imm::{Imm64, ImmCalc, ImmKind, ImmLSP32, ImmLSP64},
        physreg_set::{MirPhysRegSet, RegOperandSet},
        reg::{FPR64, GPR64, GPReg, RegID, RegOperand, RegUseFlags, VFReg},
    },
    translate::mirgen::operandgen::DispatchedReg,
};
use std::{
    cell::Cell,
    collections::VecDeque,
    fmt::{Debug, Write},
};

#[derive(Clone)]
pub struct MirRestoreRegs {
    _common: MirInstCommon,
    saved_regs: Cell<MirPhysRegSet>,
    operands: RegOperandSet,
}

impl Debug for MirRestoreRegs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let regs = self.saved_regs.get();

        match regs.num_regs() {
            0 => f
                .debug_struct("MirRestoreRegs")
                .field("<-->", &(prev, next))
                .field("regs", &"[]")
                .finish(),
            1..=10 => f
                .debug_struct("MirRestoreRegs")
                .field("<-->", &(prev, next))
                .field("regs", &regs.dump_regs())
                .finish(),
            x => f
                .debug_struct("MirRestoreRegs")
                .field("<-->", &(prev, next))
                .field("regs", &regs)
                .field("n_regs", &x)
                .finish(),
        }
    }
}

impl IMirSubInst for MirRestoreRegs {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        self.operands.update(self.get_saved_regs());
        self.operands.operands()
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &[]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirSaveRegs)
    }
    fn new_empty(opcode: MirOP) -> Self {
        MirRestoreRegs {
            _common: MirInstCommon::new(opcode),
            saved_regs: Cell::new(MirPhysRegSet::new_empty()),
            operands: RegOperandSet::new(RegUseFlags::DEF),
        }
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        let MirInst::MirRestoreRegs(inst) = mir_inst else {
            return None;
        };
        Some(inst)
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirRestoreRegs(self)
    }
}

impl MirRestoreRegs {
    pub fn new(saved_regs: MirPhysRegSet) -> Self {
        MirRestoreRegs {
            _common: MirInstCommon::new(MirOP::MirSaveRegs),
            saved_regs: Cell::new(saved_regs),
            operands: RegOperandSet::new(RegUseFlags::DEF),
        }
    }

    pub fn new_aapcs_callee() -> Self {
        MirRestoreRegs {
            _common: MirInstCommon::new(MirOP::MirSaveRegs),
            saved_regs: Cell::new(MirPhysRegSet::new_aapcs_callee()),
            operands: RegOperandSet::new(RegUseFlags::DEF),
        }
    }

    pub fn get_saved_regs(&self) -> MirPhysRegSet {
        self.saved_regs.get()
    }
    pub fn set_saved_regs(&self, saved_regs: MirPhysRegSet) {
        self.saved_regs.set(saved_regs);
    }

    pub fn add_saved_gpreg_id(&self, reg_id: u32) {
        assert!(reg_id < 32, "Invalid GPR ID: {reg_id}");
        let saved_regs = self.saved_regs.get();
        self.saved_regs
            .set(saved_regs.insert_saved_gpr(RegID::Phys(reg_id)));
    }
    pub fn add_saved_fpreg_id(&self, reg_id: u32) {
        assert!(reg_id < 32, "Invalid FPR ID: {reg_id}");
        let saved_regs = self.saved_regs.get();
        self.saved_regs
            .set(saved_regs.insert_saved_fpr(RegID::Phys(reg_id)));
    }
    pub fn add_saved_gpreg(&self, reg: GPReg) {
        self.add_saved_gpreg_id(reg.get_id_raw());
    }
    pub fn add_saved_fpreg(&self, reg: VFReg) {
        self.add_saved_fpreg_id(reg.get_id_raw());
    }

    /// MIR pseudo-assembly syntax: `mir.restoreregs <reg1, reg2, ...>`
    pub fn fmt_asm(&self, formatter: &mut FuncFormatContext) -> std::fmt::Result {
        write!(formatter, "mir.restoreregs ")?;
        let saved_regs = self.saved_regs.get();
        for reg in saved_regs.iter() {
            match DispatchedReg::from_reg(reg) {
                DispatchedReg::F32(fpr32) => fpr32.fmt_asm(formatter)?,
                DispatchedReg::F64(fpr64) => fpr64.fmt_asm(formatter)?,
                DispatchedReg::G32(gpr32) => gpr32.fmt_asm(formatter)?,
                DispatchedReg::G64(gpr64) => gpr64.fmt_asm(formatter)?,
            }
            write!(formatter, ", ")?;
        }
        Ok(())
    }

    /// 把该指令转换为恢复寄存器的动作模板.
    ///
    /// 主要步骤包括:
    ///
    /// * 如果自己是空的, 就什么都不做.
    /// * 对每个寄存器, 生成对应的 ldr 指令, 把寄存器的值从栈上恢复.
    /// * 计算自己被恢复的寄存器数量, 按照每个寄存器 8 字节的大小计算出寄存器保存消耗的栈空间
    /// * 按 16 字节对齐的原则计算出实际的栈空间, 最后给出 add sp, sp, <size> 的指令.
    pub fn dump_actions_template(&self, out_insts: &mut VecDeque<MirInst>) {
        let saved_regs = self.get_saved_regs();
        let num_regs = saved_regs.num_regs();
        if num_regs == 0 {
            return;
        }

        // 先恢复所有寄存器
        for (index, reg) in saved_regs.iter().enumerate() {
            let RegOperand(id, _, _, is_fp) = reg;
            let reg_id = RegID::from_real(id);
            let offset = ImmLSP64::new(index as u64 * 8);
            let sp = GPR64::sp();

            let ldr_inst = if is_fp {
                let dst = FPR64::new(reg_id);
                LoadF64Base::new(MirOP::LdrF64Base, dst, sp, offset).into_mir()
            } else {
                let dst = GPR64::new(reg_id);
                LoadGr64Base::new(MirOP::LdrGr64Base, dst, sp, offset).into_mir()
            };
            out_insts.push_back(ldr_inst);
        }

        // 然后释放栈空间
        let size = (num_regs * 8).next_multiple_of(16);
        let sp = GPR64::sp();
        let add_sp = Bin64RC::new(MirOP::Add64I, sp, sp, ImmCalc::new(size));
        out_insts.push_back(MirInst::Bin64RC(add_sp));
    }
}

/// 指令占位符: 在本次函数返回前, 归还所有申请的栈空间, 恢复所有由被调用者保存的寄存器.
///
/// #### Remusys 约定的栈布局
///
/// 这里按地址从小到大的顺序简要介绍一下 `Remusys` 编译器约定的函数活动栈空间布局.
///
/// 这些是被调用者管理的部分:
///
/// * SP: 指向函数活动的底部. 由于 SysY 没有变长数组语法, 每次函数活动的栈布局都是
///   固定的, 因此不需要 FP 做动态调整.
/// * 局部变量段: 自 SP 位置往上一段, 存放函数活动的局部变量.
/// * 被调用者保存的寄存器段: 紧接着局部变量段, 存放被调用者保存的寄存器.
///
/// 这些是函数调用者管理的部分:
///
/// * 本次调用的溢出参数段: 紧接着被调用者保存的寄存器段, 存放本次调用中传参寄存器
///   (`X0~X7, D0~D7`) 放不下的参数.
/// * 调用者保存的寄存器段: 紧接着溢出参数段, 存放调用者保存的寄存器.
///
/// #### 占位符会变成什么
///
/// `MirRestoreHostRegs` 会恢复成下面几组指令:
///
/// * 收回局部变量段的栈空间: 通常是 `add sp, sp, #<size>`
/// * 恢复被调用者保存的寄存器: 编译器会检查当前函数栈布局定义中要恢复的寄存器,
///   恢复除了返回值以外的寄存器. 然后生成 `add sp, sp, #<size>` 收回对应的栈空间.
#[derive(Clone)]
pub struct MirRestoreHostRegs {
    _common: MirInstCommon,
    regs_norestore: Cell<MirPhysRegSet>,
}

impl Debug for MirRestoreHostRegs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        write!(
            f,
            "MirRestoreHostRegs {} <--> {}",
            node_head.prev, node_head.next
        )
    }
}

impl IMirSubInst for MirRestoreHostRegs {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &[]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &[]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirRestoreHostRegs)
    }
    fn new_empty(opcode: MirOP) -> Self {
        MirRestoreHostRegs {
            _common: MirInstCommon::new(opcode),
            regs_norestore: Cell::new(MirPhysRegSet::new_empty()),
        }
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        let MirInst::MirRestoreHostRegs(inst) = mir_inst else {
            return None;
        };
        Some(inst)
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirRestoreHostRegs(self)
    }
}

impl MirRestoreHostRegs {
    pub fn new(regs_norestore: MirPhysRegSet) -> Self {
        MirRestoreHostRegs {
            _common: MirInstCommon::new(MirOP::MirRestoreHostRegs),
            regs_norestore: Cell::new(regs_norestore),
        }
    }

    /// MIR pseudo-assembly syntax: `mir.restorehostregs`
    pub fn fmt_asm(&self, formatter: &mut FuncFormatContext) -> std::fmt::Result {
        write!(formatter, "mir.restorehostregs")
    }

    pub fn get_regs_norestore(&self) -> MirPhysRegSet {
        self.regs_norestore.get()
    }
    pub fn set_regs_norestore(&self, regs_norestore: MirPhysRegSet) {
        self.regs_norestore.set(regs_norestore);
    }
    pub fn modify_regs_norestore<F>(&self, mut f: F)
    where
        F: FnMut(MirPhysRegSet) -> MirPhysRegSet,
    {
        let regs_norestore = self.regs_norestore.get();
        self.regs_norestore.set(f(regs_norestore));
    }

    pub fn dump_template(&self, out_insts: &mut VecDeque<MirInst>, parent_stack: &MirStackLayout) {
        let var_section_size = parent_stack.vars_size;
        let reg_section_size = parent_stack.saved_regs_section_size();
        let sp = GPR64::sp();

        // Step 1: 收回局部变量段的栈空间
        if var_section_size == 0 {
        } else if let Some(delta_sp) = ImmCalc::try_new(var_section_size as u64) {
            let add_sp = Bin64RC::new(MirOP::Add64I, sp, sp, delta_sp);
            out_insts.push_back(add_sp.into_mir());
        } else {
            // 如果局部变量段太大了, 那么就需要使用临时寄存器来存储偏移量
            let tmpreg = GPR64::new_raw(29);
            let ldr_const = LoadConst64::new(
                MirOP::LoadConst64,
                tmpreg,
                Imm64(var_section_size as u64, ImmKind::Full),
            );
            out_insts.push_back(ldr_const.into_mir());
            let subsp_inst = Bin64R::new(MirOP::Add64R, sp, sp, tmpreg, None);
            out_insts.push_back(subsp_inst.into_mir());
        }

        // Step 2: 恢复被调用者保存的寄存器
        parent_stack.foreach_saved_regs(|saved_reg, sp_offset| {
            let preg = saved_reg.preg;
            if self.get_regs_norestore().has_saved_reg(preg) {
                // 如果这个寄存器不需要恢复, 就跳过
                return;
            }
            let ldr_inst = match DispatchedReg::from_reg(saved_reg.preg) {
                DispatchedReg::F32(dst) => {
                    let offset = ImmLSP32::new(sp_offset as u32);
                    LoadF32Base::new(MirOP::LdrF32Base, dst, sp, offset).into_mir()
                }
                DispatchedReg::F64(dst) => {
                    let offset = ImmLSP64::new(sp_offset as u64);
                    LoadF64Base::new(MirOP::LdrF64Base, dst, sp, offset).into_mir()
                }
                DispatchedReg::G32(dst) => {
                    let offset = ImmLSP32::new(sp_offset as u32);
                    LoadGr32Base::new(MirOP::LdrGr32Base, dst, sp, offset).into_mir()
                }
                DispatchedReg::G64(dst) => {
                    let offset = ImmLSP64::new(sp_offset as u64);
                    LoadGr64Base::new(MirOP::LdrGr64Base, dst, sp, offset).into_mir()
                }
            };
            out_insts.push_back(ldr_inst);
        });

        // Step 3: 收回被调用者保存的寄存器段的栈空间
        if reg_section_size > 0 {
            let delta_sp = ImmCalc::new(reg_section_size as u32);
            let add_sp = Bin64RC::new(MirOP::Add64I, sp, sp, delta_sp);
            out_insts.push_back(MirInst::Bin64RC(add_sp));
        }
    }
}
