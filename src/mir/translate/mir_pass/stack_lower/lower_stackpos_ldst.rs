use crate::mir::{
    inst::{IMirSubInst, impls::*, inst::MirInst, opcode::MirOP},
    module::stack::MirStackLayout,
    operand::{IMirSubOperand, imm::*, reg::*},
    translate::mir_pass::stack_lower::{
        TmpRegAlloc,
        lower_stackpos_operand::{
            LowerPosAction, find_original_sp_offset, lower_stackpos_reg_for_operand,
        },
    },
};
use std::collections::VecDeque;

pub(crate) fn lower_stackpos_for_ldr32base(
    inst: &LoadGr32Base,
    insts: &mut VecDeque<MirInst>,
    stack: &MirStackLayout,
    tmpreg: GPR64,
    extra_delta_sp: u64,
    sp: GPR64,
) -> LowerPosAction {
    let rd = GPR32::from_real(inst.get_rd());
    let base_ptr = GPR64::from_real(inst.get_rn());
    let ImmLSP32(offset_op) = inst.get_rm();

    let sp_offset = extra_delta_sp + find_original_sp_offset(stack, base_ptr);
    let new_offset = offset_op + sp_offset as u32;
    if let Some(new_offset) = ImmLSP32::try_new(new_offset) {
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(new_offset);
        LowerPosAction::Keep
    } else {
        let ldr_const64 = LoadConst64::new(
            MirOP::LoadConst64,
            tmpreg,
            Imm64(new_offset as u64, ImmKind::Full),
        );
        insts.push_back(ldr_const64.into_mir());
        let ldrr = LoadGr32::new(MirOP::LdrGr32, rd, sp, tmpreg, None);
        insts.push_back(ldrr.into_mir());
        LowerPosAction::Replace
    }
}

pub(crate) fn lower_stackpos_for_ldr64base(
    inst: &LoadGr64Base,
    insts: &mut VecDeque<MirInst>,
    stack: &MirStackLayout,
    tmpreg: GPR64,
    extra_delta_sp: u64,
    sp: GPR64,
) -> LowerPosAction {
    let rd = GPR64::from_real(inst.get_rd());
    let base_ptr = GPR64::from_real(inst.get_rn());
    let ImmLSP64(offset_op) = inst.get_rm();

    let sp_offset = extra_delta_sp + find_original_sp_offset(stack, base_ptr);
    let new_offset = offset_op + sp_offset;
    if let Some(new_offset) = ImmLSP64::try_new(new_offset) {
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(new_offset);
        LowerPosAction::Keep
    } else {
        let ldr_const64 =
            LoadConst64::new(MirOP::LoadConst64, tmpreg, Imm64(new_offset, ImmKind::Full));
        insts.push_back(ldr_const64.into_mir());
        let ldrr = LoadGr64::new(MirOP::LdrGr64, rd, sp, tmpreg, None);
        insts.push_back(ldrr.into_mir());
        LowerPosAction::Replace
    }
}

pub(crate) fn lower_stackpos_for_ldrsw(
    inst: &LdrSWBase,
    insts: &mut VecDeque<MirInst>,
    stack: &MirStackLayout,
    tmpreg: GPR64,
    extra_delta_sp: u64,
    sp: GPR64,
) -> LowerPosAction {
    let rd = GPR64::from_real(inst.get_rd());
    let base_ptr = GPR64::from_real(inst.get_rn());
    let ImmLSP32(offset_op) = inst.get_rm();

    let sp_offset = extra_delta_sp + find_original_sp_offset(stack, base_ptr);
    let new_offset = offset_op as u64 + sp_offset;
    if let Some(new_offset) = ImmLSP32::try_new(new_offset.try_into().unwrap()) {
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(new_offset);
        LowerPosAction::Keep
    } else {
        let ldr_const64 =
            LoadConst64::new(MirOP::LoadConst64, tmpreg, Imm64(new_offset, ImmKind::Full));
        insts.push_back(ldr_const64.into_mir());
        let ldrr = LoadGr64::new(MirOP::LdrGr64, rd, sp, tmpreg, None);
        insts.push_back(ldrr.into_mir());
        LowerPosAction::Replace
    }
}

