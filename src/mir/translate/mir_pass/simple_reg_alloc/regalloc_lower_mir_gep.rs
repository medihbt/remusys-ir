use crate::mir::{
    inst::{
        IMirSubInst,
        gep::{MirGEP, MirGEPBase, MirGEPOffset},
        impls::*,
        inst::MirInst,
        mirops::MirCommentedInst,
        opcode::MirOP,
    },
    module::stack::MirStackLayout,
    operand::{
        compound::MirSymbolOp,
        imm::{Imm64, ImmCalc, ImmLSP32, ImmLSP64},
        imm_traits,
        reg::{GPR64, RegOP},
    },
    translate::mir_pass::simple_reg_alloc::{SRATmpRegAlloc, SpillVRegsError, SpillVRegsResult},
};
use std::collections::VecDeque;

/// 在 RegAlloc 阶段, MIR GEP 就要被 lower 成其他 MIR 指令了.
/// 这主要是因为：
///
/// * GEP 是变长操作数指令, 显然不能为每个虚拟寄存器操作数分配一个
///   临时寄存器, 因此所有虚拟寄存器操作数共享一个临时寄存器.
/// * 不论后续优化如何, 在这个 pass 一定要把指令中所有的虚拟寄存器读操作
///   lower 成一个物理寄存器 + 一个 load 指令。要是有不止一个虚拟寄存器读操作,
///   那这些 load 就会堆在一起, 后面的覆盖前面的结果, 产生错误.
pub(super) fn lower_gep(
    vreg_info: &SpillVRegsResult,
    stack: &MirStackLayout,
    loads_before: &mut VecDeque<MirInst>,
    stores_after: &mut VecDeque<MirInst>,
    gep: &MirGEP,
) -> bool {
    let mut tmpr_alloc = SRATmpRegAlloc::new();

    let dst = gep.get_dst();
    let base = gep.get_base();

    let tmpr1 = loop {
        let tmpr = tmpr_alloc.alloc_gpr64();
        if !tmpr.same_pos_as(dst) && !base.matches_reg(tmpr) {
            break tmpr;
        }
    };

    // 准备好临时寄存器, 用于存储 GEP 的结果.
    let (dst, tmpr) = if dst.is_virtual() {
        let stackpos = vreg_info.findpos_full_unwrap(stack, dst);
        let dst_pos = tmpr1;
        let str = StoreGr64Base::new(MirOP::StrGr64Base, dst_pos, stackpos, ImmLSP64(0));
        stores_after.push_back(str.into_mir());

        let tmpr2 = tmpr_alloc.alloc_gpr64();
        (dst_pos, tmpr2)
    } else {
        (dst, tmpr1)
    };

    lower_gep_base(vreg_info, stack, loads_before, base, dst);

    let weight_reg = loop {
        let tmpr = tmpr_alloc.alloc_gpr64();
        if !tmpr.same_pos_as(dst) && !base.matches_reg(tmpr) {
            break tmpr;
        }
    };
    for (off, weight) in gep.iter_offsets() {
        match off {
            MirGEPOffset::Imm(off) => {
                // GEP 中出现负数就比较可疑了, 这里尝试拦截一下, 以后遇到负数再处理
                assert!(
                    off >= 0,
                    "Inspect: GEP offset should not be negative: {off}"
                );
                lower_gep_immoff(loads_before, dst, tmpr, weight, off)
            }
            MirGEPOffset::G64(reg) => {
                let offset = if reg.is_virtual() {
                    let stackpos = vreg_info.findpos_full_unwrap(stack, reg);
                    let ldr = LoadGr64Base::new(MirOP::LdrGr64Base, tmpr, stackpos, ImmLSP64(0));
                    loads_before.push_back(ldr.into_mir());
                    tmpr
                } else {
                    reg
                };
                lower_gep_gpr64_offset(loads_before, weight_reg, dst, offset, weight);
            }
            MirGEPOffset::S32(off) => {
                let (offset64, inst) = if off.is_virtual() {
                    let stackpos = vreg_info.findpos_full_unwrap(stack, off);
                    let ldrsw = LdrSWBase::new(MirOP::LdrSWBase, tmpr, stackpos, ImmLSP32(0));
                    (tmpr, ldrsw.into_mir())
                } else {
                    let offset64 = off.to_gpr64();
                    let sxtw = ExtR::new(MirOP::SXTW64, offset64, off);
                    (offset64, sxtw.into_mir())
                };
                loads_before.push_back(inst);
                lower_gep_gpr64_offset(loads_before, weight_reg, dst, offset64, weight);
            }
            MirGEPOffset::U32(off) => {
                let offset = if !off.is_virtual() {
                    off
                } else {
                    let stackpos = vreg_info.findpos_full_unwrap(stack, off);
                    let tmpr = tmpr.trunc_to_gpr32();
                    let ldr = LoadGr32Base::new(MirOP::LdrGr32Base, tmpr, stackpos, ImmLSP32(0));
                    loads_before.push_back(ldr.into_mir());
                    tmpr
                };
                let offset = offset.to_gpr64();
                lower_gep_gpr64_offset(loads_before, weight_reg, dst, offset, weight);
            }
        }
    }

    loads_before.push_back({
        // 这里准备放一个注释上去. 注释里的虚拟寄存器都换成栈位置.
        let gep = gep.clone();
        MirCommentedInst::new(gep.into_mir()).into_mir()
    });

    true
}

