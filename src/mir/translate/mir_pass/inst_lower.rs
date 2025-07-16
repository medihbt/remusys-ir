use std::collections::VecDeque;

use crate::{
    mir::{
        inst::{impls::{Una32R, Una64R}, inst::MirInst, mirops::MirReturn, opcode::MirOP, IMirSubInst}, module::{func::MirFunc, stack::VirtRegAlloc}, operand::{reg::{GPReg, RegUseFlags, VFReg, GPR32, GPR64}, MirOperand}, translate::mirgen::operandgen::OperandMap
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
    fn make_move_to_r0_inst(gp: GPReg) -> MirInst {
        let bits = 1 << gp.get_subreg_index().get_bits_log2();
        match bits {
            32 => Una32R::new(MirOP::Mov32R, GPR32(0, RegUseFlags::DEF), GPR32(gp.get_id_raw(), RegUseFlags::KILL), None).into_mir(),
            64 => Una64R::new(MirOP::Mov64R, GPR64(0, RegUseFlags::DEF), GPR64(gp.get_id_raw(), RegUseFlags::KILL), None).into_mir(),
            _ => panic!("Binary bits {bits} not supported")
        }
    }
    if let Some(retval) = mir_ret.retval() {
        let retval = retval.get();
        match retval {
            MirOperand::GPReg(GPReg(GPReg::RETVAL_POS, ..)) |
            MirOperand::VFReg(VFReg(VFReg::RETVAL_POS, ..)) => {},
            MirOperand::GPReg(gp) => {
                let inst = make_move_to_r0_inst(gp);
                out_insts.push_back(inst);
            }
        }
    }
}
