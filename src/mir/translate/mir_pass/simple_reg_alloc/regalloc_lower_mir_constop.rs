use std::collections::VecDeque;

use crate::mir::{
    inst::{IMirSubInst, impls::*, inst::MirInst, opcode::MirOP},
    operand::{
        IMirSubOperand,
        imm::{ImmLSP32, ImmLSP64},
        reg::*,
    },
    translate::mir_pass::simple_reg_alloc::{SRATmpRegAlloc, SpillVRegsResult},
};

/// 给 MirLdImmF32 分配临时寄存器
///
/// #### 指令语法及语义
///
/// * `mir.ldimm.f32 #<fimm> to <f32reg> through <tmpreg64>`
///
/// 把立即数 `#<fimm>` 载入到浮点寄存器 `<f32reg>` 中, 在载入过程中可能需要使用
/// `<tmpreg64>` 做中转.
///
/// 注意: 考虑到后续处理步骤会用到相关寄存器的全部二进制位, 在指令定义中这里 `<tmpreg64>`
/// 仍然是 64 位的.
pub(super) fn lower_mldrf32(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    stores_after: &mut VecDeque<MirInst>,
    ldrf32: &MirLdImmF32,
) -> bool {
    let mut tmp_regalloc = SRATmpRegAlloc::new();
    let tmpreg = tmp_regalloc.alloc_gpr64();
    let dst = FPR32::from_real(ldrf32.get_rd());
    if dst.is_physical() {
        ldrf32.set_tmpreg(tmpreg.into_real());
        false
    } else {
        // 如果是个虚拟寄存器, 那这就跟浮点没什么关系了——所有值都可以直接通过 GPR 传递.
        // 原来这个传入的 ldrf32 指令直接删除, 它会被替换成一个新的 store 指令.
        let stackpos = vreg_info.find_stackpos(dst).unwrap();
        let src = ldrf32.get_src().zext_to_imm64();
        let ldconst = LoadConst64::new(MirOP::LoadConst64, tmpreg, src);
        loads_before.push_back(ldconst.into_mir());
        let str = StoreGr32Base::new(
            MirOP::StrGr32Base,
            tmpreg.trunc_to_gpr32(),
            stackpos,
            ImmLSP32(0),
        );
        stores_after.push_back(str.into_mir());
        true
    }
}

/// 给 MirLdImmF64 分配临时寄存器
///
/// #### 指令语法及语义
///
/// * `mir.ldimm.f64 #<fimm> to <f64reg> through <tmpreg64>`
///
/// 把立即数 `#<fimm>` 载入到浮点寄存器 `<f64reg>` 中, 在载入过程中可能需要使用
/// `<tmpreg64>` 做中转.
pub(super) fn lower_mldrf64(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    stores_after: &mut VecDeque<MirInst>,
    ldrf64: &MirLdImmF64,
) -> bool {
    let mut tmp_regalloc = SRATmpRegAlloc::new();
    let tmpreg = tmp_regalloc.alloc_gpr64();
    let dst = FPR64::from_real(ldrf64.get_rd());
    if dst.is_physical() {
        ldrf64.set_tmpreg(tmpreg.into_real());
        false
    } else {
        // 如果是个虚拟寄存器, 那这就跟浮点没什么关系了——所有值都可以直接通过 GPR 传递.
        // 原来这个传入的 ldrf64 指令直接删除, 它会被替换成一个新的 store 指令.
        let stackpos = vreg_info.find_stackpos(dst).unwrap();
        let src = ldrf64.get_src();
        let ldconst = LoadConst64::new(MirOP::LoadConst64, tmpreg, src);
        loads_before.push_back(ldconst.into_mir());
        let str = StoreGr64Base::new(MirOP::StrGr64Base, tmpreg, stackpos, ImmLSP64(0));
        stores_after.push_back(str.into_mir());
        true
    }
}

