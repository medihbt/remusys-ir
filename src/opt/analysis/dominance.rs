use crate::{
    base::DSU,
    ir::{BlockID, FuncID, IRAllocs, ISubInstID, InstID},
    opt::{CfgBlockStat, CfgCache, CfgDfsSeq},
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

pub enum DomiTreeKind {
    Dom,
    PostDom,
}

pub struct DominatorTreeNode {
    pub block: CfgBlockStat,
    /// 半支配结点. 使用 CfgBlockStat 来考虑根节点是 Virtual Exit 的情况.
    pub semidom: CfgBlockStat,
    /// 直接支配结点. 使用 CfgBlockStat 来考虑根节点是 Virtual Exit 的情况.
    pub idom: CfgBlockStat,
    /// 所有的子结点. 与重构前不同的是, 这次不要懒加载，在构建时就要把所有子结点算出来.
    pub children: HashSet<BlockID>,

    /// 缓存支配关系查询结果的集合.
    dominate_cache: RefCell<HashMap<BlockID, bool>>,
}

pub struct DominatorTree {
    pub func_id: FuncID,
    pub dfs: CfgDfsSeq,
    pub nodes: Vec<DominatorTreeNode>,
    inst_orders: HashMap<InstID, usize>,
}

impl DominatorTree {
    pub fn builder(allocs: &IRAllocs, func_id: FuncID) -> DominatorTreeBuilder {
        DominatorTreeBuilder::new(allocs, func_id, false)
    }
    pub fn postdom_builder(allocs: &IRAllocs, func_id: FuncID) -> DominatorTreeBuilder {
        DominatorTreeBuilder::new(allocs, func_id, true)
    }

    pub fn get_kind(&self) -> DomiTreeKind {
        if self.is_postdom() { DomiTreeKind::PostDom } else { DomiTreeKind::Dom }
    }

    pub fn is_postdom(&self) -> bool {
        self.dfs.order.is_back()
    }

    pub fn write_to_dot(&self, allocs: &IRAllocs, writer: &mut dyn std::io::Write) {
        writeln!(writer, "digraph dominator_tree {{").unwrap();
        writeln!(writer, "  rankdir=TB;").unwrap();
        writeln!(writer, "  node [shape=rect];").unwrap();

        // Emit nodes: use DFS index as node id; label with block index or VROOT/VEXIT
        for (dfn, node) in self.dfs.nodes.iter().enumerate() {
            let label = match node.block {
                CfgBlockStat::Block(bid) => format!("{:#x}", bid.get_indexed(allocs)),
                CfgBlockStat::Virtual => {
                    if self.is_postdom() {
                        "%VEXIT".to_string()
                    } else {
                        "%[INVALID NODE]".to_string()
                    }
                }
            };
            writeln!(writer, "  {dfn} [label=\"{label}\"];").unwrap();
        }

        // Emit idom edges: idom -> node
        for (dfn, node) in self.nodes.iter().enumerate() {
            let from_id = match node.idom {
                CfgBlockStat::Virtual => {
                    if self.is_postdom() {
                        0
                    } else {
                        continue;
                    } // virtual root/exit
                }
                CfgBlockStat::Block(bid) => match self.dfs.try_block_dfn(bid) {
                    Some(dfn) => dfn,
                    None => continue,
                },
            };
            let to_id = dfn;
            if from_id != to_id {
                writeln!(writer, "  {from_id} -> {to_id};").unwrap();
            }
        }
        writeln!(writer, "}}").unwrap();
    }

    pub fn block_dominates_block(
        &self,
        a: impl Into<CfgBlockStat>,
        b: impl Into<CfgBlockStat>,
    ) -> bool {
        let a = a.into();
        let b = b.into();
        let (a, b) = match (a, b) {
            (CfgBlockStat::Block(ab), CfgBlockStat::Block(bb)) => (ab, bb),
            (CfgBlockStat::Virtual, CfgBlockStat::Virtual) => return true, // Virtual block dominates itself
            // Normal dom: no virtual exit block
            (CfgBlockStat::Block(_), CfgBlockStat::Virtual) => return !self.is_postdom(),
            // Postdom: virtual exit block dominates all normal blocks
            (CfgBlockStat::Virtual, CfgBlockStat::Block(_)) => return self.is_postdom(),
        };

        let a_dfn = match self.dfs.try_block_dfn(a) {
            Some(dfn) => dfn,
            None => return false,
        };
        let b_dfn = match self.dfs.try_block_dfn(b) {
            Some(dfn) => dfn,
            None => return false,
        };
        let a_node: &DominatorTreeNode = &self.nodes[a_dfn];
        if let Some(&cached_result) = a_node.dominate_cache.borrow().get(&b) {
            return cached_result;
        }
        // Walk up from b to root using idom links
        let mut current_dfn = b_dfn;
        while current_dfn != CfgDfsSeq::NULL_PARENT {
            if current_dfn == a_dfn {
                a_node.dominate_cache.borrow_mut().insert(b, true);
                return true;
            }
            let current_node: &DominatorTreeNode = &self.nodes[current_dfn];
            current_dfn = match current_node.idom {
                CfgBlockStat::Block(bid) => match self.dfs.try_block_dfn(bid) {
                    Some(dfn) => dfn,
                    None => break,
                },
                CfgBlockStat::Virtual => CfgDfsSeq::NULL_PARENT,
            };
        }
        a_node.dominate_cache.borrow_mut().insert(b, false);
        false
    }

    pub fn inst_dominates_inst(&self, allocs: &IRAllocs, a: InstID, b: InstID) -> bool {
        if a == b {
            return true;
        }
        let a_block = match a.get_parent(allocs) {
            Some(bb) => bb,
            None => return false,
        };
        let b_block = match b.get_parent(allocs) {
            Some(bb) => bb,
            None => return false,
        };
        if a_block != b_block {
            self.block_dominates_block(a_block, b_block)
        } else {
            // Same block: use instruction order
            let a_order = match self.inst_orders.get(&a) {
                Some(&ord) => ord,
                None => return false,
            };
            let b_order = match self.inst_orders.get(&b) {
                Some(&ord) => ord,
                None => return false,
            };
            a_order < b_order
        }
    }
}

pub struct DominatorTreeBuilder {
    func_id: FuncID,
    dfs: CfgDfsSeq,
    cache: CfgCache,
    is_postdom: bool,
    inst_orders: HashMap<InstID, usize>,
}

impl DominatorTreeBuilder {
    pub fn new(allocs: &IRAllocs, func_id: FuncID, is_postdom: bool) -> Self {
        use crate::opt::analysis::dfs::DfsOrder;
        let order = if is_postdom { DfsOrder::BackPre } else { DfsOrder::Pre };
        let dfs = CfgDfsSeq::new(allocs, func_id, order).expect("Failed to build DFS for function");
        let cache = CfgCache::new();
        let inst_orders = Self::build_orders(func_id, allocs);
        Self { func_id, dfs, cache, is_postdom, inst_orders }
    }

    fn build_orders(func_id: FuncID, allocs: &IRAllocs) -> HashMap<InstID, usize> {
        let mut orders = HashMap::new();
        for (bb_id, _) in func_id.get_blocks(allocs).unwrap().iter(&allocs.blocks) {
            let insts = bb_id.get_insts(allocs);
            for (count, (inst, _)) in insts.iter(&allocs.insts).enumerate() {
                orders.insert(inst, count);
            }
        }
        orders
    }

    pub fn build(mut self, allocs: &IRAllocs) -> DominatorTree {
        // Prepare mappings
        let n = self.dfs.nodes.len();
        assert!(n > 0, "DFS must contain at least root node");

        // preds_by_dfn[u] -> Vec<dfn of predecessors of u>
        let mut preds_by_dfn: Vec<Vec<usize>> = vec![Vec::new(); n];

        // Build predecessor index depending on dom/postdom
        for (dfn, node) in self.dfs.nodes.iter().enumerate() {
            let bb = match node.block {
                CfgBlockStat::Block(b) => b,
                CfgBlockStat::Virtual if !self.is_postdom => continue,
                CfgBlockStat::Virtual => {
                    // Virtual Exit: predecessors are all exit blocks
                    let exit_dfns = self.dfs.backward_get_exit_dfns().expect(
                        "Internal error: Postdom building should have a backward virtual root",
                    );
                    preds_by_dfn[dfn].extend_from_slice(exit_dfns);
                    continue;
                }
            };
            let build_towards = if self.is_postdom {
                // Postdominator tree: on backward graph, "successors" are original predecessors
                self.cache.get_preds(allocs, bb)
            } else {
                // Dominator tree: use successors to build reverse (preds of each node)
                self.cache.get_succs(allocs, bb)
            };
            for &toward in build_towards {
                if let Some(toward_dfn) = self.dfs.try_block_dfn(toward) {
                    preds_by_dfn[dfn].push(toward_dfn);
                }
            }
        }

        // Semi-NCA arrays
        let mut semidom: Vec<usize> = (0..n).collect();
        let mut idom: Vec<usize> = vec![usize::MAX; n];
        let mut best: Vec<usize> = (0..n).collect();

        // DSU over DFNs
        let mut dsu = DSU::new(n);

        // Compute semidominators in reverse DFS order (skip root 0)
        for u in (1..n).rev() {
            let mut candidate = usize::MAX;
            for &v in &preds_by_dfn[u] {
                dsu.find_when(v, |x, parent_dfn, _| {
                    let old = best[parent_dfn];
                    let bx = best[x];
                    if semidom[old] < semidom[bx] {
                        best[x] = old;
                    }
                });
                let y = best[v];
                let m = if v < u { v } else { semidom[y] };
                if m < candidate {
                    candidate = m;
                }
            }
            if candidate == usize::MAX {
                // No reachable predecessors: default to parent
                candidate = self.dfs.nodes[u].parent;
            }
            semidom[u] = candidate;
            let parent = self.dfs.nodes[u].parent;
            if parent != CfgDfsSeq::NULL_PARENT {
                dsu.set_direct_parent(u, parent);
            }
        }

        // Prepare nodes vector
        let mut nodes: Vec<DominatorTreeNode> = Vec::with_capacity(n);
        for (dfn, &semidom) in semidom.iter().enumerate() {
            let block = self.dfs.dfn_block(dfn);
            let sdom_blk = self.dfs.dfn_block(semidom);
            nodes.push(DominatorTreeNode {
                block,
                semidom: sdom_blk,
                idom: CfgBlockStat::Virtual, // placeholder
                children: HashSet::new(),
                dominate_cache: RefCell::new(HashMap::new()),
            });
        }

        // Compute idom by Semi-NCA correction
        for w in 1..n {
            let w_sdom = semidom[w];
            let mut id = self.dfs.nodes[w].parent;
            while id != 0 && semidom[id] > w_sdom {
                id = idom[id];
            }
            idom[w] = id;
        }
        idom[0] = CfgDfsSeq::NULL_PARENT; // root has no idom

        // Write idom blocks and build children sets
        for w in 0..n {
            let id = idom[w];
            let id_block = if id == CfgDfsSeq::NULL_PARENT {
                CfgBlockStat::Virtual
            } else {
                self.dfs.dfn_block(id)
            };
            nodes[w].idom = id_block;
            if id != CfgDfsSeq::NULL_PARENT {
                let child_block = match nodes[w].block {
                    CfgBlockStat::Block(b) => b,
                    CfgBlockStat::Virtual => continue,
                };
                nodes[id].children.insert(child_block);
            }
        }

        let Self { func_id, dfs, inst_orders, .. } = self;
        DominatorTree { func_id, dfs, nodes, inst_orders }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use crate::ir::{FuncID, ISubGlobalID};
    use crate::testing::cases::test_case_cfg_deep_while_br;

    #[test]
    fn dominance_basic_relations() {
        let module = test_case_cfg_deep_while_br().module;
        let fid = module
            .get_global_by_name("main")
            .map(FuncID::raw_from)
            .expect("func not found");
        let allocs = &module.allocs;

        let dom = DominatorTree::builder(allocs, fid).build(allocs);
        let mut file = File::create("target/test_dom.dot").expect("Failed to create dot file");
        dom.write_to_dot(allocs, &mut file);
    }

    #[test]
    fn postdom_basic_relations() {
        let builder = test_case_cfg_deep_while_br();
        let module = builder.module;
        let allocs = &module.allocs;
        let fid = module
            .get_global_by_name("main")
            .map(FuncID::raw_from)
            .expect("func not found");

        let post = DominatorTree::postdom_builder(allocs, fid).build(allocs);
        let mut dot_file =
            File::create("target/test_postdom.dot").expect("Failed to create dot file");
        post.write_to_dot(allocs, &mut dot_file);
    }
}