pub(crate) fn lower_stackpos_for_ldrf32base(
    inst: &LoadF32Base,
    insts: &mut VecDeque<MirInst>,
    stack: &MirStackLayout,
    tmpreg: GPR64,
    extra_delta_sp: u64,
    sp: GPR64,
) -> LowerPosAction {
    let rd = FPR32::from_real(inst.get_rd());
    let base_ptr = GPR64::from_real(inst.get_rn());
    let ImmLSP32(offset_op) = inst.get_rm();

    let sp_offset = extra_delta_sp + find_original_sp_offset(stack, base_ptr);
    let new_offset = offset_op + sp_offset as u32;
    if let Some(new_offset) = ImmLSP32::try_new(new_offset) {
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(new_offset);
        LowerPosAction::Keep
    } else {
        let ldr_const64 = LoadConst64::new(
            MirOP::LoadConst64,
            tmpreg,
            Imm64(new_offset as u64, ImmKind::Full),
        );
        insts.push_back(ldr_const64.into_mir());
        let ldrr = LoadF32::new(MirOP::LdrF32, rd, sp, tmpreg, None);
        insts.push_back(ldrr.into_mir());
        LowerPosAction::Replace
    }
}

pub(crate) fn lower_stackpos_for_ldrf64base(
    inst: &LoadF64Base,
    insts: &mut VecDeque<MirInst>,
    stack: &MirStackLayout,
    tmpreg: GPR64,
    extra_delta_sp: u64,
    sp: GPR64,
) -> LowerPosAction {
    let rd = FPR64::from_real(inst.get_rd());
    let base_ptr = GPR64::from_real(inst.get_rn());
    let ImmLSP64(offset_op) = inst.get_rm();

    let sp_offset = extra_delta_sp + find_original_sp_offset(stack, base_ptr);
    let new_offset = offset_op + sp_offset;
    if let Some(new_offset) = ImmLSP64::try_new(new_offset) {
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(new_offset);
        LowerPosAction::Keep
    } else {
        let ldr_const64 =
            LoadConst64::new(MirOP::LoadConst64, tmpreg, Imm64(new_offset, ImmKind::Full));
        insts.push_back(ldr_const64.into_mir());
        let ldrr = LoadF64::new(MirOP::LdrF64, rd, sp, tmpreg, None);
        insts.push_back(ldrr.into_mir());
        LowerPosAction::Replace
    }
}

pub(crate) fn lower_stackpos_for_str32base(
    inst: &StoreGr32Base,
    insts: &mut VecDeque<MirInst>,
    stack: &MirStackLayout,
    tmpreg: GPR64,
    extra_delta_sp: u64,
    sp: GPR64,
) -> LowerPosAction {
    let rs = GPR32::from_real(inst.get_rd());
    let base_ptr = GPR64::from_real(inst.get_rn());
    let ImmLSP32(offset_op) = inst.get_rm();

    let sp_offset = extra_delta_sp + find_original_sp_offset(stack, base_ptr);
    let new_offset = offset_op + sp_offset as u32;
    if let Some(new_offset) = ImmLSP32::try_new(new_offset) {
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(new_offset);
        LowerPosAction::Keep
    } else {
        let ldr_const64 = LoadConst64::new(
            MirOP::LoadConst64,
            tmpreg,
            Imm64(new_offset as u64, ImmKind::Full),
        );
        insts.push_back(ldr_const64.into_mir());
        let strr = StoreGr32::new(MirOP::StrGr32, rs, sp, tmpreg, None);
        insts.push_back(strr.into_mir());
        LowerPosAction::Replace
    }
}

