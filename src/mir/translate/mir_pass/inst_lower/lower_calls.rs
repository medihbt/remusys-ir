use crate::mir::{inst::mirops::MirCall, translate::mir_pass::inst_lower::LowerInstAction};
use std::collections::VecDeque;

pub fn lower_mir_call(call_inst: &MirCall, out_actions: &mut VecDeque<LowerInstAction>) {
    call_inst.dump_actions_template(out_actions);
}
