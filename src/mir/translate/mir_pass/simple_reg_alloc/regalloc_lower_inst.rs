use crate::mir::{
    inst::{IMirSubInst, inst::MirInst, opcode::MirOP},
    operand::MirOperand,
    translate::mir_pass::simple_reg_alloc::{
        SpillVRegsResult, fetch_load_store_pair, regalloc_lower_mir_ldrlit, regalloc_lower_movs,
    },
};
use std::{cell::Cell, collections::VecDeque};

pub(crate) fn regalloc_lower_a_mir_inst(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    stores_after: &mut VecDeque<MirInst>,
    inst: &MirInst,
) -> bool {
    match inst {
        MirInst::Una64R(mov_inst) if mov_inst.opcode_is(MirOP::Mov64R) => {
            regalloc_lower_movs::lower_mov64r(vreg_info, loads_before, stores_after, mov_inst)
        }
        MirInst::Una32R(mov_inst) if mov_inst.opcode_is(MirOP::Mov32R) => {
            regalloc_lower_movs::lower_mov32r(vreg_info, loads_before, stores_after, mov_inst)
        }
        MirInst::UnaF32(mov_inst) if mov_inst.opcode_is(MirOP::FMov32R) => {
            regalloc_lower_movs::lower_movf32(vreg_info, loads_before, stores_after, mov_inst)
        }
        MirInst::UnaF64(mov_inst) if mov_inst.opcode_is(MirOP::FMov64R) => {
            regalloc_lower_movs::lower_movf64(vreg_info, loads_before, stores_after, mov_inst)
        }
        MirInst::MirLdrLitG64(ldrlit) => {
            regalloc_lower_mir_ldrlit::lower_mldrlit_g64(vreg_info, stores_after, ldrlit)
        }
        MirInst::MirLdrLitG32(ldrlit) => {
            regalloc_lower_mir_ldrlit::lower_mldrlit_g32(vreg_info, stores_after, ldrlit)
        }
        MirInst::MirLdrLitF64(ldrlit) => {
            regalloc_lower_mir_ldrlit::lower_mldrlit_f64(vreg_info, stores_after, ldrlit)
        }
        MirInst::MirLdrLitF32(ldrlit) => {
            regalloc_lower_mir_ldrlit::lower_mldrlit_f32(vreg_info, stores_after, ldrlit)
        }
        _ => {
            regalloc_lower_ordinary_insts(
                vreg_info,
                loads_before,
                stores_after,
                inst.in_operands(),
                inst.out_operands(),
            );
            false
        }
    }
}

pub fn regalloc_lower_ordinary_insts(
    vreg_info: &SpillVRegsResult,
    loads_before: &mut VecDeque<MirInst>,
    stores_after: &mut VecDeque<MirInst>,
    in_operands: &[Cell<MirOperand>],
    out_operands: &[Cell<MirOperand>],
) {
    // 下面的寄存器作为操作数使用的临时寄存器, 需要在指令前后添加 load/store 指令.
    // * `X8-X15`, `D8-D15`: 按照操作数分布分配
    let mut curr_used_gpr = 8;
    let mut curr_used_fpr = 8;
    for operand in in_operands {
        let (ldr_inst, str_inst) =
            fetch_load_store_pair(vreg_info, &mut curr_used_gpr, &mut curr_used_fpr, operand);
        if let Some(ldr) = ldr_inst {
            loads_before.push_back(ldr);
        }
        if let Some(str) = str_inst {
            stores_after.push_back(str);
        }
    }
    for operand in out_operands {
        let (ldr_inst, str_inst) =
            fetch_load_store_pair(vreg_info, &mut curr_used_gpr, &mut curr_used_fpr, operand);
        if let Some(ldr) = ldr_inst {
            loads_before.push_back(ldr);
        }
        if let Some(str) = str_inst {
            stores_after.push_back(str);
        }
    }
}
