use crate::{ir::{graph_traits::inst, inst::InstRef}, mir::{inst::MirInstRef, translate::mirgen::operandgen::OperandMap}};

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
}

pub fn dispatch_inst(
    state: &mut InstDispatchState,
    inst_ref: InstRef,
    operand_map: &OperandMap,
) -> MirInstRef {
    todo!("Implement dispatch_inst for MIR generation");
}