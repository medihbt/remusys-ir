use std::ops::Deref;

use slab::Slab;

use crate::{
    base::{slablist::SlabListRange, slabref::SlabRef},
    ir::{
        block::{
            BlockData, BlockRef,
            jump_target::{JumpTargetData, JumpTargetRef},
        },
        global::GlobalData,
        inst::{InstData, terminator::Jump},
        module::{Module, ModuleError, rcfg::RcfgAlloc},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticalEdgeInfo {
    pub from: BlockRef,
    pub to: BlockRef,
    pub edge: JumpTargetRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockInfo(u8);

impl BlockInfo {
    const fn new() -> Self {
        BlockInfo(0)
    }
    const fn has_multi_preds(self) -> bool {
        self.0 & 0b0000_0001 != 0
    }
    const fn has_multi_succs(self) -> bool {
        self.0 & 0b0000_0010 != 0
    }
    const fn insert_multi_preds(self, value: bool) -> Self {
        let raw = self.0 & 0b1111_1110;
        Self(raw | if value { 0b0000_0001 } else { 0 })
    }
    const fn insert_multi_succs(self, value: bool) -> Self {
        let raw = self.0 & 0b1111_1101;
        Self(raw | if value { 0b0000_0010 } else { 0 })
    }
    fn set_multi_preds(&mut self, value: bool) {
        *self = self.insert_multi_preds(value);
    }
    fn set_multi_succs(&mut self, value: bool) {
        *self = self.insert_multi_succs(value);
    }
}

pub fn collect_critical_edges(ir_module: &Module) -> Vec<CriticalEdgeInfo> {
    let alloc_value = ir_module.borrow_value_alloc();
    let alloc_block = &alloc_value.alloc_block;
    let alloc_inst = &alloc_value.alloc_inst;
    let alloc_global = &alloc_value.alloc_global;

    let alloc_jt = ir_module.borrow_jt_alloc();
    let alloc_jt = alloc_jt.deref();

    let bodies = collect_funcdef_bodies(ir_module, alloc_global);
    let mut block_info_map = vec![BlockInfo::new(); alloc_block.capacity()];

    let rcfg = match ir_module.enable_rcfg() {
        Ok(_) | Err(ModuleError::RCFGEnabled) => ir_module.borrow_rcfg_alloc().unwrap(),
        Err(e) => panic!("Failed to enable RCFG: {e:?}"),
    };
    let rcfg = rcfg.deref();

    let mut num_prob_critical_edges = 0;
    for body in &bodies {
        num_prob_critical_edges +=
            analyze_block_traits(&body, alloc_block, alloc_inst, rcfg, &mut block_info_map);
    }

    let mut critical_edges = Vec::with_capacity(num_prob_critical_edges);
    for body in &bodies {
        analyze_edges_traits(
            &body,
            alloc_block,
            alloc_inst,
            alloc_jt,
            &block_info_map,
            &mut critical_edges,
        );
    }
    critical_edges
}

fn collect_funcdef_bodies(
    ir_module: &Module,
    alloc_global: &Slab<GlobalData>,
) -> Vec<SlabListRange<BlockRef>> {
    let mut funcdef_bodies = Vec::new();
    for (_, gref) in &*ir_module.global_defs.borrow() {
        let func_data = match gref.to_slabref_unwrap(alloc_global) {
            GlobalData::Func(func_data) => func_data,
            _ => continue,
        };
        if let Some(body) = func_data.get_blocks() {
            let body = &*body;
            let body_range = body.load_range();
            let body_len = body.len();
            if body_len > 0 {
                funcdef_bodies.push(body_range);
            }
        }
    }
    funcdef_bodies
}

fn analyze_block_traits(
    body: &SlabListRange<BlockRef>,
    alloc_block: &Slab<BlockData>,
    alloc_inst: &Slab<InstData>,
    rcfg: &RcfgAlloc,
    bb_info: &mut [BlockInfo],
) -> usize {
    let mut n_maybe_critical_edges = 0;
    for (bb_id, bb) in body.view(alloc_block) {
        let bb_rcfg = rcfg.get_node(bb_id).n_preds();
        if bb_rcfg > 1 {
            bb_info[bb_id.get_handle()].set_multi_preds(true);
        }
        let terminator = bb
            .get_terminator_subref_from_alloc(alloc_inst)
            .expect("Block must have a terminator");
        let n_succs = terminator.get_n_jump_targets(alloc_inst);
        if terminator.get_n_jump_targets(alloc_inst) > 1 {
            bb_info[bb_id.get_handle()].set_multi_succs(true);
            n_maybe_critical_edges += n_succs;
        }
    }
    n_maybe_critical_edges
}

fn analyze_edges_traits(
    body: &SlabListRange<BlockRef>,
    alloc_block: &Slab<BlockData>,
    alloc_inst: &Slab<InstData>,
    alloc_jt: &Slab<JumpTargetData>,
    bb_info: &[BlockInfo],
    critical_edges: &mut Vec<CriticalEdgeInfo>,
) {
    for (bb_id, bb) in body.view(alloc_block) {
        if !bb_info[bb_id.get_handle()].has_multi_succs() {
            continue;
        }
        let terminator = bb
            .get_terminator_subref_from_alloc(alloc_inst)
            .expect("Block must have a terminator");
        let succs = match terminator.get_jump_targets_from_alloc_inst(alloc_inst) {
            Some(succs) => succs.load_range(),
            None => continue,
        };
        for (jt_ref, jt) in succs.view(alloc_jt) {
            let to_bb = jt.get_block();
            if bb_info[to_bb.get_handle()].has_multi_preds() {
                critical_edges.push(CriticalEdgeInfo {
                    from: bb_id,
                    to: to_bb,
                    edge: jt_ref,
                });
            }
        }
    }
}

pub(super) fn break_critical_edge(ir_module: &Module, edge_info: &CriticalEdgeInfo) -> BlockRef {
    // Step 0: Create a new block jumping to the target block
    let (common, jump) = Jump::new(ir_module, edge_info.to);
    let jump = ir_module.insert_inst(InstData::Jump(common, jump));

    let break_bb = BlockData::new_unreachable(ir_module).unwrap();
    break_bb
        .set_terminator(ir_module, jump)
        .expect("Failed to set terminator for break block");
    let break_bb = ir_module.insert_block(break_bb);

    // Step 1: Modify `PHI` nodes of the successor block
    let alloc_value = ir_module.borrow_value_alloc();
    let alloc_inst = &alloc_value.alloc_inst;
    let to_bb_data = ir_module.get_block(edge_info.to);
    for (_, inst) in to_bb_data.instructions.view(alloc_inst) {
        let phi_node = if let InstData::Phi(_, phi) = inst {
            phi
        } else {
            // PHI nodes are always located at the beginning of the block.
            // Meeting non-phi node means we have processed all phi nodes.
            break;
        };
        phi_node.replace_from_bb_with_new(edge_info.from, break_bb, ir_module);
    }
    // Step 2: Redirect the edge to the new block
    edge_info.edge.set_block(ir_module, break_bb);

    // Final: return the new block reference
    break_bb
}
