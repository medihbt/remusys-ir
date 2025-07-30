use crate::mir::{
    inst::{IMirSubInst, inst::MirInst, opcode::MirOP},
    module::stack::MirStackLayout,
    operand::MirOperand,
    translate::mir_pass::simple_reg_alloc::{
        SpillVRegsResult, fetch_load_store_pair, regalloc_lower_mir_constop,
        regalloc_lower_mir_gep, regalloc_lower_mir_ldrlit, regalloc_lower_movs,
    },
};
use std::{cell::Cell, collections::VecDeque};

pub(crate) fn regalloc_lower_a_mir_inst(
    vreg_info: &SpillVRegsResult,
    stack: &MirStackLayout,
    loads_before: &mut VecDeque<MirInst>,
    stores_after: &mut VecDeque<MirInst>,
    inst: &MirInst,
) -> bool {
    match inst {
        MirInst::Una64R(mov_inst) if mov_inst.opcode_is(MirOP::Mov64R) => {
            regalloc_lower_movs::lower_mov64r(
                vreg_info,
                stack,
                loads_before,
                stores_after,
                mov_inst,
            )
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
        MirInst::MirStrLitG64(strlit) => {
            regalloc_lower_mir_ldrlit::lower_mstrlit_g64(vreg_info, stack, loads_before, strlit)
        }
        MirInst::MirStrLitG32(strlit) => {
            regalloc_lower_mir_ldrlit::lower_mstrlit_g32(vreg_info, loads_before, strlit)
        }
        MirInst::MirStrLitF64(strlit) => {
            regalloc_lower_mir_ldrlit::lower_mstrlit_f64(vreg_info, loads_before, strlit)
        }
        MirInst::MirStrLitF32(strlit) => {
            regalloc_lower_mir_ldrlit::lower_mstrlit_f32(vreg_info, loads_before, strlit)
        }
        MirInst::MirLdImmF32(ldrf32) => {
            // 加载一个 F32 浮点立即数到寄存器中.
            // 在 SimpleRegAlloc 中, 这个操作不会立即 lower, 而是填充一个临时寄存器.
            regalloc_lower_mir_constop::lower_mldrf32(vreg_info, loads_before, stores_after, ldrf32)
        }
        MirInst::MirLdImmF64(ldrf64) => {
            regalloc_lower_mir_constop::lower_mldrf64(vreg_info, loads_before, stores_after, ldrf64)
        }
        MirInst::MirStImm32(str32) => {
            regalloc_lower_mir_constop::lower_mstimm32(vreg_info, loads_before, str32)
        }
        MirInst::MirStImm64(str64) => {
            regalloc_lower_mir_constop::lower_mstimm64(vreg_info, loads_before, str64)
        }
        MirInst::MirStSym64(str64) => {
            regalloc_lower_mir_constop::lower_mstsym64(vreg_info, loads_before, str64)
        }
        MirInst::MirStImm32Sym(str32sym) => regalloc_lower_mir_constop::lower_mstimm32sym(str32sym),
        MirInst::MirStImm64Sym(str64sym) => regalloc_lower_mir_constop::lower_mstimm64sym(str64sym),
        MirInst::MirStSym64Sym(strsym_sym) => {
            regalloc_lower_mir_constop::lower_mstsym_sym(strsym_sym)
        }
        MirInst::MirGEP(gep) => {
            regalloc_lower_mir_gep::lower_gep(vreg_info, stack, loads_before, stores_after, gep)
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