fn lower_gep_base(
    vreg_info: &SpillVRegsResult,
    stack: &MirStackLayout,
    loads: &mut VecDeque<MirInst>,
    base: MirGEPBase,
    dst: GPR64,
) {
    match base {
        MirGEPBase::Reg(base) if !base.is_virtual() => {
            // 移动 base 到 dst 寄存器中.
            let mov = Una64R::new(MirOP::Mov64R, dst, base, None);
            loads.push_back(mov.into_mir());
        }
        MirGEPBase::Reg(base) => match vreg_info.find_stackpos_full(stack, base) {
            Ok(stackpos) => {
                // 直接把基地址塞到返回的临时寄存器中, 接下来加减啥的方便.
                let ldr = LoadGr64Base::new(MirOP::LdrGr64Base, dst, stackpos, ImmLSP64(0));
                loads.push_back(ldr.into_mir());
            }
            Err(SpillVRegsError::IsStackPos) => {
                // 基地址是一个栈位置, 直接把基地址塞到返回的临时寄存器中.
                let mov = Una64R::new(MirOP::Mov64R, dst, base, None);
                loads.push_back(mov.into_mir());
            }
            Err(e) => panic!("Unexpected error: {e:?}"),
        },
        MirGEPBase::Sym(base) => {
            // 同上, 直接把基地址塞到返回的临时寄存器中
            let src = MirSymbolOp::Global(base);
            let ldr = LoadConst64Symbol::new(MirOP::LoadConst64Symbol, dst, src);
            loads.push_back(ldr.into_mir());
        }
    }
}

fn lower_gep_immoff(loads: &mut VecDeque<MirInst>, dst: GPR64, tmpr: GPR64, weight: u64, off: i64) {
    let off = off * weight as i64;
    if off == 0 {
        // 跳过偏移为 0 的情况
    } else if off > 0 && imm_traits::is_calc_imm(off as u64) {
        // 如果偏移是正数且可以用 ImmCalc 表示, 则直接使用 ImmCalc.
        let offset = ImmCalc::new(off as u32);
        let add = Bin64RC::new(MirOP::Add64I, dst, dst, offset);
        loads.push_back(add.into_mir());
        // todo!("Handle positive offset {off} for {gep:#?}");
    } else if off < 0 && imm_traits::is_calc_imm((-off) as u64) {
        // 如果偏移是负数且可以用 ImmCalc 表示, 则直接使用 ImmCalc.
        let offset = ImmCalc::new((-off) as u32);
        let sub = Bin64RC::new(MirOP::Sub64I, dst, dst, offset);
        loads.push_back(sub.into_mir());
        // todo!("Handle negative offset {off} for {gep:#?}");
    } else {
        // 否则, 就要动用临时寄存器了.
        let ldconst = LoadConst64::new(MirOP::LoadConst64, tmpr, Imm64::full(off as u64));
        loads.push_back(ldconst.into_mir());
        let add = Bin64R::new(MirOP::Add64R, dst, dst, tmpr, None);
        loads.push_back(add.into_mir());
    }
}

fn lower_gep_gpr64_offset(
    loads_before: &mut VecDeque<MirInst>,
    weight_reg: GPR64,
    dst: GPR64,
    offset: GPR64,
    weight: u64,
) {
    if weight == 1 {
        let add = Bin64R::new(MirOP::Add64R, dst, dst, offset, None);
        loads_before.push_back(add.into_mir());
    } else if weight.is_power_of_two() {
        let lsl_count = weight.trailing_zeros() as u8;
        let rm_op = RegOP::LSL(lsl_count);
        let add = Bin64R::new(MirOP::Add64R, dst, dst, offset, Some(rm_op));
        loads_before.push_back(add.into_mir());
    } else {
        let ldrconst = LoadConst64::new(MirOP::LoadConst64, weight_reg, Imm64::full(weight as u64));
        loads_before.push_back(ldrconst.into_mir());
        let madd = TenaryG64::new(MirOP::MAdd64, dst, offset, weight_reg, dst);
        loads_before.push_back(madd.into_mir());
    }
}
