use crate::{
    base::SlabRef,
    ir::{
        ValueSSA,
        block::{BlockRef, jump_target::JumpTargetRef},
        constant::expr::ConstExprRef,
        global::GlobalRef,
        inst::{InstRef, UseRef},
        module::{Module, ModuleError},
    },
};

use super::liveset::IRRefLiveSet;

pub(super) fn sweep_module(module: &Module, live_set: &IRRefLiveSet) -> Result<(), ModuleError> {
    sweep_insts(module, live_set)?;
    sweep_blocks(module, live_set)?;
    sweep_globals(module, live_set)?;
    sweep_exprs(module, live_set)?;

    module.borrow_jt_alloc_mut().retain(|jt, _| {
        let jt = JumpTargetRef::from_handle(jt);
        live_set.jt_is_live(jt).unwrap()
    });
    module.borrow_use_alloc_mut().retain(|use_ref, _| {
        let use_ref = UseRef::from_handle(use_ref);
        live_set.use_is_live(use_ref).unwrap()
    });
    Ok(())
}

fn sweep_insts(module: &Module, live_set: &IRRefLiveSet) -> Result<(), ModuleError> {
    let dead_insts = cleanup_insts_before_sweep(module, live_set)?;

    // remove dead instruction RDFG if RDFG is enabled
    if let Some(mut rdfg) = module.borrow_rdfg_alloc_mut() {
        for inst_ref in &dead_insts {
            rdfg.free_node(ValueSSA::Inst(*inst_ref))?;
        }
    }

    // remove dead instructions
    let mut alloc_value = module.borrow_value_alloc_mut();
    let alloc_inst = &mut alloc_value.alloc_inst;
    for inst_ref in &dead_insts {
        alloc_inst.remove(inst_ref.get_handle());
    }
    Ok(())
}

fn cleanup_insts_before_sweep(
    module: &Module,
    live_set: &IRRefLiveSet,
) -> Result<Vec<InstRef>, ModuleError> {
    let mut dead_insts = Vec::new();
    let alloc_value = module.borrow_value_alloc();
    let alloc_use = module.borrow_use_alloc();
    let alloc_jt = module.borrow_jt_alloc();
    let alloc_inst = &alloc_value.alloc_inst;

    // Clean up instruction data before sweeping.
    let rdfg = module.borrow_rdfg_alloc();
    let rcfg = module.borrow_rcfg_alloc();
    for (inst_ref, inst_data) in alloc_inst {
        let inst_ref = InstRef::from_handle(inst_ref);
        // Skip if the instruction is live.
        if live_set.value_is_live(ValueSSA::Inst(inst_ref)).unwrap() {
            continue;
        }
        dead_insts.push(inst_ref);
        inst_data.on_gc_cleanup(&rcfg, &rdfg, &*alloc_use, &*&alloc_jt);
    }

    Ok(dead_insts)
}

fn sweep_blocks(module: &Module, live_set: &IRRefLiveSet) -> Result<(), ModuleError> {
    let dead_blocks = {
        let alloc_value = module.borrow_value_alloc();
        let alloc_block = &alloc_value.alloc_block;
        alloc_block
            .iter()
            .filter_map(|(handle, _)| {
                let block_ref = BlockRef::from_handle(handle);
                // Skip if the block is live.
                if live_set.value_is_live(ValueSSA::Block(block_ref)).unwrap() {
                    None
                } else {
                    Some(block_ref)
                }
            })
            .collect::<Vec<_>>()
    };

    // remove dead blocks' RCFG and RDFG node.
    if let Some(mut rdfg) = module.borrow_rdfg_alloc_mut() {
        for block_ref in &dead_blocks {
            rdfg.free_node(ValueSSA::Block(*block_ref))?;
        }
    }
    if let Some(mut rcfg) = module.borrow_rcfg_alloc_mut() {
        for block_ref in &dead_blocks {
            rcfg.free_node(*block_ref);
        }
    }

    // remove dead blocks
    let mut alloc_value = module.borrow_value_alloc_mut();
    let alloc_block = &mut alloc_value.alloc_block;
    for block_ref in &dead_blocks {
        alloc_block.remove(block_ref.get_handle());
    }

    Ok(())
}

fn sweep_globals(module: &Module, live_set: &IRRefLiveSet) -> Result<(), ModuleError> {
    let dead_globals = {
        let alloc_value = module.borrow_value_alloc();
        let alloc_global = &alloc_value.alloc_global;
        alloc_global
            .iter()
            .filter_map(|(handle, _)| {
                let global_ref = GlobalRef::from_handle(handle);
                // Skip if the global is live.
                if live_set
                    .value_is_live(ValueSSA::Global(global_ref))
                    .unwrap()
                {
                    None
                } else {
                    Some(global_ref)
                }
            })
            .collect::<Vec<_>>()
    };

    // remove RDFG node for globals
    if let Some(mut rdfg) = module.borrow_rdfg_alloc_mut() {
        for global_ref in &dead_globals {
            rdfg.free_node(ValueSSA::Global(*global_ref))?;
        }
    }

    // remove dead globals
    let mut alloc_value = module.borrow_value_alloc_mut();
    let alloc_global = &mut alloc_value.alloc_global;
    for global_ref in &dead_globals {
        alloc_global.remove(global_ref.get_handle());
    }

    Ok(())
}

fn sweep_exprs(module: &Module, live_set: &IRRefLiveSet) -> Result<(), ModuleError> {
    let dead_exprs = {
        let alloc_value = module.borrow_value_alloc();
        let alloc_expr = &alloc_value.alloc_expr;
        alloc_expr
            .iter()
            .filter_map(|(handle, _)| {
                let expr_ref = ConstExprRef::from_handle(handle);
                // Skip if the expr is live.
                if live_set
                    .value_is_live(ValueSSA::ConstExpr(expr_ref))
                    .unwrap()
                {
                    None
                } else {
                    Some(expr_ref)
                }
            })
            .collect::<Vec<_>>()
    };

    // remove RDFG node for exprs
    if let Some(mut rdfg) = module.borrow_rdfg_alloc_mut() {
        for expr_ref in &dead_exprs {
            rdfg.free_node(ValueSSA::ConstExpr(*expr_ref))?;
        }
    }

    // remove dead exprs
    let mut alloc_value = module.borrow_value_alloc_mut();
    let alloc_expr = &mut alloc_value.alloc_expr;
    for expr_ref in &dead_exprs {
        alloc_expr.remove(expr_ref.get_handle());
    }

    Ok(())
}
