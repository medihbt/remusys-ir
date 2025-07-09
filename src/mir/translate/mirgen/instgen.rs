use std::collections::VecDeque;

use crate::{
    ir::inst::{InstDataKind, InstRef},
    mir::{
        inst::{
            call_ret::MirReturn, data_process::UnaryOp, load_store::{LoadAddr, LoadConst}, opcode::MirOP, IMirSubInst, MirInst, MirInstRef
        },
        module::stack::VirtRegAlloc,
        operand::{
            reg::PReg, suboperand::{IMirSubOperand, ImmSymOperand, RegOperand}, MirOperand
        },
        translate::mirgen::{
            imm_utils::{try_cast_f32_to_aarch8, try_cast_f64_to_aarch8}, operandgen::OperandMap, InstTranslateInfo
        },
    },
};

pub struct InstDispatchState {
    last_pstate_modifier: Option<(InstRef, MirInstRef)>,
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
    state: &mut InstDispatchState,
    inst_info: InstTranslateInfo,
    operand_map: &OperandMap,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
) -> Result<(), InstDispatchError> {
    let ir_ref = inst_info.ir;
    let kind = inst_info.kind;

    match kind {
        InstDataKind::ListGuideNode
        | InstDataKind::PhiInstEnd
        | InstDataKind::Phi
        | InstDataKind::Unreachable => Err(InstDispatchError::ShouldNotTranslate(ir_ref, kind)),
        InstDataKind::Ret => {
            let ret_inst = MirReturn::new(true);
            ret_inst.set_retval(todo!("Handle return value"));
            out_insts.push_back(ret_inst.into_mir_inst());
        },
        InstDataKind::Jump => todo!(),
        InstDataKind::Br => todo!(),
        InstDataKind::Switch => todo!(),
        InstDataKind::Alloca => todo!(),
        InstDataKind::Load => todo!(),
        InstDataKind::Store => todo!(),
        InstDataKind::Select => todo!(),
        InstDataKind::BinOp => todo!(),
        InstDataKind::Cmp => todo!(),
        InstDataKind::Cast => todo!(),
        InstDataKind::IndexPtr => todo!(),
        InstDataKind::Call => todo!(),
        InstDataKind::Intrin => todo!("Implement intrinsics handling"),
    }
}

/// Generates a copy instruction in MIR from `from` to `to` while keeping the binary
/// representation of the value intact.
///
/// e.g. if copying a value from a float vreg containing `1.0f32` to an integer vreg,
/// the value of integer vreg will be `0x3f800000` (bit pattern of `1.0f32`).
pub fn make_copy_inst(
    to: RegOperand,
    from: MirOperand,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
) {
    ensure_destreg_valid(to);
    let dest_float = to.is_float();

    fn push_load_addr(
        dest_float: bool,
        to: RegOperand,
        from: ImmSymOperand,
        out_insts: &mut VecDeque<MirInst>,
    ) {
        assert!(!dest_float, "Cannot load address into a float register");
        out_insts.push_back(LoadAddr::new(MirOP::Ldr, to, from).into_mir_inst());
    }

    use ImmSymOperand::*;

    match from {
        MirOperand::PReg(preg) => {
            todo!("move from PReg to RegOperand: {preg:?}");
        }
        MirOperand::VReg(vreg) => {
            todo!("move from PReg to RegOperand: {vreg:?}");
        }
        MirOperand::Imm(imm) => {
            if dest_float {
                let vx = *vreg_alloc.alloc_gp();
                out_insts
                    .push_back(LoadConst::new(MirOP::Ldr, RegOperand::V(vx), imm).into_mir_inst());
                out_insts.push_back(
                    UnaryOp::new(MirOP::FMov, to, MirOperand::VReg(vx), None).into_mir_inst(),
                );
            } else {
                out_insts.push_back(LoadConst::new(MirOP::Ldr, to, imm).into_mir_inst());
            }
        }
        MirOperand::Global(g) => push_load_addr(dest_float, to, Sym(g), out_insts),
        MirOperand::Label(b) => push_load_addr(dest_float, to, Label(b), out_insts),
        MirOperand::VecSwitchTab(idx) => {
            push_load_addr(dest_float, to, VecSwitchTabPos(idx), out_insts)
        }
        MirOperand::BinSwitchTab(idx) => {
            push_load_addr(dest_float, to, BinSwitchTabPos(idx), out_insts)
        }
        MirOperand::None => {}
    };
}

fn push_move_to_float_reg(
    to: RegOperand,
    from: i64,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
) {
    ensure_destreg_valid(to);
    if !to.is_float() {
        panic!("Cannot move float value to non-float register: {to:?}");
    }
    let subreg_index = to.get_subreg_index();
    let to_nbits_log2 = subreg_index.get_bits_log2();
    let imm8 = match to_nbits_log2 {
        5 => try_cast_f32_to_aarch8(f32::from_bits(from as u64 as u32)),
        6 => try_cast_f64_to_aarch8(f64::from_bits(from as u64)),
        _ => {
            panic!("Unsupported float register size for immediate move: {to_nbits_log2} bits");
        }
    };
    match imm8 {
        Some(_) => {
            let fmov = UnaryOp::new(MirOP::FMov, to, from.into_mirop(), None);
            fmov.rd().set(to.into_mirop());
            out_insts.push_back(fmov.into_mir_inst())
        }
        None => {
            let vx = vreg_alloc.alloc_gp();
            *vx.subreg_index_mut() = subreg_index;
            let ldconst = LoadConst::new_empty(MirOP::Ldr);
            ldconst.rt().set(to.into_mirop());
            ldconst.imm().set(from.into_mirop());
            out_insts.push_back(ldconst.into_mir_inst());
            let fmov = UnaryOp::new_empty(MirOP::FMov);
            fmov.rd().set(to.into_mirop());
            fmov.rhs().set(MirOperand::VReg(*vx));
            out_insts.push_back(fmov.into_mir_inst());
        }
    }
}

fn ensure_destreg_valid(to: RegOperand) {
    use PReg::*;
    use RegOperand::*;
    match to {
        P(ZR(..)) | P(PState(_)) | P(PC(..)) => {
            panic!("Register {to:?} is not writable in data processing instructions");
        }
        _ => {}
    }
}
