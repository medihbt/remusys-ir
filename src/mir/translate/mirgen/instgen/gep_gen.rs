use crate::{
    base::slabref::SlabRef,
    ir::{
        ValueSSA,
        inst::{
            InstData, InstRef,
            gep::{IrGEPOffset, IrGEPOffsetIter},
            usedef::UseData,
        },
        module::Module,
    },
    mir::{
        inst::{
            IMirSubInst,
            gep::{MirGEP, MirGEPBase, MirGEPOffset},
            inst::MirInst,
            mirops::MirCommentedInst,
        },
        module::vreg_alloc::VirtRegAlloc,
        operand::{IMirSubOperand, reg::GPR64},
        translate::mirgen::operandgen::{DispatchedReg, InstRetval, OperandMap},
    },
};
use slab::Slab;
use std::collections::VecDeque;

/// 生成 MIR GEP 指令
pub(super) fn dispatch_gep(
    ir_module: &Module,
    operand_map: &OperandMap,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    ir_ref: InstRef,
    alloc_inst: &Slab<InstData>,
    alloc_use: &Slab<UseData>,
) {
    // 准备 MIR GEP 的目标寄存器
    let dst = match operand_map.find_operand_for_inst(ir_ref) {
        Some(InstRetval::Reg(op)) => match DispatchedReg::from_reg(op) {
            DispatchedReg::G64(gpr64) => gpr64,
            _ => panic!("Expected a GPR64 register for GEP destination, found: {op:?}"),
        },
        Some(InstRetval::Wasted) => return, // No destination to generate
        None => panic!("No operand found for GEP instruction: {ir_ref:?}"),
    };

    // 获取 GEP 的基地址和偏移量迭代器
    let (base_ptr, offset_iter) = match ir_ref.to_data(alloc_inst) {
        InstData::IndexPtr(_, gep) => (
            gep.base_ptr.get_operand(&alloc_use),
            IrGEPOffsetIter::from_module(gep, ir_module),
        ),
        _ => panic!("Invalid GEP instruction type"),
    };

    // 解包基地址
    let base_mir = translate_base_ptr(operand_map, base_ptr);
    let tmpreg = vreg_alloc.insert_gpr64(GPR64::new_empty());

    // 处理偏移量
    let offset_weight = offset_iter.map(|off| match off {
        IrGEPOffset::Imm(value) => (MirGEPOffset::Imm(value), 1),
        IrGEPOffset::Arg(_, arg_id, weight) => match operand_map.find_operand_for_arg(arg_id) {
            Some(op) => match DispatchedReg::from_reg(op) {
                DispatchedReg::G64(gpr64) => (MirGEPOffset::G64(gpr64), weight),
                // 注意：按语法来说这里这里本应是 sext，但我为了省事儿实现成了 zext. 得想办法加点什么了.
                DispatchedReg::G32(gpr32) => (MirGEPOffset::G64(gpr32.to_gpr64()), weight),
                _ => panic!("Expected a GPR64 register for GEP index, found: {op:?}"),
            },
            None => panic!("No operand found for GEP index argument: {arg_id:?}"),
        },
        IrGEPOffset::Inst(inst, weight) => match operand_map.find_operand_for_inst(inst) {
            Some(InstRetval::Reg(op)) => match DispatchedReg::from_reg(op) {
                DispatchedReg::G64(gpr64) => (MirGEPOffset::G64(gpr64), weight),
                // 注意：按语法来说这里这里本应是 sext，但我为了省事儿实现成了 zext. 得想办法加点什么了.
                DispatchedReg::G32(gpr32) => (MirGEPOffset::G64(gpr32.to_gpr64()), weight),
                _ => panic!("Expected a GPR64 register for GEP index, found: {op:?}"),
            },
            Some(InstRetval::Wasted) => {
                panic!("GEP index cannot be a wasted operand: {inst:?}")
            }
            None => panic!("No operand found for GEP index instruction: {inst:?}"),
        },
    });
    let mut mir_gep = MirGEP::new(dst, tmpreg, base_mir, offset_weight);

    // 优化手段: 直接合并常量偏移量
    assert!(!mir_gep.offsets().is_empty(), "Expected at least one GEP offset");
    mir_gep.merge_const_offsets();

    // 尝试简化为简单的 MOV 或加法/减法指令，否则将 MIR GEP 添加到输出指令队列
    if !mir_gep.try_simplify(|inst| out_insts.push_back(inst)) {
        out_insts.push_back(mir_gep.into_mir());
    } else {
        let commented = MirCommentedInst::new(mir_gep.into_mir());
        out_insts.push_back(commented.into_mir());
    }
}

fn translate_base_ptr(operand_map: &OperandMap, base_ptr: ValueSSA) -> MirGEPBase {
    match base_ptr {
        ValueSSA::FuncArg(_, id) => {
            let reg = operand_map.find_operand_for_arg(id).unwrap();
            match DispatchedReg::from_reg(reg) {
                DispatchedReg::G64(gpr64) => MirGEPBase::Reg(gpr64),
                _ => panic!("Expected a GPR64 register for GEP base pointer, found: {reg:?}"),
            }
        }
        ValueSSA::Inst(inst) => match operand_map.find_operand_for_inst(inst) {
            Some(InstRetval::Reg(op)) => match DispatchedReg::from_reg(op) {
                DispatchedReg::G64(gpr64) => MirGEPBase::Reg(gpr64),
                _ => panic!("Expected a GPR64 register for GEP base pointer, found: {op:?}"),
            },
            Some(InstRetval::Wasted) => {
                panic!("GEP base pointer cannot be a wasted operand: {inst:?}")
            }
            None => panic!("No operand found for GEP base pointer instruction: {inst:?}"),
        },
        ValueSSA::Global(gref) => match operand_map.find_operand_for_global(gref) {
            Some(mir_gref) => MirGEPBase::Sym(mir_gref),
            None => panic!("No operand found for GEP base pointer global: {gref:?}"),
        },
        _ => panic!("Invalid base pointer type for GEP instruction: {base_ptr:?}"),
    }
}
