use crate::{
    base::{NullableValue, slablist::SlabListRange, slabref::SlabRef},
    ir::{
        block::{
            BlockData, BlockRef,
            jump_target::{JumpTargetData, JumpTargetRef},
        },
        global::{GlobalData, GlobalRef},
        inst::{InstData, terminator::Jump},
        module::{Module, ModuleError, rcfg::RcfgAlloc},
    },
    mir::translate::ir_pass::critical_edge,
    opt::analysis::cfg::{self, snapshot::CfgSnapshot},
};
use slab::Slab;
use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockCriticalInfo(u8);

impl BlockCriticalInfo {
    const fn new() -> Self {
        BlockCriticalInfo(0)
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

#[derive(Debug, Clone)]
struct EdgeCriticalInfo {
    from: BlockRef,
    to: BlockRef,
    edge_left: u32,
    edge_right: u32,
}

#[derive(Debug, Clone)]
struct CriticalEdges {
    parent_func: GlobalRef,
    cfg_snapshot: CfgSnapshot,
    info: Vec<EdgeCriticalInfo>,
    edges: Vec<JumpTargetRef>,
}

fn collect_critical_edges_in_func(ir_module: &Module, func: GlobalRef) -> CriticalEdges {
    let cfg_snapshot = CfgSnapshot::new_from_func(ir_module, func);

    let mut pred_candidate = Vec::with_capacity(cfg_snapshot.nodes.len());
    let mut succ_candidate = Vec::with_capacity(cfg_snapshot.nodes.len());
    for (index, node) in cfg_snapshot.nodes.iter().enumerate() {
        let mut is_candidate = false;
        if node.prev_set.len() > 1 {
            succ_candidate.push(index);
        }
        if node.next_set.len() > 1 {
            pred_candidate.push(index);
        }
    }
    if pred_candidate.is_empty() || succ_candidate.is_empty() {
        return CriticalEdges {
            parent_func: func,
            cfg_snapshot,
            info: Vec::new(),
            edges: Vec::new(),
        };
    }
    assert!(pred_candidate.is_sorted());
    assert!(succ_candidate.is_sorted());

    let alloc_jt = ir_module.borrow_jt_alloc();
    let mut critical_edges = CriticalEdges {
        parent_func: func,
        cfg_snapshot,
        info: Vec::with_capacity(pred_candidate.len().min(succ_candidate.len())),
        edges: Vec::with_capacity(pred_candidate.len().max(succ_candidate.len())),
    };

    for index in succ_candidate {
        let alloc_jt = ir_module.borrow_jt_alloc();
        let from_bb = cfg_snapshot.nodes[index].block;
        let from_bb_data = ir_module.get_block(from_bb);
        let terminator = from_bb_data
            .get_terminator_subref(ir_module)
            .expect("Block has no terminator");
        let jts_view = if let Some(jts) = terminator.get_jump_targets(ir_module) {
            jts.load_range()
        } else {
            continue;
        };
        for (jt_ref, jt) in jts_view.view(&alloc_jt) {
            let to_bb = jt.get_block();
            if pred_candidate.binary_search(to_bb).is_err() {
                continue; // Not a critical edge
            }
            let edges_left = critical_edges.edges.len() as u32;
            critical_edges.edges.push(jt_ref);
            if let Some(edge_info) = critical_edges.info.last_mut() {
                if edge_info.from == from_bb {
                    edge_info.edge_right += 1;
                }
            }
        }
    }

    ()
}

// pub fn collect_critical_edges(ir_module: &Module) -> Vec<CriticalEdgeInfo> {
// }

// fn collect_funcdef_bodies(
//     ir_module: &Module,
//     alloc_global: &Slab<GlobalData>,
// ) -> Vec<(SlabListRange<BlockRef>, GlobalRef)> {
//     let mut funcdef_bodies = Vec::new();
//     for (_, gref) in &*ir_module.global_defs.borrow() {
//         let func_data = match gref.to_slabref_unwrap(alloc_global) {
//             GlobalData::Func(func_data) => func_data,
//             _ => continue,
//         };
//         if let Some(body) = func_data.get_blocks() {
//             let body = &*body;
//             let body_range = body.load_range();
//             let body_len = body.len();
//             if body_len > 0 {
//                 funcdef_bodies.push((body_range, *gref));
//             }
//         }
//     }
//     funcdef_bodies
// }

// fn break_one_critical_edge(ir_module: &Module, edge_info: &CriticalEdgeInfo) -> Option<BlockRef> {
//     // Step 0: Create a new block jumping to the target block
//     let (common, jump) = Jump::new(ir_module, edge_info.to);
//     let jump_target = jump.get_jt(&*ir_module.borrow_jt_alloc());
//     let jump = ir_module.insert_inst(InstData::Jump(common, jump));

//     let break_bb = BlockData::new_unreachable(ir_module).unwrap();
//     break_bb
//         .set_terminator(ir_module, jump)
//         .expect("Failed to set terminator for break block");
//     let break_bb = ir_module.insert_block(break_bb);

//     // Step 1: Modify `PHI` nodes of the successor block
//     let alloc_value = ir_module.borrow_value_alloc();
//     let alloc_inst = &alloc_value.alloc_inst;
//     let to_bb_data = ir_module.get_block(edge_info.to);

//     let mut n_phi_nodes = 0;
//     for (_, inst) in to_bb_data.instructions.view(alloc_inst) {
//         let phi_node = if let InstData::Phi(_, phi) = inst {
//             phi
//         } else {
//             // PHI nodes are always located at the beginning of the block.
//             // Meeting non-phi node means we have processed all phi nodes.
//             break;
//         };
//         let success = phi_node.replace_from_bb_with_new(edge_info.from, break_bb, ir_module);
//         if success {
//             n_phi_nodes += 1;
//         }
//     }
//     if n_phi_nodes != 0 {
//         // Step 2: Push new block into function body
//         let alloc_block = &alloc_value.alloc_block;
//         let alloc_global = &alloc_value.alloc_global;
//         let parent_func = edge_info.from.get_parent_func(alloc_block);
//         assert!(parent_func.is_nonnull());
//         let parent_func = match parent_func.to_slabref_unwrap(alloc_global) {
//             GlobalData::Func(func_data) => func_data,
//             _ => panic!("Parent function of block must be a function"),
//         };
//         parent_func
//             .get_blocks()
//             .unwrap()
//             .push_back_ref(alloc_block, break_bb)
//             .expect("Failed to push new block into function body");

//         // Step 3: Redirect the edge to the new block
//         edge_info.edge.set_block(ir_module, break_bb);
//         // Final: return the new block reference
//         Some(break_bb)
//     } else {
//         // No PHI nodes were found, so we don't need to break the edge.
//         // Disconnect the Jump instruction target.
//         jump_target.set_block(ir_module, BlockRef::new_null());
//         None
//     }
// }

// /// Remove critical edges from the IR module. Used in PHI-Node elimination.
// ///
// /// ### How to scan
// ///
// /// Suppose there are two basic blocks Ba and Bb with a directed edge `Jt = Ba -> Bb`.
// /// If Ba has multiple successors and Bb has multiple predecessors, then Jt is a "critical edge".
// ///
// /// ### How to remove
// ///
// /// To remove a critical edge, we need to split the edge into two edges:
// ///
// /// 1. Create a new basic block Bc.
// /// 2. Insert Bc between Ba and Bb.
// /// 3. Redirect the edge from Ba to Bc.
// /// 4. Redirect the edge from Bc to Bb.
// ///
// /// ### Notes
// ///
// /// This pass has less memory usage if we perform a "mark and compact"
// /// garbage collection before this pass.
// pub fn break_critical_edges(ir_module: &Module) {
//     let critical_edges = collect_critical_edges(ir_module);
//     if critical_edges.is_empty() {
//         return;
//     }
//     for edge in critical_edges {
//         break_one_critical_edge(ir_module, &edge);
//     }
// }
