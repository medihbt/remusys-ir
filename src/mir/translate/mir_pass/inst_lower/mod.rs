mod lower_calls;
mod lower_copy;
mod lower_ldr_const;
mod lower_returns;

use crate::{
    base::slabref::SlabRef,
    mir::{
        inst::inst::MirInst,
        module::{MirGlobal, MirModule, func::MirFunc},
        translate::mir_pass::inst_lower::lower_calls::lower_mir_call,
        util::stack_adjust::{AdjTreeBuilder, LowerInstAction, MirSpAdjustTree},
    },
};
use std::{collections::VecDeque, rc::Rc};

pub use lower_copy::*;
pub use lower_ldr_const::*;
pub use lower_returns::lower_mir_ret;

fn lower_an_inst(
    inst: &MirInst,
    parent_func: &MirFunc,
    out_actions: &mut VecDeque<LowerInstAction>,
) {
    match inst {
        MirInst::MirCopy64(copy64) => lower_copy64_inst(copy64, out_actions),
        MirInst::MirCopy32(copy32) => lower_copy32_inst(copy32, out_actions),
        MirInst::MirFCopy64(fcopy64) => {
            let mut inner = parent_func.borrow_inner_mut();
            lower_fcopy64_inst(fcopy64, &mut inner.vreg_alloc, out_actions)
        }
        MirInst::MirFCopy32(fcopy32) => {
            let mut inner = parent_func.borrow_inner_mut();
            lower_fcopy32_inst(fcopy32, &mut inner.vreg_alloc, out_actions)
        }
        MirInst::MirCall(call_inst) => {
            lower_mir_call(call_inst, out_actions);
        }
        MirInst::MirReturn(mir_ret) => {
            let mut inner = parent_func.borrow_inner_mut();
            lower_mir_ret(mir_ret, &mut inner.vreg_alloc, out_actions)
        }
        MirInst::MirPCopy(pcopy) => {
            todo!("Handle inst {pcopy:?}: Please implement MRS and MSR in RIG file first!")
        }
        MirInst::MirSwitch(mir_switch) => todo!("Handle inst {mir_switch:?}"),
        _ => {}
    }
}

pub fn lower_a_function(module: &MirModule, func: &MirFunc, adj_tree_builder: &mut AdjTreeBuilder) {
    let mut allocs = module.allocs.borrow_mut();
    let mut insts_to_process = Vec::new();

    for (block_ref, block) in func.blocks.view(&allocs.block) {
        for (inst_ref, inst) in block.insts.view(&allocs.inst) {
            let is_mir_pseudo = matches!(
                inst,
                MirInst::MirCopy64(_)
                    | MirInst::MirCopy32(_)
                    | MirInst::MirFCopy64(_)
                    | MirInst::MirFCopy32(_)
                    | MirInst::MirPCopy(_)
                    | MirInst::MirCall(_)
                    | MirInst::MirReturn(_)
                    | MirInst::MirSwitch(_)
            );
            if is_mir_pseudo {
                insts_to_process.push((inst_ref, inst.clone(), block_ref));
            }
        }
    }

    let mut out_actions = VecDeque::new();
    for (iref, inst, parent_bb) in insts_to_process {
        adj_tree_builder.focus_to_block(parent_bb);
        lower_an_inst(&inst, func, &mut out_actions);
        while let Some(action) = out_actions.pop_front() {
            let new_inst = adj_tree_builder.exec(action, &mut allocs.inst);
            parent_bb
                .get_insts(&allocs.block)
                .node_add_prev(&allocs.inst, iref, new_inst)
                .expect("Failed to add new inst");
        }
        parent_bb
            .get_insts(&allocs.block)
            .unplug_node(&allocs.inst, iref)
            .expect("Failed to unplug old inst");
        allocs.inst.remove(iref.get_handle());
    }
}

pub fn lower_a_module(module: &MirModule) -> MirSpAdjustTree {
    let mut funcs = Vec::new();
    for items in &module.items {
        match &*items.data_from_module(module) {
            MirGlobal::Function(f) => {
                if f.is_extern() {
                    continue;
                }
                funcs.push(Rc::clone(f));
            }
            _ => continue,
        }
    }
    let mut adj_tree_builder = AdjTreeBuilder::new();
    for func in funcs {
        lower_a_function(module, &func, &mut adj_tree_builder);
    }
    adj_tree_builder.build()
}
