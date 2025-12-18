//! Dominator tree analysis for Remusys IR functions.
//!
//! 提供支配树和后支配树的构建与查询功能。

use crate::{
    base::DSU,
    ir::{BlockID, FuncID, IRAllocs, ISubInstID, InstID, InstOrdering, ListWalkOrder},
    opt::{CfgBlockStat, CfgDfsSeq, CfgRes, CfgSnapshot},
};
use smallvec::SmallVec;
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
    /// 直接支配结点的 DFS 编号, 用于快速查询.
    /// NULL value: CfgDfsSeq::NULL_PARENT
    pub idom_dfn: usize,
    /// 子结点的 DFS 编号集合, 用于快速查询某个节点是否是当前节点的子节点.
    pub children_dfn: SmallVec<[usize; 4]>,

    /// 缓存支配关系查询结果的集合.
    dominate_cache: RefCell<HashMap<BlockID, bool>>,
}
impl Default for DominatorTreeNode {
    fn default() -> Self {
        Self {
            block: CfgBlockStat::Virtual,
            semidom: CfgBlockStat::Virtual,
            idom: CfgBlockStat::Virtual,
            idom_dfn: CfgDfsSeq::NULL_PARENT,
            children_dfn: SmallVec::default(),
            dominate_cache: RefCell::default(),
        }
    }
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
    pub fn builder(allocs: &IRAllocs, func_id: FuncID) -> CfgRes<DominatorTreeBuilder> {
        DominatorTreeBuilder::new(allocs, func_id, false)
    }
    /// 创建一个后支配树构建器. 你需要调用 `build` 方法来实际构建后支配树.
    pub fn postdom_builder(allocs: &IRAllocs, func_id: FuncID) -> CfgRes<DominatorTreeBuilder> {
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

    pub fn dfn_dom_children(&self, dfn: usize) -> &[usize] {
        &self.nodes[dfn].children_dfn
    }

    pub fn write_to_dot(&self, allocs: &IRAllocs, writer: &mut dyn std::io::Write) {
        writeln!(writer, "digraph dominator_tree {{").unwrap();
        writeln!(writer, "  rankdir=TB;").unwrap();
        writeln!(writer, "  node [shape=rect];").unwrap();

        // Emit nodes: use DFS index as node id; label with block index or VROOT/VEXIT
        for (dfn, node) in self.dfs.nodes.iter().enumerate() {
            let label = match node.block {
                CfgBlockStat::Block(bid) => format!("{:#x}", bid.to_raw_index(allocs)),
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
    cfg: CfgSnapshot,
    is_postdom: bool,
}

impl DominatorTreeBuilder {
    pub fn new(allocs: &IRAllocs, func_id: FuncID, is_postdom: bool) -> CfgRes<Self> {
        use crate::opt::analysis::dfs::DfsOrder;
        let order = if is_postdom { DfsOrder::BackPre } else { DfsOrder::Pre };
        let dfs = CfgDfsSeq::new(allocs, func_id, order)?;
        let cfg = CfgSnapshot::new(allocs, func_id)?;
        Ok(Self { func_id, dfs, cfg, is_postdom })
    }

    pub fn build_with_relation<R>(self, relation: R) -> DominatorTree<R>
    where
        R: InstOrdering,
    {
        self.build().map_relation(relation)
    }
    pub fn build(self) -> DominatorTree {
        const ROOT_DFN: usize = 0;
        const BRANCH_START: usize = 1;

        let mut dfn_dsu = DSU::new(self.nnodes());
        let mut best_candidate = Box::from_iter(0..self.nnodes());
        let mut semidom = Box::from_iter(0..self.nnodes());
        let mut dt_nodes = {
            let mut dt_nodes = Vec::with_capacity(self.nnodes());
            for _ in 0..self.nnodes() {
                dt_nodes.push(DominatorTreeNode::default());
            }
            dt_nodes
        };

        for dfn in (BRANCH_START..self.nnodes()).rev() {
            let mut res = usize::MAX;
            let block_stat = self.dfs.dfn_block(dfn);
            let is_postdom = self.is_postdom;
            let preds = if is_postdom {
                self.cfg.succ_of(block_stat)
            } else {
                self.cfg.pred_of(block_stat)
            };

            let mut delegate_update_semidom = |pred_dfn: usize, res: usize| {
                dfn_dsu.find_when(pred_dfn, |pred_dfn, old_parent_dfn, _| {
                    let old_parent_elect = best_candidate[old_parent_dfn];
                    let pred_elect = best_candidate[pred_dfn];
                    if semidom[old_parent_elect] < semidom[pred_elect] {
                        best_candidate[pred_dfn] = old_parent_elect;
                    }
                });
                if pred_dfn < dfn {
                    res.min(pred_dfn)
                } else {
                    res.min(semidom[best_candidate[pred_dfn]])
                }
            };

            for &pred_bb in preds.unwrap_or(&[]) {
                let Some(pred_dfn) = self.dfs.try_block_dfn(pred_bb) else {
                    // pred_bb is unreachable so that we just ignore it
                    continue;
                };
                res = delegate_update_semidom(pred_dfn, res);
            }
            if self.block_ends_function(block_stat) && self.is_postdom {
                res = delegate_update_semidom(ROOT_DFN, res);
            }

            debug_assert_ne!(
                res,
                usize::MAX,
                "Internal error: it's confusing that a unreachable {block_stat:?} has a DFN"
            );
            semidom[dfn] = res;
            let dfs_parent = self.dfs.nodes[dfn].parent;
            dfn_dsu.set_direct_parent(dfn, dfs_parent);
        }

        for (dfn, &semi_dfn) in semidom.iter().enumerate() {
            let dt_node = &mut dt_nodes[dfn];
            dt_node.block = self.dfs.dfn_block(dfn);
            dt_node.semidom = self.dfs.dfn_block(semi_dfn);
        }

        for dfn in BRANCH_START..self.nnodes() {
            let semidom_dfn = semidom[dfn];
            let dfs_parent = self.dfs.nodes[dfn].parent;

            let mut idom_dfn = dfs_parent;
            while idom_dfn != ROOT_DFN && idom_dfn > semidom_dfn {
                idom_dfn = dt_nodes[idom_dfn].idom_dfn;
            }

            dt_nodes[dfn].idom = self.dfs.dfn_block(idom_dfn);
            dt_nodes[dfn].idom_dfn = idom_dfn;
            dt_nodes[idom_dfn].children_dfn.push(dfn);
        }

        DominatorTree {
            func_id: self.func_id,
            dfs: self.dfs,
            nodes: dt_nodes,
            inst_order: ListWalkOrder,
        }
    }

    fn nnodes(&self) -> usize {
        self.dfs.nodes.len()
    }
    fn block_ends_function(&self, block: CfgBlockStat) -> bool {
        let CfgBlockStat::Block(block) = block else {
            return false;
        };
        self.cfg.exits.contains(&block)
    }
}

pub struct DominanceFrontier<'ir, Order> {
    pub dom_tree: &'ir DominatorTree<Order>,
    pub df: Box<[HashSet<usize>]>,
    pub cfg: CfgSnapshot,
}
impl<'ir, Order> DominanceFrontier<'ir, Order> {
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

    pub fn new(dom_tree: &'ir DominatorTree<Order>, allocs: &'ir IRAllocs) -> CfgRes<Self> {
        let nnodes = dom_tree.nodes.len();
        let mut frontiers = vec![HashSet::new(); nnodes];
        let mut ret = Self {
            dom_tree,
            df: Box::new([]),
            cfg: CfgSnapshot::new(allocs, dom_tree.func_id)?,
        };

        // 递归计算每个节点的支配边界
        // 使用后序遍历，确保子节点的 frontier 先被计算
        for dfn in (0..nnodes).rev() {
            ret.calc_df_recursive(dfn, frontiers.as_mut_slice());
        }
        ret.df = frontiers.into_boxed_slice();
        Ok(ret)
    }

    fn calc_df_recursive(&self, curr_dfn: usize, frontiers: &mut [HashSet<usize>]) {
        let dt = &self.dom_tree;
        let dfs = &dt.dfs;
        let CfgBlockStat::Block(curr_bb) = dfs.dfn_block(curr_dfn) else {
            return; // 虚拟节点没有支配边界
        };

        // 第一步：计算 local frontier
        // DF_local(X) = {Y | Y是X的后继, 且X不严格支配Y}
        let succs = if dt.is_postdom() {
            self.cfg.pred_of(curr_bb).unwrap_or(&[])
        } else {
            self.cfg.succ_of(curr_bb).unwrap_or(&[])
        };
        for &succ_bb in succs {
            // X 不严格支配 Y 意味着：X != Y 且 X 不支配 Y
            let dominates = dt.block_strictly_dominates_block(curr_bb, succ_bb);
            if !dominates {
                let succ_dfn = dt.dfs.block_dfn(succ_bb);
                frontiers[curr_dfn].insert(succ_dfn);
            }
        }

        // 第二步：计算 up frontier
        // DF_up(X) = ∪{DF(Z) | Z是X在支配树中的子节点, 且X不严格支配DF(Z)中的节点}
        // let children_dfns = &dt.nodes[dfn].children_dfn;
        for &child_dfn in dt.dfn_dom_children(curr_dfn) {
            let child_idom_dfn = dt.nodes[child_dfn].idom_dfn;
            if child_idom_dfn == CfgDfsSeq::NULL_PARENT || child_idom_dfn != curr_dfn {
                continue;
            }
            let [frontier, child_frontier] =
                frontiers.get_disjoint_mut([curr_dfn, child_dfn]).unwrap();
            let child_frontier = child_frontier.iter().filter(|&&frontier_dfn| {
                let curr_bb = dfs.dfn_block(curr_dfn);
                let frontier_bb = dfs.dfn_block(frontier_dfn);
                !dt.block_strictly_dominates_block(curr_bb, frontier_bb)
            });
            frontier.extend(child_frontier);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{FuncID, IRWriteOption, ISubGlobalID, write_ir_to_file};
    use crate::testing::cases::{test_case_cfg_deep_while_br, test_case_minmax};
    use std::fs::File;

    #[test]
    fn dominance_basic_relations() {
        let module = test_case_minmax().module;
        write_ir_to_file("../target/predom-demo.ll", &module, IRWriteOption::loud());
        let fid = module
            .get_global_by_name("main")
            .map(FuncID::raw_from)
            .expect("func not found");
        let allocs = &module.allocs;

        let dom = DominatorTree::builder(allocs, fid).unwrap().build();
        let mut file = File::create("../target/test_dom.dot").expect("Failed to create dot file");
        dom.write_to_dot(allocs, &mut file);
    }

    #[test]
    fn postdom_basic_relations() {
        let builder = test_case_cfg_deep_while_br();
        let module = builder.module;
        write_ir_to_file("../target/postdom-demo.ll", &module, IRWriteOption::loud());
        let allocs = &module.allocs;
        let fid = module
            .get_global_by_name("main")
            .map(FuncID::raw_from)
            .expect("func not found");

        let post = DominatorTree::postdom_builder(allocs, fid).unwrap().build();
        let mut dot_file =
            File::create("../target/test_postdom.dot").expect("Failed to create dot file");
        post.write_to_dot(allocs, &mut dot_file);
    }
}
