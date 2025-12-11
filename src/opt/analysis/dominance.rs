//! Dominator tree analysis for Remusys IR functions.
//!
//! 提供支配树和后支配树的构建与查询功能。

use crate::{
    base::DSU,
    ir::{BlockID, FuncID, IRAllocs, ISubInstID, InstID, InstOrdering, ListWalkOrder},
    opt::{CfgBlockStat, CfgCache, CfgDfsSeq, CfgSnapshot},
};
use smallvec::SmallVec;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    vec,
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
    /// 直接支配结点的 DFS 编号, 用于快速查询.
    pub idom_dfn: usize,
    /// 子结点的 DFS 编号集合, 用于快速查询某个节点是否是当前节点的子节点.
    pub children_dfn: SmallVec<[usize; 4]>,

    /// 缓存支配关系查询结果的集合.
    dominate_cache: RefCell<HashMap<BlockID, bool>>,
}

pub struct DominatorTree<Order = ListWalkOrder> {
    /// 所属函数 ID.
    pub func_id: FuncID,
    /// DFS 序列. 根据是支配树还是后支配树, 其遍历顺序会有所不同.
    /// 支配树使用的遍历顺序是 `Pre`, 后支配树使用的遍历顺序是 `BackPre`.
    pub dfs: CfgDfsSeq,
    /// 支配树节点列表, 按 DFS 序列顺序排列.
    pub nodes: Vec<DominatorTreeNode>,
    /// 存储 “一条指令是否为另一条指令” 的数据结构
    pub inst_order: Order,
}

impl DominatorTree {
    /// 创建一个支配树构建器. 你需要调用 `build` 方法来实际构建支配树.
    pub fn builder(allocs: &IRAllocs, func_id: FuncID) -> DominatorTreeBuilder {
        DominatorTreeBuilder::new(allocs, func_id, false)
    }
    /// 创建一个后支配树构建器. 你需要调用 `build` 方法来实际构建后支配树.
    pub fn postdom_builder(allocs: &IRAllocs, func_id: FuncID) -> DominatorTreeBuilder {
        DominatorTreeBuilder::new(allocs, func_id, true)
    }
}

impl<Order> DominatorTree<Order> {
    /// 根节点在 DFS 序列中的索引.
    pub const ROOT_INDEX: usize = 0;

    pub fn map_relation<S: InstOrdering>(self, relation: S) -> DominatorTree<S> {
        let Self { func_id, dfs, nodes, .. } = self;
        DominatorTree { func_id, dfs, nodes, inst_order: relation }
    }
    pub fn get_kind(&self) -> DomiTreeKind {
        if self.is_postdom() { DomiTreeKind::PostDom } else { DomiTreeKind::Dom }
    }
    pub fn is_postdom(&self) -> bool {
        self.dfs.order.is_back()
    }
    pub fn root_node(&self) -> &DominatorTreeNode {
        &self.nodes[0]
    }

    pub fn write_to_dot(&self, allocs: &IRAllocs, writer: &mut dyn std::io::Write) {
        writeln!(writer, "digraph dominator_tree {{").unwrap();
        writeln!(writer, "  rankdir=TB;").unwrap();
        writeln!(writer, "  node [shape=rect];").unwrap();

        // Emit nodes: use DFS index as node id; label with block index or VROOT/VEXIT
        for (dfn, node) in self.dfs.nodes.iter().enumerate() {
            let label = match node.block {
                CfgBlockStat::Block(bid) => format!("{:#x}", bid.get_indexed_id(allocs)),
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
            let from_id = node.idom_dfn;
            let to_id = dfn;
            if from_id != CfgDfsSeq::NULL_PARENT {
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
            current_dfn = current_node.idom_dfn;
        }
        a_node.dominate_cache.borrow_mut().insert(b, false);
        false
    }
    pub fn block_strictly_dominates_block(
        &self,
        a: impl Into<CfgBlockStat>,
        b: impl Into<CfgBlockStat>,
    ) -> bool {
        let a = a.into();
        let b = b.into();
        if a == b {
            return false;
        }
        self.block_dominates_block(a, b)
    }

    pub fn inst_dominates_block(&self, allocs: &IRAllocs, inst: InstID, block: BlockID) -> bool {
        let inst_block = match inst.get_parent(allocs) {
            Some(bb) => bb,
            None => return false,
        };
        self.block_dominates_block(inst_block, block)
    }

    pub fn inst_dominates_inst(&self, allocs: &IRAllocs, a: InstID, b: InstID) -> bool
    where
        Order: InstOrdering,
    {
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
        } else if !self.is_postdom() {
            // Same block: use instruction order
            self.inst_order.comes_before(allocs, a, b)
        } else {
            // Postdom: reverse instruction order
            self.inst_order.comes_before(allocs, b, a)
        }
    }
    pub fn inst_strictly_dominates_inst(&self, allocs: &IRAllocs, a: InstID, b: InstID) -> bool
    where
        Order: InstOrdering,
    {
        if a == b {
            return false;
        }
        self.inst_dominates_inst(allocs, a, b)
    }
}

pub struct DominatorTreeBuilder {
    func_id: FuncID,
    dfs: CfgDfsSeq,
    cache: CfgCache,
    is_postdom: bool,
}

impl DominatorTreeBuilder {
    pub fn new(allocs: &IRAllocs, func_id: FuncID, is_postdom: bool) -> Self {
        use crate::opt::analysis::dfs::DfsOrder;
        let order = if is_postdom { DfsOrder::BackPre } else { DfsOrder::Pre };
        let dfs = CfgDfsSeq::new(allocs, func_id, order).expect("Failed to build DFS for function");
        let cache = CfgCache::new();
        Self { func_id, dfs, cache, is_postdom }
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
                idom: CfgBlockStat::Virtual,      // placeholder
                idom_dfn: CfgDfsSeq::NULL_PARENT, // placeholder
                children_dfn: SmallVec::new(),
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
            nodes[w].idom_dfn = id;
            nodes[w].idom = id_block;
            if id != CfgDfsSeq::NULL_PARENT {
                nodes[id].children_dfn.push(w);
            }
        }

