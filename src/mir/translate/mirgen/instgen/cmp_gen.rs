use crate::{
    base::{NullableValue, slabref::SlabRef},
    ir::{
        inst::{InstData, InstRef, usedef::UseData},
        module::Module,
        opcode::Opcode,
    },
    mir::{
        inst::{IMirSubInst, MirInstRef, impls::*, inst::MirInst, opcode::MirOP},
        module::vreg_alloc::VirtRegAlloc,
        operand::reg::*,
        translate::mirgen::{
            instgen::InstDispatchState,
            operandgen::{DispatchedReg, OperandMap},
        },
    },
};
use slab::Slab;
use std::{cell::Ref, collections::VecDeque};

pub(super) fn dispatch_cmp(
    ir_module: &Module,
    operand_map: &OperandMap<'_>,
    state: &mut InstDispatchState,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    ir_ref: InstRef,
    alloc_inst: &Slab<InstData>,
    alloc_use: Ref<Slab<UseData>>,
) {
    let (opcode, inst) = match ir_ref.to_data(alloc_inst) {
        InstData::Cmp(c, b) => (c.opcode, b),
        _ => panic!("Expected Cmp instruction"),
    };
    let lhs_ir = inst.lhs.get_operand(&alloc_use);
    let rhs_ir = inst.rhs.get_operand(&alloc_use);
    let type_ctx = &ir_module.type_ctx;
    let lhs_mir =
        DispatchedReg::from_valuessa(operand_map, type_ctx, vreg_alloc, out_insts, &lhs_ir, true)
            .expect("Failed to convert LHS operand to MIR");
    let rhs_mir =
        DispatchedReg::from_valuessa(operand_map, type_ctx, vreg_alloc, out_insts, &rhs_ir, true)
            .expect("Failed to convert LHS operand to MIR");
    use DispatchedReg::*;
    let inst = match (lhs_mir, rhs_mir) {
        (F32(lhs), F32(rhs)) => {
            assert_eq!(opcode, Opcode::Fcmp);
            FCmp32::new(MirOP::FCmp32, PState::in_cmp(), lhs, rhs).into_mir()
        }
        (F64(lhs), F64(rhs)) => {
            assert_eq!(opcode, Opcode::Fcmp);
            FCmp64::new(MirOP::FCmp64, PState::in_cmp(), lhs, rhs).into_mir()
        }
        (G32(lhs), G32(rhs)) => {
            assert_eq!(opcode, Opcode::Icmp);
            ICmp32R::new(MirOP::ICmp32R, PState::in_cmp(), lhs, rhs, None).into_mir()
        }
        (G64(lhs), G64(rhs)) => {
            assert_eq!(opcode, Opcode::Icmp);
            ICmp64R::new(MirOP::ICmp64R, PState::in_cmp(), lhs, rhs, None).into_mir()
        }
        _ => panic!(
            "Invalid operands for comparison: {:?} and {:?}",
            lhs_mir, rhs_mir
        ),
    };
    out_insts.push_back(inst);
    state.last_pstate_modifier = Some((ir_ref, MirInstRef::new_null()));
}
