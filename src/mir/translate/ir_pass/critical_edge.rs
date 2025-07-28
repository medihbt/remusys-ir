use crate::{
    base::{NullableValue, slabref::SlabRef},
    ir::{
        block::{BlockData, BlockRef, jump_target::JumpTargetRef},
        global::{GlobalData, GlobalRef},
        inst::{InstData, terminator::Jump},
        module::Module,
    },
    opt::analysis::cfg::snapshot::CfgSnapshot,
};

/// Representing a critical edge or a group of repeated critical edges
/// between two basic blocks in the IR module.
#[derive(Debug)]
struct EdgeCriticalInfo {
    from: BlockRef,
    to: BlockRef,
    edge_left: u32,
    edge_right: u32,
}

#[derive(Debug)]
struct CriticalEdgeGroup<'a> {
    from: BlockRef,
    to: BlockRef,
    edges: &'a [JumpTargetRef],
}

#[derive(Debug)]
struct CriticalEdges {
    cfg_snapshot: CfgSnapshot,
    info: Vec<EdgeCriticalInfo>,
    edges: Vec<JumpTargetRef>,
}

impl CriticalEdges {
    fn new_empty(func: GlobalRef, cfg: CfgSnapshot) -> Self {
        Self::with_capacity(func, cfg, 0, 0)
    }
    fn with_capacity(
        _func: GlobalRef,
        cfg: CfgSnapshot,
        info_capacity: usize,
        edges_capacity: usize,
    ) -> Self {
        CriticalEdges {
            cfg_snapshot: cfg,
            info: Vec::with_capacity(info_capacity),
            edges: Vec::with_capacity(edges_capacity),
        }
    }

    fn new_from_func(func: GlobalRef, ir_module: &Module) -> Self {
        let cfg_snapshot = CfgSnapshot::new_from_func(ir_module, func);

        let mut multi_succs_candidate = Vec::with_capacity(cfg_snapshot.nodes.len());
        let mut multi_preds_candidate = Vec::with_capacity(cfg_snapshot.nodes.len());
        for (index, node) in cfg_snapshot.nodes.iter().enumerate() {
            if node.prev_set.len() > 1 && node.block.has_phi(ir_module) {
                multi_preds_candidate.push(index);
            }
            if node.next_set.len() > 1 {
                multi_succs_candidate.push(index);
            }
        }
        if multi_succs_candidate.is_empty() || multi_preds_candidate.is_empty() {
            return CriticalEdges::new_empty(func, cfg_snapshot);
        }
        assert!(multi_succs_candidate.is_sorted());
        assert!(multi_preds_candidate.is_sorted());

        let mut critical_edges = CriticalEdges::with_capacity(
            func,
            cfg_snapshot,
            multi_succs_candidate.len().min(multi_preds_candidate.len()),
            multi_succs_candidate.len().max(multi_preds_candidate.len()),
        );
        for &index in &multi_succs_candidate {
            critical_edges._build_block_add_edges(ir_module, &multi_preds_candidate, index);
        }
        critical_edges
    }

    fn _is_candidate(cfg: &CfgSnapshot, candidates: &[usize], bb: BlockRef) -> bool {
        let bb_index = cfg
            .block_get_node_pos(bb)
            .expect("Block not found in CFG snapshot");
        candidates.binary_search(&bb_index).is_ok()
    }
    fn _build_block_add_edges(
        &mut self,
        ir_module: &Module,
        succ_candidates: &[usize],
        pred_index: usize,
    ) {
        let from_bb = self.cfg_snapshot.nodes[pred_index].block;
        let (jts, len) = match from_bb.get_jump_targets(ir_module) {
            Some(jts) => (jts.load_range(), jts.len()),
            None => return, // No jump targets, no critical edges
        };

        let alloc_jt = ir_module.borrow_jt_alloc();
        let mut raw_edges = Vec::with_capacity(len);
        for (jt_ref, jt) in jts.view(&alloc_jt) {
            let to_bb = jt.get_block();
            if !Self::_is_candidate(&self.cfg_snapshot, succ_candidates, to_bb) {
                continue; // Not a critical edge
            }
            raw_edges.push((from_bb, to_bb, jt_ref));
        }
        if raw_edges.is_empty() {
            return; // No critical edges found
        }

        raw_edges.sort_unstable_by(|a, b| {
            let (_, lhs_to, lhs_jt) = *a;
            let (_, rhs_to, rhs_jt) = *b;
            match lhs_to.cmp(&rhs_to) {
                std::cmp::Ordering::Equal => lhs_jt.cmp(&rhs_jt),
                ord => ord,
            }
        });

        let mut edge_left = self.edges.len() as u32;
        for (from, to, jt) in raw_edges {
            if self.info.last().map_or(false, |info| info.to == to) {
                // If the last edge is the same target, just append the edge
                self.edges.push(jt);
                self.info.last_mut().unwrap().edge_right += 1;
            } else {
                // Otherwise, create a new edge info
                self.info
                    .push(EdgeCriticalInfo { from, to, edge_left, edge_right: edge_left + 1 });
                self.edges.push(jt);
                edge_left += 1;
            }
        }
    }

