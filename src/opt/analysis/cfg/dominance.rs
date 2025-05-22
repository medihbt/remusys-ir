//! Dominance Tree Analysis

use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

use slab::Slab;

use crate::{
    base::{NullableValue, dsu::DSU, slablist::SlabRefListNodeRef},
    ir::{
        block::BlockRef,
        inst::{InstData, InstRef},
        module::Module,
    },
    opt::util::DfsOrder,
};

use super::{
    dfs::{CfgDfsNode, CfgDfsSeq},
    snapshot::CfgSnapshot,
};

pub struct DominatorTreeNode {
    pub blockref: BlockRef,
    pub dfn_pre: usize,
    pub dfn_rpo: usize,

    pub semidom_block: BlockRef,
    pub semidom_pre_dfn: usize,
    pub idom_block: BlockRef,
    pub idom_pre_dfn: usize,

    pub dominator_cache: RefCell<BTreeSet<BlockRef>>,
}

pub struct DominatorTree {
    pub dfs_seq_pre: Rc<CfgDfsSeq>,
    pub dfs_seq_rpo: Rc<CfgDfsSeq>,
    pub rpo2pre_map: Box<[usize]>,

    /// Dominator tree nodes, arranged in pre-order DFS sequence.
    pub nodes: Vec<DominatorTreeNode>,

    pub root: BlockRef,
}

impl DominatorTree {
    pub fn get_nnodes(&self) -> usize {
        self.nodes.len()
    }
    pub fn pre_dfn_get_node(&self, pre_dfn: usize) -> Option<&DominatorTreeNode> {
        self.nodes.get(pre_dfn)
    }
    pub fn pre_dfn_node_mut(&mut self, pre_dfn: usize) -> Option<&mut DominatorTreeNode> {
        self.nodes.get_mut(pre_dfn)
    }
    pub fn pre_dfn_get_block(&self, pre_dfn: usize) -> Option<BlockRef> {
        self.pre_dfn_get_node(pre_dfn).map(|node| node.blockref)
    }
    pub fn pre_dfn_get_idom(&self, pre_dfn: usize) -> Option<BlockRef> {
        self.pre_dfn_get_node(pre_dfn).map(|node| node.idom_block)
    }
    pub fn pre_dfn_get_semidom(&self, pre_dfn: usize) -> Option<BlockRef> {
        self.pre_dfn_get_node(pre_dfn).map(|node| node.semidom_block)
    }
    pub fn pre_dfn_get_dfsnode(&self, pre_dfn: usize) -> Option<&CfgDfsNode> {
        self.dfs_seq_pre.dfn_get_node(pre_dfn)
    }
    pub fn pre_dfn_get_rpo(&self, pre_dfn: usize) -> Option<usize> {
        self.pre_dfn_get_node(pre_dfn).map(|node| node.dfn_rpo)
    }

    pub fn rpo_dfn_get_pre(&self, rpo_dfn: usize) -> Option<usize> {
        self.rpo2pre_map.get(rpo_dfn).copied()
    }
    pub fn rpo_dfn_get_block(&self, rpo_dfn: usize) -> Option<BlockRef> {
        self.rpo_dfn_get_pre(rpo_dfn)
            .and_then(|pre_dfn| self.pre_dfn_get_block(pre_dfn))
    }
    pub fn rpo_dfn_get_idom(&self, rpo_dfn: usize) -> Option<BlockRef> {
        self.rpo_dfn_get_pre(rpo_dfn)
            .and_then(|pre_dfn| self.pre_dfn_get_idom(pre_dfn))
    }
    pub fn rpo_dfn_get_semidom(&self, rpo_dfn: usize) -> Option<BlockRef> {
        self.rpo_dfn_get_pre(rpo_dfn)
            .and_then(|pre_dfn| self.pre_dfn_get_semidom(pre_dfn))
    }
    pub fn rpo_dfn_get_dfsnode(&self, rpo_dfn: usize) -> Option<&CfgDfsNode> {
        self.rpo_dfn_get_pre(rpo_dfn)
            .and_then(|pre_dfn| self.dfs_seq_pre.dfn_get_node(pre_dfn))
    }

