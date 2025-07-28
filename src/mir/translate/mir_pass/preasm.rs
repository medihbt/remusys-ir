use crate::{
    base::slabref::SlabRef,
    mir::{
        inst::{MirInstRef, inst::MirInst},
        module::{
            MirAllocs, MirGlobal, MirModule,
            block::{MirBlock, MirBlockRef},
        },
        translate::mir_pass::inst_lower::{lower_ldr_const64, lower_ldr_symbol},
    },
};
use slab::Slab;
use std::{collections::VecDeque, rc::Rc};

pub fn preasm_pass_for_module(module: &mut MirModule) {
    let mut all_funcs = Vec::new();
    for &globals in &module.items {
        let f = match &*globals.data_from_module(module) {
            MirGlobal::Function(f) if f.is_define() => Rc::clone(f),
            _ => continue,
        };
        all_funcs.push(f);
    }
    for func in all_funcs {
        let insts_to_process = func.dump_insts_with_module_when(module, |inst| {
            matches!(
                inst,
                MirInst::LoadConst64(_) | MirInst::LoadConst64Symbol(_)
            )
        });
        let allocs = module.allocs.get_mut();
        let MirAllocs { block: alloc_block, inst: alloc_inst, .. } = &mut *allocs;
        preasm_pass(&insts_to_process, alloc_block, alloc_inst);
    }
}

fn preasm_pass_for_inst(inst: &MirInst, out_insts: &mut VecDeque<MirInst>) {
    match inst {
        MirInst::LoadConst64(ldr_const64) => lower_ldr_const64(ldr_const64, out_insts),
        MirInst::LoadConst64Symbol(ldr_symbol) => lower_ldr_symbol(ldr_symbol, out_insts),
        _ => {}
    }
}

/// 预汇编 pass.
///
/// 诸如 LLVM 这样的汇编器不支持某些伪指令, 因此在这里消除它们.
pub fn preasm_pass(
    insts: &[(MirBlockRef, MirInstRef)],
    alloc_bb: &Slab<MirBlock>,
    alloc_inst: &mut Slab<MirInst>,
) {
    let mut out_insts = VecDeque::new();
    for &(block_ref, inst_ref) in insts {
        preasm_pass_for_inst(inst_ref.to_slabref_unwrap(alloc_inst), &mut out_insts);
        while let Some(inst) = out_insts.pop_front() {
            let new_inst_ref = MirInstRef::from_alloc(alloc_inst, inst);
            block_ref
                .get_insts(alloc_bb)
                .node_add_prev(alloc_inst, inst_ref, new_inst_ref)
                .expect("Failed to add new inst");
        }
        block_ref
            .get_insts(alloc_bb)
            .unplug_node(alloc_inst, inst_ref)
            .expect("Failed to unplug old inst");
    }
}