    fn iter(&self) -> CriticalEdgeIter {
        CriticalEdgeIter { edges: self, index: -1 }
    }

    fn break_one(ir_module: &Module, edge: CriticalEdgeGroup, func: GlobalRef) -> BlockRef {
        // Step 0: Create a new block jumping to the target block
        let (common, jump) = Jump::new(ir_module, edge.to);
        let jump = ir_module.insert_inst(InstData::Jump(common, jump));

        let break_bb = BlockData::new_unreachable(ir_module).unwrap();
        break_bb
            .set_terminator(ir_module, jump)
            .expect("Failed to set terminator for break block");
        let break_bb = ir_module.insert_block(break_bb);

        // Step 1: Modify `PHI` nodes of the successor block
        let alloc_value = ir_module.borrow_value_alloc();
        let alloc_inst = &alloc_value.alloc_inst;
        let to_bb_data = ir_module.get_block(edge.to);

        let mut n_phi_nodes = 0;
        for (_, inst) in to_bb_data.instructions.view(alloc_inst) {
            if let InstData::Phi(_, phi) = inst {
                if phi.replace_from_bb_with_new(edge.from, break_bb, ir_module) {
                    n_phi_nodes += 1;
                }
            } else {
                // PHI nodes are always located at the beginning of the block.
                // Meeting non-phi node means we have processed all phi nodes.
                break;
            }
        }

        // We've constrained that `to_bb_data` has at least one PHI node.
        if n_phi_nodes == 0 {
            todo!("No PHI nodes found in the target block. This should not happen.");
        }

        // Step 2: Push new block into function body
        let alloc_block = &alloc_value.alloc_block;
        let alloc_global = &alloc_value.alloc_global;
        let parent_func = func;
        assert!(parent_func.is_nonnull());
        let parent_func = match parent_func.to_slabref_unwrap(alloc_global) {
            GlobalData::Func(func_data) => func_data,
            _ => panic!("Parent function of block must be a function"),
        };
        parent_func
            .get_blocks()
            .unwrap()
            .push_back_ref(alloc_block, break_bb)
            .expect("Failed to push new block into function body");

        // Step 3: Redirect the edge to the new block
        // 注意: 同一个 `edge` 内的所有 `JumpTargetRef` 都是重边.
        for &edge_ref in edge.edges {
            edge_ref.set_block(ir_module, break_bb);
        }
        // Final: return the new block reference
        break_bb
    }
}

struct CriticalEdgeIter<'a> {
    edges: &'a CriticalEdges,
    index: isize,
}

impl<'a> Iterator for CriticalEdgeIter<'a> {
    type Item = CriticalEdgeGroup<'a>;
    fn next(&mut self) -> Option<CriticalEdgeGroup<'a>> {
        self.index += 1;
        if self.index as usize >= self.edges.info.len() {
            return None; // No more critical edges
        }
        let edge_info = &self.edges.info[self.index as usize];
        let edges = &self.edges.edges[edge_info.edge_left as usize..edge_info.edge_right as usize];
        Some(CriticalEdgeGroup { from: edge_info.from, to: edge_info.to, edges })
    }
}

/// Remove critical edges from the IR module. Used in PHI-Node elimination.
///
/// ### How to scan
///
/// Suppose there are two basic blocks Ba and Bb with a directed edge `Jt = Ba -> Bb`.
/// If Ba has multiple successors and Bb has multiple predecessors, then Jt is a "critical edge".
///
/// ### How to remove
///
/// To remove a critical edge, we need to split the edge into two edges:
///
/// 1. Create a new basic block Bc.
/// 2. Insert Bc between Ba and Bb.
/// 3. Redirect the edge from Ba to Bc.
/// 4. Redirect the edge from Bc to Bb.
///
/// ### Notes
///
/// This pass has less memory usage if we perform a "mark and compact"
/// garbage collection before this pass.
pub fn break_critical_edges(ir_module: &Module) {
    for func in ir_module.dump_funcs(true) {
        let critical_edges = CriticalEdges::new_from_func(func, ir_module);
        if critical_edges.info.is_empty() {
            continue; // No critical edges to break
        }
        for edge in critical_edges.iter() {
            CriticalEdges::break_one(ir_module, edge, func);
        }
    }
}
