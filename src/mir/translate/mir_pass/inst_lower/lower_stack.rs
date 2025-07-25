use std::collections::VecDeque;

use crate::mir::{
    inst::{IMirSubInst, impls::*, inst::MirInst, opcode::MirOP},
    module::stack::VirtRegAlloc,
    operand::{
        IMirSubOperand,
        imm::{Imm64, ImmCalc, ImmKind, ImmLSP64},
        imm_traits,
        physreg_set::MirPhysRegSet,
        reg::*,
    },
};

#[derive(Debug, Clone)]
pub struct SavedRegStackPos {
    pub gpr_idx: [u8; 32],
    pub fpr_idx: [u8; 32],
}

impl SavedRegStackPos {
    const INVALID_ID: u8 = u8::MAX;

    pub fn _get_gpr_stack_offset(&self, id: u8) -> Option<usize> {
        if id >= 32 || self.gpr_idx[id as usize] == Self::INVALID_ID {
            None
        } else {
            let idx = self.gpr_idx[id as usize];
            Some(idx as usize * 8)
        }
    }
    pub fn _get_fpr_stack_offset(&self, id: u8) -> Option<usize> {
        if id >= 32 || self.fpr_idx[id as usize] == Self::INVALID_ID {
            None
        } else {
            let idx = self.fpr_idx[id as usize];
            Some(idx as usize * 8)
        }
    }
    pub fn _gpr_is_saved(&self, id: u8) -> bool {
        self._get_gpr_stack_offset(id).is_some()
    }
    pub fn _fpr_is_saved(&self, id: u8) -> bool {
        self._get_fpr_stack_offset(id).is_some()
    }

    pub fn build_save_regs(saved_regs: MirPhysRegSet, out_insts: &mut VecDeque<MirInst>) -> Self {
        let mut ret = Self {
            gpr_idx: [Self::INVALID_ID; 32],
            fpr_idx: [Self::INVALID_ID; 32],
        };
        if saved_regs.is_empty() {
            return ret;
        }

        // 目前该寄存器保存函数会直接保存寄存器的所有位到栈上—— MirCallerSavedRegs 还没有位域标识.
        // 尽管对于一些只有低 32 位的寄存器，这可能不是最优的方式，但它好写啊.
        let nregs = saved_regs.num_regs();
        let saved_stack_size = nregs as u32 * 8;
        // 这个保存栈大小小于 64 * 8 = 512, 在 sub 指令的立即数限制范围内.
        debug_assert!(imm_traits::is_calc_imm(saved_stack_size as u64));
        let sp = GPR64::sp();
        let offset = ImmCalc::new(saved_stack_size);

        // 预留一部分栈空间出来.
        out_insts.push_back(Bin64RC::new(MirOP::Sub64I, sp, sp, offset).into_mir());

        for (stack_pos, reg) in saved_regs.into_iter().enumerate() {
            let offset_imm = ImmLSP64::new(stack_pos as u64 * 8);
            let RegOperand(id, _, _, is_fp) = reg;
            if is_fp {
                let rd = FPR64(id, RegUseFlags::USE);
                let store = StoreF64Base::new(MirOP::StrF64Base, rd, sp, offset_imm);
                out_insts.push_back(store.into_mir());
                ret.fpr_idx[id as usize] = stack_pos as u8;
            } else {
                let rd = GPR64(id, RegUseFlags::USE);
                let store = StoreGr64Base::new(MirOP::StrGr64Base, rd, sp, offset_imm);
                out_insts.push_back(store.into_mir());
                ret.gpr_idx[id as usize] = stack_pos as u8;
            }
        }
        ret
    }
}

pub fn make_reserve_and_restore_stack_space_insts(
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    spilled_args_size: u64,
    restore_sp_insts: &mut Vec<MirInst>,
) {
    if spilled_args_size == 0 {
        return;
    }
    // Reserve stack space for spilled arguments.
    let sp = GPR64::sp();
    if imm_traits::is_calc_imm(spilled_args_size) {
        let offset = ImmCalc::new(spilled_args_size as u32);
        let reserve_sp = Bin64RC::new(MirOP::Sub64I, sp, sp, offset);
        out_insts.push_back(reserve_sp.into_mir());

        let restore_sp = Bin64RC::new(MirOP::Add64I, sp, sp, offset);
        restore_sp_insts.push(restore_sp.into_mir());
    } else {
        let temp_reg = vreg_alloc.insert_gp(GPR64::new_empty().into_real());
        let temp_reg = GPR64::from_real(temp_reg);
        let ldrconst = LoadConst64::new(
            MirOP::LoadConst64,
            temp_reg,
            Imm64(spilled_args_size, ImmKind::Full),
        );
        out_insts.push_back(ldrconst.into_mir());
        let reserve_sp = Bin64R::new(MirOP::Sub64R, sp, sp, temp_reg, None);
        out_insts.push_back(reserve_sp.into_mir());
        let restore_sp = Bin64R::new(MirOP::Add64R, sp, sp, temp_reg, None);
        restore_sp_insts.push(restore_sp.into_mir());
    }
}
