use crate::{
    base::SlabRef,
    ir::{
        ValueSSA,
        block::jump_target::JumpTargetData,
        inst::{CmpOp, InstData, InstRef, UseData},
    },
    mir::{
        inst::{
            IMirSubInst,
            cond::MirCondFlag,
            impls::{CBZs, CondBr, UncondBr},
            inst::MirInst,
            opcode::MirOP,
        },
        module::block::MirBlockRef,
        operand::{
            IMirSubOperand,
            reg::{GPR64, PState, RegUseFlags},
        },
        translate::mirgen::operandgen::{OperandMap, OperandMapError},
    },
};
use slab::Slab;
use std::{cell::Ref, collections::VecDeque};

use super::{InstDispatchState, ir_value_as_cmp};

pub(super) fn dispatch_jump(
    operand_map: &OperandMap<'_>,
    out_insts: &mut VecDeque<MirInst>,
    ir_ref: InstRef,
    alloc_inst: &Slab<InstData>,
    alloc_jt: Ref<'_, Slab<crate::ir::block::jump_target::JumpTargetData>>,
) {
    let jump = match ir_ref.to_data(alloc_inst) {
        InstData::Jump(_, j) => j,
        _ => panic!("Expected Jump instruction"),
    };
    let target_ir = jump.get_block(&alloc_jt);
    let target_mir = operand_map
        .find_operand_for_block(target_ir)
        .expect("Failed to find target block");
    let jump_inst = UncondBr::new(MirOP::B, target_mir);
    out_insts.push_back(jump_inst.into_mir());
}

pub(crate) fn dispatch_br(
    state: &mut InstDispatchState,
    operand_map: &OperandMap<'_>,
    out_insts: &mut VecDeque<MirInst>,
    ir_ref: InstRef,
    alloc_inst: &Slab<InstData>,
    alloc_use: Ref<Slab<UseData>>,
    alloc_jt: Ref<Slab<JumpTargetData>>,
) {
    let br_inst = match ir_ref.to_data(alloc_inst) {
        InstData::Br(_, b) => b,
        _ => panic!("Expected Br instruction"),
    };
    let cond_ir = br_inst.get_cond(&alloc_use);
    let true_ir = br_inst.if_true.get_block(&alloc_jt);
    let false_ir = br_inst.if_false.get_block(&alloc_jt);
    let true_mir = operand_map
        .find_operand_for_block(true_ir)
        .expect("Failed to find true block");
    let false_mir = operand_map
        .find_operand_for_block(false_ir)
        .expect("Failed to find false block");
    if let Some(cmp_op) = ir_value_as_cmp(cond_ir, alloc_inst) {
        dispatch_cmp_br(state, out_insts, cond_ir, true_mir, false_mir, cmp_op);
    } else {
        match operand_map.find_operand_no_constdata(&cond_ir) {
            Ok(cond) => {
                let if_true_br = CBZs::new(MirOP::CBNZ, GPR64::from_mir(cond), true_mir);
                let if_false_br = UncondBr::new(MirOP::B, false_mir);
                out_insts.push_back(if_true_br.into_mir());
                out_insts.push_back(if_false_br.into_mir());
            }
            Err(OperandMapError::IsConstData(c)) => {
                let target = if c.is_zero() { false_mir } else { true_mir };
                let if_true_br = UncondBr::new(MirOP::B, target);
                out_insts.push_back(if_true_br.into_mir());
            }
            _ => {
                panic!("Branch condition must be a comparison or a register operand");
            }
        }
    }
}

pub(crate) fn dispatch_cmp_br(
    state: &mut InstDispatchState,
    out_insts: &mut VecDeque<MirInst>,
    cond_ir: ValueSSA,
    true_mir: MirBlockRef,
    false_mir: MirBlockRef,
    cmp_op: &CmpOp,
) {
    let ValueSSA::Inst(cmp_inst) = cond_ir else {
        unreachable!("Expected a comparison value for branch condition");
    };
    if state.pstate_modifier_matches(cmp_inst) {
        // 从 cmp_inst 到本指令的所有路径上都没有 PState 修改指令,
        // 因此可以直接使用 CondBr 指令
        let if_true_br = CondBr::new(
            MirOP::BCond,
            true_mir,
            PState(RegUseFlags::IMPLICIT_DEF),
            MirCondFlag::from_cmp_cond(cmp_op.cond),
        );
        let if_false_br = UncondBr::new(MirOP::B, false_mir);
        out_insts.push_back(if_true_br.into_mir());
        out_insts.push_back(if_false_br.into_mir());
    } else {
        // cmp_inst 到本指令的路径上有 PState 修改指令, 这造成了一些麻烦.
        // 先放着, SysY 语法中没有这种情况, 而且生成这种情况的优化还没做.
        todo!("Handle branch with PState modifier in path to cmp_inst");
    }
}
