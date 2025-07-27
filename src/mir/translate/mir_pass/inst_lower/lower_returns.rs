use crate::mir::{
    inst::{
        IMirSubInst,
        impls::*,
        inst::MirInst,
        mirops::{MirRestoreHostRegs, MirReturn},
        opcode::MirOP,
    },
    module::vreg_alloc::VirtRegAlloc,
    operand::{
        IMirSubOperand, MirOperand, compound::MirSymbolOp, imm::*, imm_traits,
        physreg_set::MirPhysRegSet, reg::*,
    },
    translate::mir_pass::inst_lower::LowerInstAction,
};
use std::collections::VecDeque;

/// 为 MIR Return 伪指令生成对应的汇编指令。
///
/// #### 指令结构
///
/// * (如果有返回值) 把返回值挪动到对应的返回寄存器. (比如 X0/W0 或者 S0/D0)
/// * 重置本次函数活动的栈空间 —— 由于待处理目标函数的栈布局定义在此后的几轮 pass
///   中随时会发生改变, 因此这里只会放一个占位指令 `MirRestoreHostRegs`.
/// * (上述栈空间指令之后, 调用者的栈空间、寄存器布局包括返回地址都恢复了, 所以)
///   执行 `ret ra` 指令, 返回到调用者.
///
/// #### 函数栈空间恢复占位符 `MirRestoreHostRegs`
///
/// 这个占位符最终会被替换为对应的栈空间恢复指令. 这里按地址从小到大的顺序简要介绍一下
///  `Remusys` 编译器约定的函数活动栈空间布局.
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
/// 根据 Remusys 函数调用的约定, `MirRestoreHostRegs` 会恢复成下面几组指令:
///
/// * 收回局部变量段的栈空间: 通常是 `add sp, sp, #<size>`
/// * 恢复被调用者保存的寄存器: 编译器会检查当前函数栈布局定义中要恢复的寄存器,
///   恢复除了返回值以外的寄存器. 然后生成 `add sp, sp, #<size>` 收回对应的栈空间.
pub fn lower_mir_ret(
    mir_ret: &MirReturn,
    vreg_alloc: &mut VirtRegAlloc,
    out_actions: &mut VecDeque<LowerInstAction>,
) {
    let regs_norestore = if let Some(retval) = mir_ret.retval() {
        prepare_retval(vreg_alloc, out_actions, retval.get())
    } else {
        MirPhysRegSet::new_empty()
    };
    // 恢复所处函数保存的寄存器.
    out_actions.push_back(LowerInstAction::NOP(
        MirRestoreHostRegs::new(regs_norestore).into_mir(),
    ));
    // 在 aarch64 中有专门的 ret 指令, 但奇怪的是... 这个 ret 指令的返回地址寄存器也要自己指定.
    // 在汇编里不指定就是 x30, 但 MIR 可没有汇编那种灵活性, 数据流要显式表现出来的.
    let ret_inst = BReg::new(MirOP::Ret, GPR64::ra());
    out_actions.push_back(LowerInstAction::NOP(ret_inst.into_mir()));
}

/// 生成准备返回值的指令, 然后返回有哪些寄存器不要恢复.
fn prepare_retval(
    vreg_alloc: &mut VirtRegAlloc,
    out_actions: &mut VecDeque<LowerInstAction>,
    retval: MirOperand,
) -> MirPhysRegSet {
    match retval {
        MirOperand::GPReg(GPReg(GPReg::RETVAL_POS, ..)) => {
            MirPhysRegSet::from(&[GPReg::new_retval()])
        }
        MirOperand::VFReg(VFReg(VFReg::RETVAL_POS, ..)) => {
            MirPhysRegSet::from(&[VFReg::new_double_retval()])
        }
        MirOperand::GPReg(gp) => {
            let inst = make_move_to_gp_retval_inst(gp);
            out_actions.push_back(LowerInstAction::NOP(inst));
            MirPhysRegSet::from(&[GPReg::new_retval()])
        }
        MirOperand::VFReg(vf) => {
            let inst = make_move_to_fp_retval_inst(vf);
            out_actions.push_back(LowerInstAction::NOP(inst));
            MirPhysRegSet::from(&[VFReg::new_double_retval()])
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
            MirPhysRegSet::from(&[GPReg::new_retval()])
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
            MirPhysRegSet::from(&[GPReg::new_retval()])
        }
        MirOperand::Label(label) => {
            let inst = LoadConst64Symbol::new(
                MirOP::LoadConst64Symbol,
                GPR64(0, RegUseFlags::DEF),
                MirSymbolOp::Label(label),
            );
            out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
            MirPhysRegSet::from(&[GPReg::new_retval()])
        }
        MirOperand::Global(global) => {
            let inst = LoadConst64Symbol::new(
                MirOP::LoadConst64Symbol,
                GPR64(0, RegUseFlags::DEF),
                MirSymbolOp::Global(global),
            );
            out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
            MirPhysRegSet::from(&[GPReg::new_retval()])
        }
        MirOperand::SwitchTab(idx) => {
            let inst = LoadConst64Symbol::new(
                MirOP::LoadConst64Symbol,
                GPR64(0, RegUseFlags::DEF),
                MirSymbolOp::SwitchTab(idx),
            );
            out_actions.push_back(LowerInstAction::NOP(inst.into_mir()));
            MirPhysRegSet::from(&[GPReg::new_retval()])
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
            MirPhysRegSet::from(&[VFReg::new_single_retval()])
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
            MirPhysRegSet::from(&[VFReg::new_double_retval()])
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