    pub fn block_get_pre_dfn(&self, block: BlockRef) -> Option<usize> {
        self.dfs_seq_pre.block_get_dfn(block)
    }
    pub fn block_get_rpo_dfn(&self, block: BlockRef) -> Option<usize> {
        self.dfs_seq_rpo.block_get_dfn(block)
    }
    pub fn block_get_node(&self, block: BlockRef) -> Option<&DominatorTreeNode> {
        self.nodes.get(self.block_get_pre_dfn(block)?)
    }
    pub fn block_get_idom(&self, block: BlockRef) -> Option<BlockRef> {
        self.block_get_node(block).map(|node| node.idom_block)
    }
    pub fn block_get_semidom(&self, block: BlockRef) -> Option<BlockRef> {
        self.block_get_node(block).map(|node| node.semidom_block)
    }
    pub fn block_is_reachable(&self, block: BlockRef) -> bool {
        self.dfs_seq_pre.block_is_reachable(block)
    }
    pub fn dump_dfs_seq_rpo(&self) -> CfgDfsSeq {
        CfgDfsSeq::clone(&self.dfs_seq_rpo)
    }
    pub fn dump_dfs_seq_pre(&self) -> CfgDfsSeq {
        CfgDfsSeq::clone(&self.dfs_seq_pre)
    }

    pub fn block_dominates_block(&self, domaintor: BlockRef, dominee: BlockRef) -> bool {
        if domaintor == dominee {
            return true;
        }
        if !self.block_is_reachable(domaintor) || !self.block_is_reachable(dominee) {
            return false;
        }
        let domaintor_dfn = self.block_get_pre_dfn(domaintor).unwrap();
        let dominee_dfn = self.block_get_pre_dfn(dominee).unwrap();
        if domaintor_dfn <= dominee_dfn {
            return false;
        }
        if self
            .pre_dfn_get_node(dominee_dfn)
            .unwrap()
            .dominator_cache
            .borrow()
            .contains(&domaintor)
        {
            return true;
        }
        let mut cur = dominee;
        while cur != self.root {
            if cur == domaintor {
                // Maintain the dominator cache.
                self.pre_dfn_get_node(dominee_dfn)
                    .unwrap()
                    .dominator_cache
                    .borrow_mut()
                    .insert(domaintor);
                return true;
            }
            cur = self.block_get_idom(cur).unwrap();
        }
        false
    }

    pub fn inst_dominates_block(&self, module: &Module, inst: InstRef, block: BlockRef) -> bool {
        let inst_block = inst.get_parent(module).unwrap();
        if inst_block == block {
            return true;
        }
        return self.block_dominates_block(inst_block, block);
    }
    pub fn inst_dominates_block_by_alloc(
        &self,
        alloc_inst: &Slab<InstData>,
        inst: InstRef,
        block: BlockRef,
    ) -> bool {
        let inst_block = inst.get_parent_from_alloc(alloc_inst).unwrap();
        if inst_block == block {
            return true;
        }
        return self.block_dominates_block(inst_block, block);
    }

    pub fn inst_dominates_inst(&self, module: &Module, inst1: InstRef, inst2: InstRef) -> bool {
        if inst1 == inst2 {
            return true;
        }
        let inst1_block = inst1.get_parent(module).unwrap();
        let inst2_block = inst2.get_parent(module).unwrap();
        if inst1_block == inst2_block {
            return inst1.comes_before_node(inst2, &module.borrow_value_alloc().alloc_inst);
        }
        return self.block_dominates_block(inst1_block, inst2_block);
    }
    pub fn inst_dominates_inst_by_alloc(
        &self,
        alloc_inst: &Slab<InstData>,
        inst1: InstRef,
        inst2: InstRef,
    ) -> bool {
        if inst1 == inst2 {
            return true;
        }
        let inst1_block = inst1.get_parent_from_alloc(alloc_inst).unwrap();
        let inst2_block = inst2.get_parent_from_alloc(alloc_inst).unwrap();
        if inst1_block == inst2_block {
            return inst1.comes_before_node(inst2, alloc_inst);
        }
        return self.block_dominates_block(inst1_block, inst2_block);
    }
}