        let Self { func_id, dfs, .. } = self;
        DominatorTree { func_id, dfs, nodes, inst_order: ListWalkOrder }
    }
    pub fn build_with_relation<R>(self, allocs: &IRAllocs, relation: R) -> DominatorTree<R>
    where
        R: InstOrdering,
    {
        self.build(allocs).map_relation(relation)
    }
}

pub struct DominanceFrontier<'ir, Order> {
    pub dom_tree: &'ir DominatorTree<Order>,
    pub df: Box<[HashSet<usize>]>,
    pub cfg: CfgSnapshot,
}
impl<'ir, Order> DominanceFrontier<'ir, Order> {
    pub fn new(dom_tree: &'ir DominatorTree<Order>, allocs: &'ir IRAllocs) -> Self {
        let mut builder = DominanceFrontierBuilder::new(dom_tree, allocs);
        builder.build();
        Self {
            dom_tree,
            df: builder.df.into_boxed_slice(),
            cfg: builder.cfg,
        }
    }

    pub fn get_df_of_block(&self, block: impl Into<CfgBlockStat>) -> Option<&HashSet<usize>> {
        let block = block.into();
        let dfn = match block {
            CfgBlockStat::Block(bid) => self.dom_tree.dfs.try_block_dfn(bid)?,
            CfgBlockStat::Virtual => {
                if self.dom_tree.is_postdom() {
                    self.dom_tree.dfs.virt_index?
                } else {
                    return None;
                }
            }
        };
        self.df.get(dfn)
    }
}

pub struct DominanceFrontierBuilder<'ir, Order = ListWalkOrder> {
    dom_tree: &'ir DominatorTree<Order>,
    pub df: Vec<HashSet<usize>>,
    pub cfg: CfgSnapshot,
}
impl<'ir, Order> DominanceFrontierBuilder<'ir, Order> {
    pub fn new(dom_tree: &'ir DominatorTree<Order>, allocs: &'ir IRAllocs) -> Self {
        Self {
            dom_tree,
            df: Vec::new(),
            cfg: CfgSnapshot::new(allocs, dom_tree.func_id).unwrap(),
        }
    }

    pub fn build(&mut self) {
        if !self.df.is_empty() {
            return;
        }
        let nnodes = self.get_dfs_seq().nodes.len();
        let mut df = vec![HashSet::new(); nnodes];
        self.post_order_dfs(&mut df, 0);
    }

    fn get_dfs_seq(&self) -> &CfgDfsSeq {
        &self.dom_tree.dfs
    }

    pub fn post_order_dfs(&mut self, df: &mut [HashSet<usize>], node: usize) {
        let dom_node = &self.dom_tree.nodes[node];
        for &child_dfn in &dom_node.children_dfn {
            self.post_order_dfs(df, child_dfn);
        }

        let succs: &[BlockID] = match (dom_node.block, self.dom_tree.is_postdom()) {
            (CfgBlockStat::Block(block), true) => self.cfg.succ_of(block).unwrap(),
            (CfgBlockStat::Block(block), false) => self.cfg.pred_of(block).unwrap(),
            (CfgBlockStat::Virtual, true) => self.cfg.exits.as_slice(),
            (CfgBlockStat::Virtual, false) => &[],
        };
        for &succ in succs {
            let succ_dfn = match self.get_dfs_seq().try_block_dfn(succ) {
                Some(dfn) => dfn,
                None => continue,
            };
            if self.dom_tree.nodes[succ_dfn].idom_dfn != node {
                df[node].insert(succ_dfn);
            }
        }

        for &child_dfn in &dom_node.children_dfn {
            let [child_df, node_df] = df.get_disjoint_mut([child_dfn, node]).unwrap();
            for &w in child_df.iter() {
                if self.dom_tree.nodes[w].idom_dfn != node {
                    node_df.insert(w);
                }
            }
        }
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