/// 这里要额外处理一种特殊情况: inst 的 `rd` 寄存器也是一个栈位置寄存器.
pub(crate) fn lower_stackpos_for_str64base(
    inst: &StoreGr64Base,
    insts: &mut VecDeque<MirInst>,
    stack: &MirStackLayout,
    extra_delta_sp: u64,
    sp: GPR64,
) -> LowerPosAction {
    let mut tmp_reg_alloc = TmpRegAlloc::new();
    let mut action = LowerPosAction::Keep;

    // 如果 `rs` 是一个虚拟寄存器, 那么需要先把它转化为实际的栈位置寄存器.
    // 这个 if 分支以后 `rd` 操作数可能会被修改, 因此后面得重新获取.
    if inst.get_rd().is_virtual() {
        let rs = GPR64::from_real(inst.get_rd());
        if let Some(used_reg) =
            lower_stackpos_reg_for_operand(rs, &mut tmp_reg_alloc, stack, extra_delta_sp, insts)
        {
            assert!(
                !used_reg.is_virtual(),
                "Lowered stackpos reg {rs:?} to physical reg {used_reg:?}"
            );
            inst.set_rd(used_reg.into_real());
            action = LowerPosAction::InsertFront;
            assert!(
                !inst.get_rd().is_virtual(),
                "StoreGr64Base rd should not be virtual after lowering: {inst:#?}"
            );
        } else {
            // 都到 lower_stackpos 了，虚拟寄存器还不是栈位置, 那就说明遇到错误了.
            panic!("Failed to lower stack position for StoreGr64Base: {inst:#?}");
        }
    }

    // 现在 rd 已经是一个实际的栈位置寄存器了, 下面处理 `base_ptr`.
    let rs = GPR64::from_real(inst.get_rd());
    let base_ptr = GPR64::from_real(inst.get_rn());
    // 如果 `base_ptr` 也是一个虚拟寄存器, 那么需要先把它转化为实际的栈位置寄存器.
    if base_ptr.is_virtual() {
        let ImmLSP64(offset_op) = inst.get_rm();

        let sp_offset = extra_delta_sp + find_original_sp_offset(stack, base_ptr);
        let new_offset = offset_op + sp_offset;
        action = if let Some(new_offset) = ImmLSP64::try_new(new_offset) {
            inst.set_rn(GPR64::sp().into_real());
            inst.set_rm(new_offset);
            LowerPosAction::InsertFront
        } else {
            let tmpreg = tmp_reg_alloc.alloc();
            let ldr_const64 =
                LoadConst64::new(MirOP::LoadConst64, tmpreg, Imm64(new_offset, ImmKind::Full));
            insts.push_back(ldr_const64.into_mir());
            let strr = StoreGr64::new(MirOP::StrGr64, rs, sp, tmpreg, None);
            insts.push_back(strr.into_mir());
            LowerPosAction::Replace
        };
    }

    action
}

pub(crate) fn lower_stackpos_for_strf32base(
    inst: &StoreF32Base,
    insts: &mut VecDeque<MirInst>,
    stack: &MirStackLayout,
    tmpreg: GPR64,
    extra_delta_sp: u64,
    sp: GPR64,
) -> LowerPosAction {
    let rs = FPR32::from_real(inst.get_rd());
    let base_ptr = GPR64::from_real(inst.get_rn());
    let ImmLSP32(offset_op) = inst.get_rm();

    let sp_offset = extra_delta_sp + find_original_sp_offset(stack, base_ptr);
    let new_offset = offset_op + sp_offset as u32;
    if let Some(new_offset) = ImmLSP32::try_new(new_offset) {
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(new_offset);
        LowerPosAction::Keep
    } else {
        let ldr_const64 = LoadConst64::new(
            MirOP::LoadConst64,
            tmpreg,
            Imm64(new_offset as u64, ImmKind::Full),
        );
        insts.push_back(ldr_const64.into_mir());
        let strr = StoreF32::new(MirOP::StrF32, rs, sp, tmpreg, None);
        insts.push_back(strr.into_mir());
        LowerPosAction::Replace
    }
}

pub(crate) fn lower_stackpos_for_strf64base(
    inst: &StoreF64Base,
    insts: &mut VecDeque<MirInst>,
    stack: &MirStackLayout,
    tmpreg: GPR64,
    extra_delta_sp: u64,
    sp: GPR64,
) -> LowerPosAction {
    let rs = FPR64::from_real(inst.get_rd());
    let base_ptr = GPR64::from_real(inst.get_rn());
    let ImmLSP64(offset_op) = inst.get_rm();

    let sp_offset = extra_delta_sp + find_original_sp_offset(stack, base_ptr);
    let new_offset = offset_op + sp_offset;
    if let Some(new_offset) = ImmLSP64::try_new(new_offset) {
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(new_offset);
        LowerPosAction::Keep
    } else {
        let ldr_const64 =
            LoadConst64::new(MirOP::LoadConst64, tmpreg, Imm64(new_offset, ImmKind::Full));
        insts.push_back(ldr_const64.into_mir());
        let strr = StoreF64::new(MirOP::StrF64, rs, sp, tmpreg, None);
        insts.push_back(strr.into_mir());
        LowerPosAction::Replace
    }
}