impl DominatorTree {
    pub fn new_empty(dfs_pre: Rc<CfgDfsSeq>, dfs_rpo: Rc<CfgDfsSeq>) -> Self {
        let mut rpo2pre_map = vec![0; dfs_rpo.get_nnodes()];
        let mut pre2rpo_map = vec![0; dfs_pre.get_nnodes()];
        for (block, rpo_dfn) in &dfs_rpo.dfn {
            let pre_dfn = dfs_pre.block_get_dfn(*block).unwrap();
            rpo2pre_map[*rpo_dfn] = pre_dfn;
            pre2rpo_map[pre_dfn] = *rpo_dfn;
        }
        let mut nodes = Vec::with_capacity(dfs_pre.get_nnodes());
        for pre_dfn in 0..dfs_pre.get_nnodes() {
            nodes.push(DominatorTreeNode {
                blockref: dfs_pre.dfn_get_block(pre_dfn).unwrap(),
                dfn_pre: pre_dfn,
                dfn_rpo: pre2rpo_map[pre_dfn],
                semidom_block: BlockRef::new_null(),
                idom_block: BlockRef::new_null(),
                dominator_cache: RefCell::new(BTreeSet::new()),
                semidom_pre_dfn: usize::MAX,
                idom_pre_dfn: usize::MAX,
            });
        }

        let root = dfs_pre.get_root();
        Self {
            dfs_seq_pre: dfs_pre,
            dfs_seq_rpo: dfs_rpo,
            rpo2pre_map: rpo2pre_map.into_boxed_slice(),
            nodes: nodes,
            root,
        }
    }

    /// `Remusys-IR` uses Semi-NCA algorithm to build dominator tree.
    /// Relavent notes can be found in the documentation of `Remusys-IR`.
    pub fn new_from_snapshot(snapshot: &CfgSnapshot) -> Self {
        let dfs_seq_rpo = Rc::new(CfgDfsSeq::new_from_snapshot(
            snapshot,
            DfsOrder::ReversePost,
        ));
        let dfs_seq_pre = Rc::new(CfgDfsSeq::new_from_snapshot(snapshot, DfsOrder::Pre));
        let mut dominator_tree = Self::new_empty(dfs_seq_pre, dfs_seq_rpo);
        dominator_tree._build_semidom(snapshot);
        dominator_tree._build_idom();
        dominator_tree
    }

    fn _build_semidom(&mut self, snapshot: &CfgSnapshot) -> DSU {
        let mut pre_dfn_dsu = DSU::new(self.get_nnodes());
        let mut sdom_best_elect = (0..self.get_nnodes()).collect::<Box<_>>();
        let mut semidom_dfn = (0..self.get_nnodes()).collect::<Box<_>>();

        for u in (1..self.get_nnodes()).rev() {
            let mut res = usize::MAX;
            let u_block = self.pre_dfn_get_block(u).unwrap();
            let prev = match snapshot.block_get_prev(u_block) {
                Some(prev) => prev,
                None => continue,
            };
            for (_, blockref) in prev {
                let v = match self.dfs_seq_pre.block_get_dfn(*blockref) {
                    Some(v) => v,
                    None => continue,
                };
                pre_dfn_dsu.find_when(v, |x: usize, old_parent_dfn, _| {
                    let old_parent_elect = sdom_best_elect[old_parent_dfn];
                    let x_elect = sdom_best_elect[x];
                    if semidom_dfn[old_parent_elect] < semidom_dfn[x_elect] {
                        sdom_best_elect[x] = old_parent_elect;
                    }
                });

                res = if v < u {
                    res.min(v)
                } else {
                    res.min(semidom_dfn[sdom_best_elect[v]])
                };
            }
            semidom_dfn[u] = res;
            let parent = self.dfs_seq_pre.dfn_get_parent_dfn(u).unwrap();
            pre_dfn_dsu.set_direct_parent(u, parent);
        }

        for (node, &semidom) in semidom_dfn.iter().enumerate() {
            let semidom_bb = self.pre_dfn_get_block(semidom).unwrap();
            let node = self.pre_dfn_node_mut(node).unwrap();
            node.semidom_block = semidom_bb;
            node.semidom_pre_dfn = semidom;
        }

        pre_dfn_dsu
    }
    fn _build_idom(&mut self) {}
}
