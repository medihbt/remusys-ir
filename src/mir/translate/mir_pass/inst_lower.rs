use std::collections::VecDeque;

use crate::{
    mir::{
        inst::{inst::MirInst, mirops::MirReturn},
        module::{func::MirFunc, stack::VirtRegAlloc},
        translate::mirgen::operandgen::OperandMap,
    },
    typing::id::ValTypeID,
};

/// Generate MIR instructions for a return operation.
pub fn lower_mir_ret(
    operand_map: &OperandMap,
    mir_ret: &MirReturn,
    parent_func: &MirFunc,
    ret_type: &ValTypeID,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
) {
    if mir_ret.has_retval() {
    }
}
