use crate::{
    ir::inst::{InstDataKind, InstRef},
    mir::{
        inst::{data_process::UnaryOp, load_store::{ILoadStoreInst, LoadConst}, opcode::MirOP, MirInst, MirInstRef}, module::stack::VirtRegAlloc, operand::{reg::PReg, suboperand::{IMirSubOperand, RegOperand}, MirOperand}, translate::mirgen::{operandgen::OperandMap, InstTranslateInfo}
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
}

pub fn dispatch_inst(
    state: &mut InstDispatchState,
    inst_info: InstTranslateInfo,
    operand_map: &OperandMap,
) -> Result<MirInstRef, InstDispatchError> {
    let ir_ref = inst_info.ir;
    let kind = inst_info.kind;

    match kind {
        InstDataKind::ListGuideNode
        | InstDataKind::PhiInstEnd
        | InstDataKind::Phi
        | InstDataKind::Unreachable => {
            return Err(InstDispatchError::ShouldNotTranslate(ir_ref, kind));
        }

        InstDataKind::Ret => todo!(),
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
    todo!("Implement dispatch_inst for MIR generation");
}

pub fn make_copy_inst(
    to: RegOperand,
    from: MirOperand,
    vreg_alloc: &mut VirtRegAlloc,
) -> Vec<MirInst> {
    match from {
        MirOperand::Imm(i) => if to.is_float() {
            let vreg = *vreg_alloc.alloc_gp();
            vec![
                MirInst::LoadConst(LoadConst::new(MirOP::Ldr, vreg, i)),
                MirInst::Unary(UnaryOp::new(MirOP::FMov, to, vreg.into_mirop())),
            ]
        } else {
            vec![MirInst::LoadConst(LoadConst::new(MirOP::Ldr, to, i))]
        }, 
        MirOperand::PReg(p) => match p {
            PReg::PState(_) => todo!("Add MRS and MSR support"),
            PReg::ZR(..) => MirInst::Unary(
                UnaryOp::new(MirOP::Mov, to, 0i64.into_mirop())
            ),
            PReg::V(n, si, uf) => {
            }
        }
    }
}
