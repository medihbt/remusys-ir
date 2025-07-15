use crate::{
    base::slabref::SlabRef,
    ir::{
        ValueSSA,
        inst::{InstData, InstDataKind, InstRef, cmp::CmpOp, usedef::UseData},
        module::Module,
    },
    mir::{
        inst::{
            IMirSubInst, MirInstRef, impls::*, inst::MirInst, mirops::MirReturn, opcode::MirOP,
        },
        module::stack::VirtRegAlloc,
        operand::{
            MirOperand,
            reg::{FPR32, FPR64, GPR32, GPR64, RegOperand, RegUseFlags},
        },
        translate::mirgen::{InstTranslateInfo, operandgen::OperandMap},
    },
    typing::id::ValTypeID,
};
use core::panic;
use slab::Slab;
use std::{cell::Ref, collections::VecDeque};

mod binary_gen;
mod call_gen;
mod cast_gen;
mod cmp_gen;
mod gep_gen;
mod jumps_gen;
mod load_store_gen;

pub struct InstDispatchState {
    pub last_pstate_modifier: Option<(InstRef, MirInstRef)>,
}

impl InstDispatchState {
    pub fn pstate_modifier_matches(&self, inst_ref: InstRef) -> bool {
        if let Some((last_inst, _)) = self.last_pstate_modifier {
            last_inst == inst_ref
        } else {
            false
        }
    }

