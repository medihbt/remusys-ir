use crate::mir::{
    fmt::FuncFormatContext,
    inst::{IMirSubInst, MirInstCommon, impls::*, inst::MirInst, opcode::MirOP},
    operand::{
        IMirSubOperand, MirOperand,
        imm::{ImmCalc, ImmLSP64},
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
pub struct MirSaveRegs {
    _common: MirInstCommon,
    saved_regs: Cell<MirPhysRegSet>,
    operands: RegOperandSet,
}

impl Debug for MirSaveRegs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let regs = self.saved_regs.get();

        match regs.num_regs() {
            0 => f
                .debug_struct("MirSaveRegs")
                .field("<-->", &(prev, next))
                .field("regs", &"[]")
                .finish(),
            1..=10 => f
                .debug_struct("MirSaveRegs")
                .field("<-->", &(prev, next))
                .field("regs", &regs.dump_regs())
                .finish(),
            x => f
                .debug_struct("MirSaveRegs")
                .field("<-->", &(prev, next))
                .field("regs", &regs)
                .field("n_regs", &x)
                .finish(),
        }
    }
}

impl IMirSubInst for MirSaveRegs {
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
        self.operands.update(self.saved_regs.get());
        self.operands.operands()
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirSaveRegs)
    }
    fn new_empty(opcode: MirOP) -> Self {
        MirSaveRegs {
            _common: MirInstCommon::new(opcode),
            saved_regs: Cell::new(MirPhysRegSet::new_empty()),
            operands: RegOperandSet::new(RegUseFlags::USE),
        }
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        let MirInst::MirSaveRegs(inst) = mir_inst else {
            return None;
        };
        Some(inst)
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirSaveRegs(self)
    }
}

impl MirSaveRegs {
    pub fn new(saved_regs: MirPhysRegSet) -> Self {
        MirSaveRegs {
            _common: MirInstCommon::new(MirOP::MirSaveRegs),
            saved_regs: Cell::new(saved_regs),
            operands: RegOperandSet::new(RegUseFlags::USE),
        }
    }

    pub fn new_aapcs_callee() -> Self {
        MirSaveRegs {
            _common: MirInstCommon::new(MirOP::MirSaveRegs),
            saved_regs: Cell::new(MirPhysRegSet::new_aapcs_callee()),
            operands: RegOperandSet::new(RegUseFlags::USE),
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

    /// MIR pseudo-assembly syntax: `mir.saveregs <reg1, reg2, ...>`
    pub fn fmt_asm(&self, formatter: &mut FuncFormatContext) -> std::fmt::Result {
        write!(formatter, "mir.saveregs ")?;
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

    /// 把该指令转换为保存寄存器的动作模板.
    ///
    /// 主要步骤包括:
    ///
    /// * 如果自己是空的, 就什么都不做.
    /// * 计算自己被保存的寄存器数量, 按照每个寄存器 8 字节的大小计算出寄存器保存消耗的栈空间
    /// * 按 16 字节对齐的原则计算出实际的栈空间, 最后给出 sub sp, sp, <size> 的指令.
    /// * 对每个寄存器, 生成对应的 str 指令, 把寄存器的值保存到栈上.
    pub fn dump_actions_template(&self, out_insts: &mut VecDeque<MirInst>) {
        let saved_regs = self.get_saved_regs();
        let num_regs = saved_regs.num_regs();
        if num_regs == 0 {
            return;
        }
        let size = (num_regs * 8).next_multiple_of(16);
        let sp = GPR64::sp();
        let sub_sp = Bin64RC::new(MirOP::Sub64I, sp, sp, ImmCalc::new(size));
        out_insts.push_back(MirInst::Bin64RC(sub_sp));

        for (index, reg) in saved_regs.iter().enumerate() {
            let RegOperand(id, _, _, is_fp) = reg;
            let reg_id = RegID::from_raw(id);
            let offset = ImmLSP64::new((index as u64) * 8);

            let str_inst = if is_fp {
                let src = FPR64::new(reg_id);
                StoreF64Base::new(MirOP::StrF64Base, src, sp, offset).into_mir()
            } else {
                let src = GPR64::new(reg_id);
                StoreGr64Base::new(MirOP::StrGr64Base, src, sp, offset).into_mir()
            };
            out_insts.push_back(str_inst);
        }
    }
}
