use crate::mir::{
    inst::{IMirSubInst, impls::*, inst::MirInst, opcode::MirOP},
    operand::{
        IMirSubOperand,
        imm::{ImmLSP32, ImmLSP64},
        reg::*,
    },
    translate::mir_pass::simple_reg_alloc::{SRATmpRegAlloc, SpillVRegsResult},
};
use std::collections::VecDeque;

pub(super) fn lower_mov64r(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    stores_after: &mut VecDeque<MirInst>,
    mov_inst: &Una64R,
) -> bool {
    let dst = mov_inst.get_dst();
    let src = mov_inst.get_src();

    if dst.same_pos_as(src) {
        // 如果源寄存器和目标寄存器是同一个寄存器, 则不需要做任何操作.
        return true;
    }
    let dst_virtual = dst.is_virtual();
    let src_virtual = src.is_virtual();

    let dst = GPR64::from_real(dst);
    let src = GPR64::from_real(src);

    let mut tmp_regalloc = SRATmpRegAlloc::new();
    let z64 = ImmLSP64(0);
    match (dst_virtual, src_virtual) {
        (true, true) => {
            let imr = tmp_regalloc.alloc_gpr64();
            let src = vreg_info.find_stackpos(src).unwrap();
            let dst = vreg_info.find_stackpos(dst).unwrap();
            let ldr = LoadGr64Base::new(MirOP::LdrGr64Base, imr, src, z64);
            let str = StoreGr64Base::new(MirOP::StrGr64Base, imr, dst, z64);
            loads_before.push_back(ldr.into_mir());
            stores_after.push_back(str.into_mir());
            true
        }
        (true, false) => {
            let dst = vreg_info.find_stackpos(dst).unwrap();
            let str = StoreGr64Base::new(MirOP::StrGr64Base, src, dst, z64);
            stores_after.push_back(str.into_mir());
            true
        }
        (false, true) => {
            let src = vreg_info.find_stackpos(src).unwrap();
            let ldr = LoadGr64Base::new(MirOP::LdrGr64Base, dst, src, z64);
            loads_before.push_back(ldr.into_mir());
            true
        }
        (false, false) => false,
    }
}

pub(super) fn lower_mov32r(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    stores_after: &mut VecDeque<MirInst>,
    mov_inst: &Una32R,
) -> bool {
    let dst = mov_inst.get_dst();
    let src = mov_inst.get_src();

    if dst.same_pos_as(src) {
        // 如果源寄存器和目标寄存器是同一个寄存器, 则不需要做任何操作.
        return true;
    }
    let dst_virtual = dst.is_virtual();
    let src_virtual = src.is_virtual();

    let dst = GPR32::from_real(dst);
    let src = GPR32::from_real(src);

    let mut tmp_regalloc = SRATmpRegAlloc::new();
    let z32 = ImmLSP32(0);
    match (dst_virtual, src_virtual) {
        (true, true) => {
            let imr = tmp_regalloc.alloc_gpr32();
            let src = vreg_info.find_stackpos(src).unwrap();
            let dst = vreg_info.find_stackpos(dst).unwrap();
            let ldr = LoadGr32Base::new(MirOP::LdrGr32Base, imr, src, z32);
            let str = StoreGr32Base::new(MirOP::StrGr32Base, imr, dst, z32);
            loads_before.push_back(ldr.into_mir());
            stores_after.push_back(str.into_mir());
            true
        }
        (true, false) => {
            let dst = vreg_info.find_stackpos(dst).unwrap();
            let str = StoreGr32Base::new(MirOP::StrGr32Base, src, dst, z32);
            stores_after.push_back(str.into_mir());
            true
        }
        (false, true) => {
            let src = vreg_info.find_stackpos(src).unwrap();
            let ldr = LoadGr32Base::new(MirOP::LdrGr32Base, dst, src, z32);
            loads_before.push_back(ldr.into_mir());
            true
        }
        (false, false) => false,
    }
}

pub(super) fn lower_movf32(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    stores_after: &mut VecDeque<MirInst>,
    mov_inst: &UnaF32,
) -> bool {
    let dst = mov_inst.get_dst();
    let src = mov_inst.get_src();

    if dst.same_pos_as(src) {
        // 如果源寄存器和目标寄存器是同一个寄存器, 则不需要做任何操作.
        return true;
    }
    let dst_virtual = dst.is_virtual();
    let src_virtual = src.is_virtual();

    let dst = FPR32::from_real(dst);
    let src = FPR32::from_real(src);

    let mut tmp_regalloc = SRATmpRegAlloc::new();
    let z32 = ImmLSP32(0);
    match (dst_virtual, src_virtual) {
        (true, true) => {
            let imr = tmp_regalloc.alloc_fpr32();
            let src = vreg_info.find_stackpos(src).unwrap();
            let dst = vreg_info.find_stackpos(dst).unwrap();
            let ldr = LoadF32Base::new(MirOP::LdrF32Base, imr, src, z32);
            let str = StoreF32Base::new(MirOP::StrF32Base, imr, dst, z32);
            loads_before.push_back(ldr.into_mir());
            stores_after.push_back(str.into_mir());
            true
        }
        (true, false) => {
            let dst = vreg_info.find_stackpos(dst).unwrap();
            let str = StoreF32Base::new(MirOP::StrF32Base, src, dst, z32);
            stores_after.push_back(str.into_mir());
            true
        }
        (false, true) => {
            let src = vreg_info.find_stackpos(src).unwrap();
            let ldr = LoadF32Base::new(MirOP::LdrF32Base, dst, src, z32);
            loads_before.push_back(ldr.into_mir());
            true
        }
        (false, false) => false,
    }
}

pub(super) fn lower_movf64(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    stores_after: &mut VecDeque<MirInst>,
    mov_inst: &UnaF64,
) -> bool {
    let dst = mov_inst.get_dst();
    let src = mov_inst.get_src();

    if dst.same_pos_as(src) {
        // 如果源寄存器和目标寄存器是同一个寄存器, 则不需要做任何操作.
        return true;
    }
    let dst_virtual = dst.is_virtual();
    let src_virtual = src.is_virtual();

    let dst = FPR64::from_real(dst);
    let src = FPR64::from_real(src);

    let mut tmp_regalloc = SRATmpRegAlloc::new();
    let z64 = ImmLSP64(0);
    match (dst_virtual, src_virtual) {
        (true, true) => {
            let imr = tmp_regalloc.alloc_fpr64();
            let src = vreg_info.find_stackpos(src).unwrap();
            let dst = vreg_info.find_stackpos(dst).unwrap();
            let ldr = LoadF64Base::new(MirOP::LdrF64Base, imr, src, z64);
            let str = StoreF64Base::new(MirOP::StrF64Base, imr, dst, z64);
            loads_before.push_back(ldr.into_mir());
            stores_after.push_back(str.into_mir());
            true
        }
        (true, false) => {
            let dst = vreg_info.find_stackpos(dst).unwrap();
            let str = StoreF64Base::new(MirOP::StrF64Base, src, dst, z64);
            stores_after.push_back(str.into_mir());
            true
        }
        (false, true) => {
            let src = vreg_info.find_stackpos(src).unwrap();
            let ldr = LoadF64Base::new(MirOP::LdrF64Base, dst, src, z64);
            loads_before.push_back(ldr.into_mir());
            true
        }
        (false, false) => false,
    }
}