    pub fn new() -> Self {
        Self {
            last_pstate_modifier: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InstDispatchError {
    ShouldNotTranslate(InstRef, InstDataKind),
    Unknown,
}

pub fn dispatch_inst(
    ir_module: &Module,
    state: &mut InstDispatchState,
    inst_info: InstTranslateInfo,
    operand_map: &OperandMap,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
) -> Result<(), InstDispatchError> {
    let ir_ref = inst_info.ir;
    let kind = inst_info.kind;

    let alloc_value = ir_module.borrow_value_alloc();
    let alloc_inst = &alloc_value.alloc_inst;
    let alloc_use = ir_module.borrow_use_alloc();
    let alloc_jt = ir_module.borrow_jt_alloc();
    match kind {
        InstDataKind::ListGuideNode
        | InstDataKind::PhiInstEnd
        | InstDataKind::Phi
        | InstDataKind::Unreachable => {
            return Err(InstDispatchError::ShouldNotTranslate(ir_ref, kind));
        }
        InstDataKind::Ret => {
            dispatch_mir_return(operand_map, out_insts, ir_ref, alloc_inst, alloc_use);
        }
        InstDataKind::Jump => {
            jumps_gen::dispatch_jump(operand_map, out_insts, ir_ref, alloc_inst, alloc_jt)
        }
        InstDataKind::Br => jumps_gen::dispatch_br(
            state,
            operand_map,
            out_insts,
            ir_ref,
            alloc_inst,
            alloc_use,
            alloc_jt,
        ),
        InstDataKind::Switch => todo!("Implement switch instruction handling"),
        InstDataKind::Alloca => {
            // Alloca instructions are not translated to MIR directly,
        }
        InstDataKind::Load => {
            load_store_gen::dispatch_load(operand_map, ir_ref, alloc_inst, alloc_use, out_insts);
        }
        InstDataKind::Store => {
            if let Some(value) = load_store_gen::generate_store_inst(
                ir_module,
                operand_map,
                vreg_alloc,
                out_insts,
                ir_ref,
                alloc_inst,
                alloc_use,
            ) {
                return value;
            }
        }
        InstDataKind::Select => todo!("Implement select instruction handling"),
        InstDataKind::BinOp => binary_gen::dispatch_binaries(
            operand_map,
            ir_module,
            vreg_alloc,
            out_insts,
            ir_ref,
            alloc_inst,
            alloc_use,
        ),
        InstDataKind::Cmp => cmp_gen::dispatch_cmp(
            ir_module,
            operand_map,
            state,
            vreg_alloc,
            out_insts,
            ir_ref,
            alloc_inst,
            alloc_use,
        ),
        InstDataKind::Cast => {
            if let Some(value) = cast_gen::dispatch_casts(
                ir_module,
                operand_map,
                vreg_alloc,
                out_insts,
                ir_ref,
                alloc_inst,
                alloc_use,
                state.last_pstate_modifier.map(|(ref_inst, _)| ref_inst),
            ) {
                return value;
            }
        }
        InstDataKind::IndexPtr => gep_gen::dispatch_gep(
            ir_module,
            operand_map,
            vreg_alloc,
            out_insts,
            ir_ref,
            alloc_inst,
            alloc_use,
        ),
        InstDataKind::Call => call_gen::dispatch_call(
            ir_module,
            operand_map,
            vreg_alloc,
            out_insts,
            ir_ref,
            alloc_inst,
            alloc_use,
        ),
        InstDataKind::Intrin => {
            todo!("Intrinsics not implemented in IR. Do this until IR supports intrinsics")
        }
    };

    Ok(())
}

/// Generates a copy instruction in MIR from `from` to `to` while keeping the binary
/// representation of the value intact.
///
/// e.g. if copying a value from a float vreg containing `1.0f32` to an integer vreg,
/// the value of integer vreg will be `0x3f800000` (bit pattern of `1.0f32`).
pub fn make_copy_inst(to: RegOperand, from: MirOperand, out_insts: &mut VecDeque<MirInst>) {
    let RegOperand(id, si, uf, is_fp) = to;
    let bits_log2 = si.get_bits_log2();
    let inst = if is_fp {
        make_fcopy_inst(id, bits_log2, uf, from)
    } else {
        make_icopy_inst(id, bits_log2, uf, from)
    };
    out_insts.push_back(inst);

    fn make_fcopy_inst(id: u32, bits_log2: u8, uf: RegUseFlags, from: MirOperand) -> MirInst {
        match bits_log2 {
            5 => MirFCopy32::new(MirOP::MirFCopy32, FPR32(id, uf), from).into_mir(),
            6 => MirFCopy64::new(MirOP::MirFCopy64, FPR64(id, uf), from).into_mir(),
            _ => panic!("Unsupported floating-point size: 2 ** {bits_log2}"),
        }
    }
    fn make_icopy_inst(id: u32, bits_log2: u8, uf: RegUseFlags, from: MirOperand) -> MirInst {
        match bits_log2 {
            5 => MirCopy32::new(MirOP::MirCopy32, GPR32(id, uf), from).into_mir(),
            6 => MirCopy64::new(MirOP::MirCopy64, GPR64(id, uf), from).into_mir(),
            _ => panic!("Unsupported integer size: 2 ** {bits_log2}"),
        }
    }
}

fn ir_inst_is_cmp(inst: InstRef, alloc_inst: &Slab<InstData>) -> bool {
    match inst.to_slabref_unwrap(alloc_inst) {
        InstData::Cmp(..) => true,
        _ => false,
    }
}
fn ir_inst_as_cmp<'a>(inst: InstRef, alloc_inst: &'a Slab<InstData>) -> Option<&'a CmpOp> {
    match inst.to_slabref_unwrap(alloc_inst) {
        InstData::Cmp(_, cmp_op) => Some(cmp_op),
        _ => None,
    }
}
fn ir_value_is_cmp(ir_value: ValueSSA, alloc_inst: &Slab<InstData>) -> bool {
    match ir_value {
        ValueSSA::Inst(inst_ref) => ir_inst_is_cmp(inst_ref, alloc_inst),
        _ => false,
    }
}
fn ir_value_as_cmp<'a>(ir_value: ValueSSA, alloc_inst: &'a Slab<InstData>) -> Option<&'a CmpOp> {
    match ir_value {
        ValueSSA::Inst(inst_ref) => ir_inst_as_cmp(inst_ref, alloc_inst),
        _ => None,
    }
}

fn dispatch_mir_return(
    operand_map: &OperandMap<'_>,
    out_insts: &mut VecDeque<MirInst>,
    ir_ref: InstRef,
    alloc_inst: &Slab<InstData>,
    alloc_use: Ref<Slab<UseData>>,
) {
    let (ir_return, has_retval) = match ir_ref.to_slabref_unwrap(alloc_inst) {
        InstData::Ret(c, r) => (r, !matches!(c.ret_type, ValTypeID::Void)),
        _ => panic!("Expected Ret instruction"),
    };
    let mir_ret = if has_retval {
        let retval_ir = ir_return.retval.get_operand(&alloc_use);
        if ir_value_is_cmp(retval_ir, alloc_inst) {
            todo!("Handle return of comparison values");
        } else {
            let retval_mir = operand_map.make_pseudo_operand(retval_ir);
            let has_retval = !matches!(retval_mir, MirOperand::None);
            let retinst = MirReturn::new(has_retval);
            if has_retval {
                retinst.set_retval(retval_mir);
            }
            retinst
        }
    } else {
        MirReturn::new(false)
    };
    out_insts.push_back(mir_ret.into_mir());
}
