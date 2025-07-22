use crate::{
    base::slabref::SlabRef,
    ir::{
        ValueSSA,
        inst::{InstData, InstRef, usedef::UseData},
        module::Module,
    },
    mir::{
        inst::{IMirSubInst, inst::MirInst, mirops::MirCall},
        module::stack::VirtRegAlloc,
        operand::MirOperand,
        translate::mirgen::operandgen::{OperandMap, DispatchedReg},
    },
    typing::id::ValTypeID,
};
use slab::Slab;
use std::{cell::Ref, collections::VecDeque};

pub(crate) fn dispatch_call(
    ir_module: &Module,
    operand_map: &OperandMap<'_>,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    ir_ref: InstRef,
    alloc_inst: &Slab<InstData>,
    alloc_use: Ref<'_, Slab<UseData>>,
) {
    let (call_inst, has_retval) = match ir_ref.to_slabref_unwrap(alloc_inst) {
        InstData::Call(c, call) => (call, c.ret_type != ValTypeID::Void),
        _ => panic!("Expected Call instruction"),
    };
    let callee = match call_inst.callee.get_operand(&alloc_use) {
        ValueSSA::Global(func) => func,
        _ => panic!(
            "Call instruction do not process dynamic calls (like function pointers or virtual calls)"
        ),
    };
    let (func, callee_mir) = operand_map
        .find_function(callee)
        .expect("Failed to find function for call instruction");
    let args = call_inst
        .args
        .iter()
        .map(|arg| arg.get_operand(&alloc_use))
        .map(|arg| {
            DispatchedReg::from_valuessa(
                operand_map,
                &ir_module.type_ctx,
                vreg_alloc,
                out_insts,
                &arg,
                true,
            )
            .expect("Failed to convert argument to MIR operand")
            .into_mir()
        })
        .collect::<Vec<_>>();
    let call_inst = if has_retval {
        let ret_mir = operand_map
            .find_operand_for_inst(ir_ref)
            .expect("Failed to find return operand for call instruction");
        MirCall::with_retreg(MirOperand::Global(callee_mir), ret_mir, &args)
    } else {
        MirCall::with_return_void(MirOperand::Global(callee_mir), &args)
    };
    call_inst.set_callee_func(func);
    out_insts.push_back(call_inst.into_mir());
}
