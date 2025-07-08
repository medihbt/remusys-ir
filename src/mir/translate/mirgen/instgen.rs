use crate::{
    ir::inst::{InstDataKind, InstRef},
    mir::{
        inst::MirInstRef,
        translate::mirgen::{InstTranslateInfo, operandgen::OperandMap},
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
