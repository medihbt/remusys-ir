use crate::{
    base::SlabRef,
    ir::{
        ValueSSA,
        constant::data::ConstData,
        inst::{InstData, InstRef, UseData, UseRef},
    },
    mir::{
        inst::{IMirSubInst, inst::MirInst, mirops::MirCall},
        operand::{
            IMirSubOperand, MirOperand,
            imm::{Imm32, Imm64, ImmKind},
            reg::{GPR32, GPR64},
        },
        translate::mirgen::operandgen::{InstRetval, OperandMap},
    },
    typing::id::ValTypeID,
};
use slab::Slab;
use std::collections::VecDeque;

pub(super) fn dispatch_call(
    ir_ref: InstRef,
    operand_map: &OperandMap,
    out_insts: &mut VecDeque<MirInst>,
    alloc_inst: &Slab<InstData>,
    alloc_use: &Slab<UseData>,
) {
    let InstData::Call(c, call) = ir_ref.to_data(alloc_inst) else {
        panic!("Expected call inst");
    };
    let (callee_func, callee_mir) = {
        let ValueSSA::Global(callee_ir) = call.callee.get_operand(alloc_use) else {
            panic!("Expected global function reference");
        };
        operand_map
            .find_function(callee_ir)
            .expect("Failed to find function for call instruction")
    };
    let args = prepare_call_args(&call.args, alloc_use, operand_map);
    let ret_reg = if c.ret_type == ValTypeID::Void {
        None
    } else {
        operand_map
            .find_operand_for_inst(ir_ref)
            .and_then(|retval| match retval {
                InstRetval::Reg(reg) => Some(reg),
                _ => None,
            })
    };

    let call_inst = if let Some(ret_reg) = ret_reg {
        let callee_mir = MirOperand::Global(callee_mir);
        MirCall::with_retreg(callee_mir, ret_reg, &args)
    } else {
        let callee_mir = MirOperand::Global(callee_mir);
        MirCall::with_return_void(callee_mir, &args)
    };
    call_inst.set_callee_func(callee_func);
    out_insts.push_back(call_inst.into_mir());
}

fn prepare_call_args(
    args: &[UseRef],
    alloc_use: &Slab<UseData>,
    operand_map: &OperandMap,
) -> Vec<MirOperand> {
    let mut ret = Vec::with_capacity(args.len());
    for arg in args {
        let arg = arg.get_operand(alloc_use);
        let arg_mir = match arg {
            ValueSSA::ConstData(data) => call_arg_from_constdata(data),
            ValueSSA::FuncArg(_, arg_id) => operand_map
                .find_operand_for_arg(arg_id)
                .unwrap_or_else(|| panic!("Failed to find operand for arg_id: {arg_id}"))
                .into_mir(),
            ValueSSA::Inst(instval) => {
                let ira = operand_map
                    .find_operand_for_inst(instval)
                    .unwrap_or_else(|| panic!("Failed to find operand for inst: {instval:?}"));
                match ira {
                    InstRetval::Reg(r) => r.into_mir(),
                    InstRetval::Wasted => unreachable!("Wasted operand in call argument"),
                }
            }
            ValueSSA::Global(ir_global) => {
                let global = operand_map.find_operand_for_global(ir_global).unwrap();
                MirOperand::Global(global)
            }
            _ => panic!("Unsupported call argument type: {arg:?}"),
        };
        ret.push(arg_mir);
    }
    ret
}

fn call_arg_from_constdata(data: ConstData) -> MirOperand {
    use crate::typing::types::FloatTypeKind::*;
    match data {
        ConstData::Zero(ty) => match ty {
            ValTypeID::Ptr | ValTypeID::Int(64) => GPR64::zr().into_mir(),
            ValTypeID::Int(32) => GPR32::zr().into_mir(),
            ValTypeID::Float(Ieee32) => MirOperand::F32(0.0),
            ValTypeID::Float(Ieee64) => MirOperand::F64(0.0),
            _ => panic!("Unsupported zero constant type: {ty:?}"),
        },
        ConstData::PtrNull(_) | ConstData::Int(64, 0) => GPR64::zr().into_mir(),
        ConstData::Int(32, 0) => GPR32::zr().into_mir(),
        ConstData::Int(64, x) => Imm64(x as u64, ImmKind::Full).into_mir(),
        ConstData::Int(32, x) => Imm32(x as u32, ImmKind::Full).into_mir(),
        ConstData::Float(Ieee32, val) => MirOperand::F32(val as f32),
        ConstData::Float(Ieee64, val) => MirOperand::F64(val),
        _ => panic!("Unsupported constant data in call argument: {data:?}"),
    }
}