/// 给 MirStImm32 分配临时寄存器
///
/// #### 指令语法及语义
///
/// * `mir.stimm.32 #<imm32> to [<base> + #<offset>], tmp-value <tmpreg>`
///
/// 把立即数 `#<imm32>` 存储到栈上, 栈位置由 `<base> + #<offset>` 确定. 在存储
/// 过程中可能需要使用 `<tmpreg>` 做中转.
///
/// 注意: 考虑到后续处理步骤会用到相关寄存器的全部二进制位, 在指令定义中这里 `<tmpreg>`
/// 仍然是 64 位的.
pub(super) fn lower_mstimm32(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    str32: &MirStImm32,
) -> bool {
    let mut tmp_regalloc = SRATmpRegAlloc::new();
    let tmpreg = tmp_regalloc.alloc_gpr64();
    str32.set_tmpreg(tmpreg.into_real());

    let base = GPR64::from_real(str32.get_base());
    match base.get_id() {
        RegID::SP | RegID::Phys(_) => {
            // 直接使用物理寄存器或栈指针, 无需处理.
            return false;
        }
        RegID::ZR => panic!("Cannot store to ZR register!"),
        RegID::Virt(_) | RegID::StackPos(_) => { /* 接下来处理 */ }
        RegID::Invalid => panic!("Cannot store to Invalid register!"),
    }
    let Some(stackpos) = vreg_info.find_stackpos(base) else {
        // 没在分配列表就意味着 base 是个栈位置寄存器, 直接返回.
        return false;
    };
    let base_tmpr = tmp_regalloc.alloc_gpr64();
    let ldr_base = LoadGr64Base::new(MirOP::LdrGr64Base, base_tmpr, stackpos, ImmLSP64(0));
    str32.set_base(base_tmpr.into_real());
    loads_before.push_back(ldr_base.into_mir());
    false
}

/// 给 MirStImm64 分配临时寄存器
///
/// #### 指令语法及语义
///
/// * `mir.stimm.64 #<imm64> to [<base> + #<offset>], tmp-value <tmpreg>`
///
/// 把立即数 `#<imm64>` 存储到栈上, 栈位置由 `<base> + #<offset>` 确定. 在存储
/// 过程中可能需要使用 `<tmpreg>` 做中转.
pub(super) fn lower_mstimm64(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    str64: &MirStImm64,
) -> bool {
    let base = GPR64::from_real(str64.get_base());
    let Some((tmpreg, base_tmpr)) = lower_mir_store_const_64(vreg_info, loads_before, base) else {
        // 没在分配列表就意味着 base 是个栈位置寄存器, 直接返回.
        return false;
    };
    str64.set_tmpreg(tmpreg.into_real());
    str64.set_base(base_tmpr.into_real());
    false
}

/// 给 MirStSym64 分配临时寄存器
pub(super) fn lower_mstsym64(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    strsym64: &MirStSym64,
) -> bool {
    let base = GPR64::from_real(strsym64.get_base());
    let Some((tmpreg, base_tmpr)) = lower_mir_store_const_64(vreg_info, loads_before, base) else {
        // 没在分配列表就意味着 base 是个栈位置寄存器, 直接返回.
        return false;
    };
    strsym64.set_tmpreg(tmpreg.into_real());
    strsym64.set_base(base_tmpr.into_real());
    false
}

fn lower_mir_store_const_64(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    base: GPR64,
) -> Option<(GPR64, GPR64)> {
    let mut tmp_regalloc = SRATmpRegAlloc::new();
    let tmpreg = tmp_regalloc.alloc_gpr64();
    match base.get_id() {
        RegID::SP | RegID::Phys(_) => {
            // 直接使用物理寄存器或栈指针, 无需处理.
            return None;
        }
        RegID::ZR => panic!("Cannot store to ZR register!"),
        RegID::Virt(_) | RegID::StackPos(_) => { /* 接下来处理 */ }
        RegID::Invalid => panic!("Cannot store to Invalid register!"),
    }
    let Some(stackpos) = vreg_info.find_stackpos(base) else {
        // 没在分配列表就意味着 base 是个栈位置寄存器, 直接返回.
        return None;
    };
    let base_tmpr = tmp_regalloc.alloc_gpr64();
    let _0 = ImmLSP64(0);
    let ldr_base = LoadGr64Base::new(MirOP::LdrGr64Base, base_tmpr, stackpos, _0);
    loads_before.push_back(ldr_base.into_mir());
    Some((tmpreg, base_tmpr))
}

pub(super) fn lower_mstimm32sym(str32sym: &MirStImm32Sym) -> bool {
    let mut tmp_regalloc = SRATmpRegAlloc::new();
    str32sym.set_immreg(tmp_regalloc.alloc_gpr64().into_real());
    str32sym.set_addr_reg(tmp_regalloc.alloc_gpr64().into_real());
    false
}

pub(super) fn lower_mstimm64sym(str64sym: &MirStImm64Sym) -> bool {
    let mut tmp_regalloc = SRATmpRegAlloc::new();
    str64sym.set_immreg(tmp_regalloc.alloc_gpr64().into_real());
    str64sym.set_addr_reg(tmp_regalloc.alloc_gpr64().into_real());
    false
}

pub(super) fn lower_mstsym_sym(strsym_sym: &MirStSym64Sym) -> bool {
    let mut tmp_regalloc = SRATmpRegAlloc::new();
    strsym_sym.set_immreg(tmp_regalloc.alloc_gpr64().into_real());
    strsym_sym.set_addr_reg(tmp_regalloc.alloc_gpr64().into_real());
    false
}
