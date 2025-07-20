mod lower_calls;
mod lower_copy;
mod lower_returns;
mod lower_stack;

use crate::{
    base::slabref::SlabRef,
    mir::{
        inst::{MirInstRef, inst::MirInst},
        module::{MirGlobal, MirModule, block::MirBlock, func::MirFunc},
    },
};
use slab::Slab;
use std::{collections::VecDeque, rc::Rc};

pub use lower_calls::lower_mir_call;
pub use lower_copy::*;
pub use lower_returns::lower_mir_ret;

fn lower_an_inst(
    inst: &MirInst,
    out_insts: &mut VecDeque<MirInst>,
    alloc_block: &Slab<MirBlock>,
    parent_func: &MirFunc,
) {
    match inst {
        MirInst::MirCopy64(copy64) => lower_copy64_inst(copy64, out_insts),
        MirInst::MirCopy32(copy32) => lower_copy32_inst(copy32, out_insts),
        MirInst::MirFCopy64(fcopy64) => {
            let mut inner = parent_func.borrow_inner_mut();
            lower_fcopy64_inst(fcopy64, &mut inner.vreg_alloc, out_insts)
        }
        MirInst::MirFCopy32(fcopy32) => {
            let mut inner = parent_func.borrow_inner_mut();
            lower_fcopy32_inst(fcopy32, &mut inner.vreg_alloc, out_insts)
        }
        MirInst::MirCall(call_inst) => {
            lower_mir_call(
                call_inst,
                out_insts,
                alloc_block,
                &parent_func,
            )
        }
        MirInst::MirReturn(mir_ret) => {
            let mut inner = parent_func.borrow_inner_mut();
            lower_mir_ret(
                mir_ret,
                &inner.stack_layout.clone(),
                &mut inner.vreg_alloc,
                out_insts,
            )
        }
        MirInst::MirPCopy(pcopy) => {
            todo!("Handle inst {pcopy:?}: Please implement MRS and MSR in RIG file first!")
        }
        MirInst::MirSwitch(mir_switch) => todo!("Handle inst {mir_switch:?}"),
        _ => {}
    }
}

pub fn lower_a_function(module: &MirModule, func: &MirFunc) {
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

    let mut out_insts = VecDeque::new();
    for (iref, inst, parent_bb) in insts_to_process {
        lower_an_inst(&inst, &mut out_insts, &allocs.block, func);
        let mut curr_focus = iref;
        while let Some(new_inst) = out_insts.pop_front() {
            let new_instref = MirInstRef::from_alloc(&mut allocs.inst, new_inst);
            parent_bb
                .get_insts(&allocs.block)
                .node_add_next(&allocs.inst, curr_focus, new_instref)
                .expect("Failed to add new inst");
            curr_focus = new_instref;
        }
        parent_bb
            .get_insts(&allocs.block)
            .unplug_node(&allocs.inst, iref)
            .expect("Failed to unplug old inst");
        allocs.inst.remove(iref.get_handle());
    }
}

pub fn lower_a_module(module: &MirModule) {
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
    for func in funcs {
        lower_a_function(module, &func);
    }
}
