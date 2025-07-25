use crate::mir::{
    inst::{
        IMirSubInst,
        impls::*,
        inst::MirInst,
        mirops::{MirRestoreHostRegs, MirReturn},
        opcode::MirOP,
    },
    module::stack::VirtRegAlloc,
    operand::{IMirSubOperand, MirOperand, compound::MirSymbolOp, imm::*, imm_traits, reg::*},
    translate::mir_pass::inst_lower::LowerInstAction,
};
use std::collections::VecDeque;

/// Generate MIR instructions for a return operation.
pub fn lower_mir_ret(
    mir_ret: &MirReturn,
    vreg_alloc: &mut VirtRegAlloc,
    out_actions: &mut VecDeque<LowerInstAction>,
) {
    if let Some(retval) = mir_ret.retval() {
        prepare_retval(vreg_alloc, out_actions, retval.get());
    }
    // 恢复所处函数保存的寄存器.
    out_actions.push_back(LowerInstAction::NOP(MirRestoreHostRegs::new().into_mir()));
    // 在 aarch64 中有专门的 ret 指令, 但奇怪的是... 这个 ret 指令的返回地址寄存器也要自己指定.
    // 在汇编里不指定就是 x30, 但 MIR 可没有汇编那种灵活性, 数据流要显式表现出来的.
    let ret_inst = BReg::new(MirOP::Ret, GPR64::ra());
    out_actions.push_back(LowerInstAction::NOP(ret_inst.into_mir()));
}

