use crate::mir::{
    inst::{IMirSubInst, impls::*, inst::MirInst, opcode::MirOP},
    module::stack::MirStackLayout,
    operand::{
        IMirSubOperand,
        imm::{ImmLSP32, ImmLSP64},
        reg::*,
    },
    translate::mir_pass::simple_reg_alloc::{SRATmpRegAlloc, SpillVRegsResult},
};
use std::collections::VecDeque;

pub(super) fn lower_mldrlit_g64(
    vreg_info: &SpillVRegsResult,
    stores_after: &mut VecDeque<MirInst>,
    ldrlit: &MirLdrLitG64,
) -> bool {
    let dst = GPR64::from_real(ldrlit.get_dst());

    let mut tmp_regalloc = SRATmpRegAlloc::new();
    let z64 = ImmLSP64(0);
    if dst.is_physical() {
        // 反正 dst 寄存器的值不会被读取并且被覆盖, 拿这玩意当个临时寄存器用也行.
        ldrlit.set_tmp_addr(dst.into_real());
    } else {
        // 如果是虚拟寄存器, 则需要在栈上分配一个位置.
        let stackpos = vreg_info.find_stackpos(dst).unwrap();
        let tmp = tmp_regalloc.alloc_gpr64();
        ldrlit.set_dst(tmp.into_real());
        ldrlit.set_tmp_addr(tmp.into_real());
        let str = StoreGr64Base::new(MirOP::StrGr64Base, tmp, stackpos, z64);
        stores_after.push_back(str.into_mir());
    }
    false
}

pub(super) fn lower_mldrlit_g32(
    vreg_info: &SpillVRegsResult,
    stores_after: &mut VecDeque<MirInst>,
    ldrlit: &MirLdrLitG32,
) -> bool {
    let dst = GPR32::from_real(ldrlit.get_dst());

    let mut tmp_regalloc = SRATmpRegAlloc::new();
    let z32 = ImmLSP32(0);
    if dst.is_physical() {
        // 反正 dst 寄存器的值不会被读取并且被覆盖, 拿这玩意当个临时寄存器用也行.
        let tmpaddr = GPR64::new(dst.get_id());
        ldrlit.set_tmp_addr(tmpaddr.into_real());
    } else {
        // 如果是虚拟寄存器, 则需要在栈上分配一个位置.
        let stackpos = vreg_info.find_stackpos(dst).unwrap();
        let tmp = tmp_regalloc.alloc_gpr32();
        let tmpaddr = GPR64::new(tmp.get_id());
        ldrlit.set_dst(tmp.into_real());
        ldrlit.set_tmp_addr(tmpaddr.into_real());
        let str = StoreGr32Base::new(MirOP::StrGr32Base, tmp, stackpos, z32);
        stores_after.push_back(str.into_mir());
    }
    false
}

pub(super) fn lower_mldrlit_f64(
    vreg_info: &SpillVRegsResult,
    stores_after: &mut VecDeque<MirInst>,
    ldrlit: &MirLdrLitF64,
) -> bool {
    let dst = FPR64::from_real(ldrlit.get_dst());

    let tmp_addr = {
        let mut tmp_regalloc = SRATmpRegAlloc::new();
        tmp_regalloc.alloc_gpr64()
    };
    let z64 = ImmLSP64(0);
    ldrlit.set_tmp_addr(tmp_addr.into_real());

    if dst.is_virtual() {
        let tmppos = {
            let mut tmp_regalloc = SRATmpRegAlloc::new();
            tmp_regalloc.alloc_fpr64()
        };
        ldrlit.set_dst(tmppos.into_real());
        let stackpos = vreg_info.find_stackpos(dst).unwrap();
        let str = StoreF64Base::new(MirOP::StrF64Base, tmppos, stackpos, z64);
        stores_after.push_back(str.into_mir());
    }
    false
}

pub(super) fn lower_mldrlit_f32(
    vreg_info: &SpillVRegsResult,
    stores_after: &mut VecDeque<MirInst>,
    ldrlit: &MirLdrLitF32,
) -> bool {
    let dst = FPR32::from_real(ldrlit.get_dst());

    let tmp_addr = {
        let mut tmp_regalloc = SRATmpRegAlloc::new();
        tmp_regalloc.alloc_gpr64()
    };
    let z32 = ImmLSP32(0);
    ldrlit.set_tmp_addr(tmp_addr.into_real());

    if dst.is_virtual() {
        let tmppos = {
            let mut tmp_regalloc = SRATmpRegAlloc::new();
            tmp_regalloc.alloc_fpr32()
        };
        ldrlit.set_dst(tmppos.into_real());
        let stackpos = vreg_info.find_stackpos(dst).unwrap();
        let str = StoreF32Base::new(MirOP::StrF32Base, tmppos, stackpos, z32);
        stores_after.push_back(str.into_mir());
    }
    false
}

pub(super) fn lower_mstrlit_g64(
    vreg_info: &SpillVRegsResult,
    stack: &MirStackLayout,
    loads_before: &mut VecDeque<MirInst>,
    strlit: &MirStrLitG64,
) -> bool {
    let mut tmpr_alloc = SRATmpRegAlloc::new();
    let src = vreg_info.lower_gpr64(
        stack,
        GPR64::from_real(strlit.get_rd()),
        loads_before,
        &mut tmpr_alloc,
    );
    strlit.set_rd(src.into_real());

    let tmpreg = tmpr_alloc.alloc_gpr64();
    strlit.set_tmp_addr(tmpreg.into_real());
    false
}

pub(super) fn lower_mstrlit_g32(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    strlit: &MirStrLitG32,
) -> bool {
    let mut tmpr_alloc = SRATmpRegAlloc::new();
    let src = vreg_info.lower_gpr32(
        &MirStackLayout::default(),
        GPR32::from_real(strlit.get_rd()),
        loads_before,
        &mut tmpr_alloc,
    );
    strlit.set_rd(src.into_real());

    let tmpreg = tmpr_alloc.alloc_gpr64();
    strlit.set_tmp_addr(tmpreg.into_real());
    false
}

pub(super) fn lower_mstrlit_f64(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    strlit: &MirStrLitF64,
) -> bool {
    let mut tmpr_alloc = SRATmpRegAlloc::new();
    let src = vreg_info.lower_fpr64(
        &MirStackLayout::default(),
        FPR64::from_real(strlit.get_rd()),
        loads_before,
        &mut tmpr_alloc,
    );
    strlit.set_rd(src.into_real());

    let tmpreg = tmpr_alloc.alloc_gpr64();
    strlit.set_tmp_addr(tmpreg.into_real());
    false
}

pub(super) fn lower_mstrlit_f32(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    strlit: &MirStrLitF32,
) -> bool {
    let mut tmpr_alloc = SRATmpRegAlloc::new();
    let src = vreg_info.lower_fpr32(
        &MirStackLayout::default(),
        FPR32::from_real(strlit.get_rd()),
        loads_before,
        &mut tmpr_alloc,
    );
    strlit.set_rd(src.into_real());

    let tmpreg = tmpr_alloc.alloc_gpr64();
    strlit.set_tmp_addr(tmpreg.into_real());
    false
}
