use crate::{
    base::SlabRef,
    ir::{
        IRAllocs, ISubValueSSA, ValueSSA,
        inst::{InstData, InstRef, IrGEPOffset},
    },
    mir::{
        inst::{
            IMirSubInst,
            gep::{MirGEP, MirGEPBase, MirGEPOffset},
            inst::MirInst,
        },
        module::vreg_alloc::VirtRegAlloc,
        operand::{IMirSubOperand, reg::GPR64},
        translate::mirgen::operandgen::{DispatchedReg, InstRetval, OperandMap},
    },
    typing::{TypeContext, ValTypeID},
};
use std::collections::VecDeque;

/// 生成 MIR GEP 指令
pub(super) fn dispatch_gep(
    type_ctx: &TypeContext,
    allocs: &IRAllocs,
    operand_map: &OperandMap,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    ir_ref: InstRef,
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
    let alloc_inst = &allocs.insts;

    // 获取 GEP 的基地址和偏移量迭代器
    let InstData::GEP(gep) = ir_ref.to_data(alloc_inst) else {
        panic!("Expected GEP instruction");
    };
    let base_ptr = gep.get_base();
    assert_eq!(
        base_ptr.get_valtype(allocs),
        ValTypeID::Ptr,
        "Expected base pointer to be a pointer type"
    );
    let offset_iter = gep.offset_iter(type_ctx, allocs);

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
                DispatchedReg::G32(gpr32) => (MirGEPOffset::U32(gpr32), weight),
                _ => panic!("Expected a GPR64 register for GEP index, found: {op:?}"),
            },
            None => panic!("No operand found for GEP index argument: {arg_id:?}"),
        },
        IrGEPOffset::Inst(inst, weight) => match operand_map.find_operand_for_inst(inst) {
            Some(InstRetval::Reg(op)) => match DispatchedReg::from_reg(op) {
                DispatchedReg::G64(gpr64) => (MirGEPOffset::G64(gpr64), weight),
                // 注意：按语法来说这里这里本应是 sext，但我为了省事儿实现成了 zext. 得想办法加点什么了.
                DispatchedReg::G32(gpr32) => (MirGEPOffset::S32(gpr32), weight),
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
    assert!(
        !mir_gep.offsets().is_empty(),
        "Expected at least one GEP offset"
    );
    mir_gep.merge_const_offsets();

    // 将 MIR GEP 添加到输出指令队列
    out_insts.push_back(mir_gep.into_mir());
}

fn translate_base_ptr(operand_map: &OperandMap, base_ptr: ValueSSA) -> MirGEPBase {
    match base_ptr {
        ValueSSA::FuncArg(gref, id) => {
            assert_eq!(
                operand_map.ir_func.0, gref,
                "FuncArg is live only in its function"
            );
            let reg = operand_map.find_operand_for_arg(id).unwrap();
            match DispatchedReg::from_reg(reg) {
                DispatchedReg::G64(gpr64) => MirGEPBase::Reg(gpr64),
                _ => panic!(
                    "Expected a GPR64 register for GEP base pointer, found: {reg:?} on value {base_ptr:?}"
                ),
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