fn prepare_retval(
    vreg_alloc: &mut VirtRegAlloc,
    out_actions: &mut VecDeque<LowerInstAction>,
    retval: MirOperand,
) {
    match retval {
        MirOperand::GPReg(GPReg(GPReg::RETVAL_POS, ..))
        | MirOperand::VFReg(VFReg(VFReg::RETVAL_POS, ..)) => {}
        MirOperand::GPReg(gp) => {
            let inst = make_move_to_gp_retval_inst(gp);
            out_actions.push_back(LowerInstAction::NOP(inst));
        }
        MirOperand::VFReg(vf) => {
            let inst = make_move_to_fp_retval_inst(vf);
            out_actions.push_back(LowerInstAction::NOP(inst));
        }
        MirOperand::Imm64(imm64) => {
            let value = imm64.get_value();
            let r0_src = GPR64(0, RegUseFlags::empty());
            let r0_dest = GPR64(0, RegUseFlags::DEF);
            let inst = if value == 0 {
                Bin64R::new(MirOP::EOR64R, r0_dest, r0_src, r0_src, None).into_mir()
            } else if imm_traits::is_mov_imm(value as u64) {
                Mov64I::new(MirOP::Mov64I, r0_dest, ImmMov::new(value)).into_mir()
            } else {
                LoadConst64::new(MirOP::LoadConst64, r0_dest, imm64).into_mir()
            };
            out_actions.push_back(LowerInstAction::NOP(inst));
        }
        MirOperand::Imm32(imm32) => {
            let value = imm32.get_value();
            let r0_src = GPR32(0, RegUseFlags::empty());
            let r0_dest = GPR32(0, RegUseFlags::DEF);
            let r0_dest64 = GPR64(0, RegUseFlags::DEF);
            let inst = if value == 0 {
                Bin32R::new(MirOP::EOR32R, r0_dest, r0_src, r0_src, None).into_mir()
            } else if imm_traits::is_mov_imm(value as u64) {
                Mov32I::new(MirOP::Mov32I, r0_dest, ImmMov::new(value as u64)).into_mir()
            } else {
                LoadConst64::new(
                    MirOP::LoadConst64,
                    r0_dest64,
                    Imm64::new(value as u64, ImmKind::Full),
                )
                .into_mir()
            };
            out_actions.push_back(LowerInstAction::NOP(inst));
        }
        MirOperand::Label(label) => {
            let inst = LoadConst64Symbol::new(
                MirOP::LoadConst64Symbol,
                GPR64(0, RegUseFlags::DEF),
                MirSymbolOp::Label(label),
            );
            out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
        }
        MirOperand::Global(global) => {
            let inst = LoadConst64Symbol::new(
                MirOP::LoadConst64Symbol,
                GPR64(0, RegUseFlags::DEF),
                MirSymbolOp::Global(global),
            );
            out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
        }
        MirOperand::SwitchTab(idx) => {
            let inst = LoadConst64Symbol::new(
                MirOP::LoadConst64Symbol,
                GPR64(0, RegUseFlags::DEF),
                MirSymbolOp::SwitchTab(idx),
            );
            out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
        }
        MirOperand::F32(f) => {
            let f0 = FPR32(0, RegUseFlags::DEF);
            let inst = if imm_traits::try_cast_f32_to_aarch8(f).is_some() {
                FMov32I::new(MirOP::FMov32I, f0, ImmFMov32::new(f)).into_mir()
            } else {
                // AArch64 没有规定直接加载浮点常量到浮点寄存器的指令，
                // 需要先加载到整数寄存器，然后再移动到浮点寄存器。
                let fbits = f.to_bits() as u64;
                let GPReg(id, ..) = vreg_alloc.insert_gp(GPR32::new_empty().into_real());
                let midreg64 = GPR64(id, RegUseFlags::DEF);
                let midreg32 = GPR32(id, RegUseFlags::empty());
                let loadconst = LoadConst64::new(
                    MirOP::LoadConst64,
                    midreg64,
                    Imm64::new(fbits, ImmKind::Full),
                );
                out_actions.push_back(LowerInstAction::NOP(loadconst.into_mir()));
                UnaFG32::new(MirOP::FMovFG32, f0, midreg32).into_mir()
            };
            out_actions.push_back(LowerInstAction::NOP(inst));
        }
        MirOperand::F64(f) => {
            let f0 = FPR64(0, RegUseFlags::DEF);
            let inst = if imm_traits::try_cast_f64_to_aarch8(f).is_some() {
                FMov64I::new(MirOP::FMov64I, f0, ImmFMov64::new(f)).into_mir()
            } else {
                // AArch64 没有规定直接加载浮点常量到浮点寄存器的指令，
                // 需要先加载到整数寄存器，然后再移动到浮点寄存器。
                let fbits = f.to_bits();
                let GPReg(id, ..) = vreg_alloc.insert_gp(GPR64::new_empty().into_real());
                let midreg64_dst = GPR64(id, RegUseFlags::DEF);
                let midreg64_src = GPR64(id, RegUseFlags::empty());
                let loadconst = LoadConst64::new(
                    MirOP::LoadConst64,
                    midreg64_dst,
                    Imm64::new(fbits, ImmKind::Full),
                );
                out_actions.push_back(LowerInstAction::NOP(loadconst.into_mir()));
                UnaFG64::new(MirOP::FMovFG64, f0, midreg64_src).into_mir()
            };
            out_actions.push_back(LowerInstAction::NOP(inst));
        }
        _ => panic!("Unsupported return value type: {:?}", retval),
    }
}

fn make_move_to_gp_retval_inst(gp: GPReg) -> MirInst {
    let bits = 1 << gp.get_subreg_index().get_bits_log2();
    match bits {
        32 => Una32R::new(
            MirOP::Mov32R,
            GPR32(0, RegUseFlags::DEF),
            GPR32(gp.get_id_raw(), RegUseFlags::KILL),
            None,
        )
        .into_mir(),
        64 => Una64R::new(
            MirOP::Mov64R,
            GPR64(0, RegUseFlags::DEF),
            GPR64(gp.get_id_raw(), RegUseFlags::KILL),
            None,
        )
        .into_mir(),
        _ => panic!("Binary bits {bits} not supported"),
    }
}
fn make_move_to_fp_retval_inst(vf: VFReg) -> MirInst {
    let bits = 1 << vf.get_subreg_index().get_bits_log2();
    match bits {
        32 => UnaF32::new(
            MirOP::FMov32R,
            FPR32(0, RegUseFlags::DEF),
            FPR32::from_real(vf),
        )
        .into_mir(),
        64 => UnaF64::new(
            MirOP::FMov64R,
            FPR64(0, RegUseFlags::DEF),
            FPR64::from_real(vf),
        )
        .into_mir(),
        _ => panic!("Binary bits {bits} not supported"),
    }
}
