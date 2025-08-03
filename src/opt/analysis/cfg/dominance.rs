//! Dominance Tree Analysis

use super::{
    dfs::{CfgDfsNode, CfgDfsSeq},
    snapshot::CfgSnapshot,
};
use crate::{
    base::{DSU, INullableValue, SlabListNodeRef},
    ir::{
        block::BlockRef,
        inst::{InstData, InstRef},
        module::Module,
        util::numbering::IRValueNumberMap,
    },
    opt::util::DfsOrder,
};
use slab::Slab;
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::BTreeSet,
    rc::Rc,
};

pub struct DominatorTreeNode {
    pub blockref: BlockRef,
    pub dfn: usize,

    pub semidom_block: BlockRef,
    pub semidom_dfn: usize,
    pub idom_block: BlockRef,
    pub idom_dfn: usize,

    pub dominator_cache: RefCell<BTreeSet<BlockRef>>,
}

pub struct DominatorTree {
    pub dfs_seq: Rc<CfgDfsSeq>,
    /// Dominator tree nodes, arranged in DFS sequence of `dfs_seq`.
    pub nodes: Vec<DominatorTreeNode>,
    pub root: BlockRef,
    pub is_postdom: bool,
}

impl DominatorTree {
    pub fn get_nnodes(&self) -> usize {
        self.nodes.len()
    }
    pub fn get_dfs_order(&self) -> DfsOrder {
        self.dfs_seq.order
    }
    pub fn dfn_get_node(&self, dfn: usize) -> Option<&DominatorTreeNode> {
        self.nodes.get(dfn)
    }
    pub fn dfn_node_mut(&mut self, dfn: usize) -> Option<&mut DominatorTreeNode> {
        self.nodes.get_mut(dfn)
    }
    pub fn dfn_get_block(&self, dfn: usize) -> Option<BlockRef> {
        self.dfn_get_node(dfn).map(|node| node.blockref)
    }
    pub fn dfn_get_idom(&self, dfn: usize) -> Option<BlockRef> {
        self.dfn_get_node(dfn).map(|node| node.idom_block)
    }
    pub fn dfn_get_idom_dfn(&self, dfn: usize) -> Option<usize> {
        self.dfn_get_node(dfn).map(|node| node.idom_dfn)
    }
    pub fn dfn_get_semidom_block(&self, dfn: usize) -> Option<BlockRef> {
        self.dfn_get_node(dfn).map(|node| node.semidom_block)
    }
    pub fn dfn_get_semidom_dfn(&self, dfn: usize) -> Option<usize> {
        self.dfn_get_node(dfn).map(|node| node.semidom_dfn)
    }
    pub fn dfn_get_dfsnode(&self, dfn: usize) -> Option<&CfgDfsNode> {
        self.dfs_seq.dfn_get_node(dfn)
    }

    pub fn block_get_dfn(&self, block: BlockRef) -> Option<usize> {
        self.dfs_seq.block_get_dfn(block)
    }
    pub fn block_get_node(&self, block: BlockRef) -> Option<&DominatorTreeNode> {
        self.nodes.get(self.block_get_dfn(block)?)
    }
    pub fn block_get_idom(&self, block: BlockRef) -> Option<BlockRef> {
        self.block_get_node(block).map(|node| node.idom_block)
    }
    pub fn block_get_semidom(&self, block: BlockRef) -> Option<BlockRef> {
        self.block_get_node(block).map(|node| node.semidom_block)
    }
    pub fn block_is_reachable(&self, block: BlockRef) -> bool {
        self.dfs_seq.block_is_reachable(block)
    }
    pub fn dump_dfs_seq(&self) -> CfgDfsSeq {
        CfgDfsSeq::clone(&self.dfs_seq)
    }

    fn dfn_borrow_domcache_mut(&self, dfn: usize) -> RefMut<BTreeSet<BlockRef>> {
        self.dfn_get_node(dfn).unwrap().dominator_cache.borrow_mut()
    }
    fn dfn_borrow_domcache(&self, dfn: usize) -> Ref<BTreeSet<BlockRef>> {
        self.dfn_get_node(dfn).unwrap().dominator_cache.borrow()
    }
    pub fn block_dominates_block(&self, domaintor: BlockRef, dominee: BlockRef) -> bool {
        if domaintor == dominee {
            return true;
        }
        if !self.block_is_reachable(domaintor) || !self.block_is_reachable(dominee) {
            return false;
        }
        let domaintor_dfn = self.block_get_dfn(domaintor).unwrap();
        let dominee_dfn = self.block_get_dfn(dominee).unwrap();
        if domaintor_dfn <= dominee_dfn {
            return false;
        }
        if self.dfn_borrow_domcache(dominee_dfn).contains(&domaintor) {
            return true;
        }
        let mut cur = dominee;
        while cur != self.root {
            if cur == domaintor {
                self.dfn_borrow_domcache_mut(dominee_dfn).insert(domaintor);
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
            return inst1.comes_before_node(inst2, &module.borrow_value_alloc().insts);
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

    pub fn write_to_graphviz(
        &self,
        number_map: &IRValueNumberMap,
        writer: &mut dyn std::io::Write,
    ) {
        writeln!(writer, "digraph dominator_tree {{").unwrap();
        writeln!(writer, "  rankdir=TB;").unwrap();
        writeln!(writer, "  node [shape=circle];").unwrap();

        for i in self.nodes.iter() {
            if i.blockref.is_vexit() {
                writeln!(writer, "  {} [label=\"%VEXIT\" shape=box];", i.dfn,).unwrap();
            } else {
                writeln!(
                    writer,
                    "  {} [label=\"{}\"];",
                    i.dfn,
                    number_map.block_get_number(i.blockref).unwrap()
                )
                .unwrap();
            }
        }

        for i in self.nodes.iter() {
            let idom_dfn = i.idom_dfn;
            if idom_dfn != usize::MAX {
                writeln!(writer, "  {} -> {};", idom_dfn, i.dfn).unwrap();
            }
        }
        writeln!(writer, "}}").unwrap();
    }
}

impl DominatorTree {
    /// `Remusys-IR` uses Semi-NCA algorithm to build dominator tree.
    /// Relavent notes can be found in the documentation of `Remusys-IR`.
    pub fn new_from_snapshot(snapshot: &CfgSnapshot) -> Self {
        let dfs_seq_pre = Rc::new(CfgDfsSeq::new_from_snapshot(snapshot, DfsOrder::Pre));
        let mut dominator_tree = Self::new_empty(dfs_seq_pre, false);
        dominator_tree._build_semidom_from_snapshot(snapshot, &CfgSnapshot::block_get_prev);
        dominator_tree._build_idom_semi_nca();
        dominator_tree
    }

    /// **WARNING**: NOT TESTED
    pub fn new_postdom_from_snapshot(snapshot: &CfgSnapshot) -> Self {
        let (dfs_seq, exits) = CfgDfsSeq::new_from_rcfg_snapshot(snapshot, DfsOrder::Pre);
        let root_only = [(dfs_seq.get_root_dfn(), BlockRef::new_vexit())];
        let mut dominator_tree = Self::new_empty(Rc::new(dfs_seq), true);
        dominator_tree._build_semidom_from_snapshot(snapshot, &|snapshot, block| {
            if exits.binary_search(&block).is_ok() {
                return Some(&root_only);
            }
            snapshot.block_get_next(block)
        });
        dominator_tree._build_idom_semi_nca();
        dominator_tree
    }

    fn new_empty(dfs_seq: Rc<CfgDfsSeq>, is_postdom: bool) -> Self {
        let mut nodes = Vec::with_capacity(dfs_seq.n_logical_nodes());
        for dfn in 0..dfs_seq.n_logical_nodes() {
            nodes.push(DominatorTreeNode {
                blockref: dfs_seq.dfn_get_block(dfn).unwrap(),
                dfn,
                semidom_block: BlockRef::new_null(),
                idom_block: BlockRef::new_null(),
                dominator_cache: RefCell::new(BTreeSet::new()),
                semidom_dfn: usize::MAX,
                idom_dfn: usize::MAX,
            });
        }

        let root = dfs_seq.get_root();
        Self { dfs_seq, nodes: nodes, root, is_postdom }
    }

    fn _build_semidom_from_snapshot<'a>(
        &mut self,
        snapshot: &'a CfgSnapshot,
        get_pred: &impl Fn(&'a CfgSnapshot, BlockRef) -> Option<&'a [(usize, BlockRef)]>,
    ) -> DSU {
        let mut dfn_dsu = DSU::new(self.get_nnodes());
        let mut best_candidate = (0..self.get_nnodes()).collect::<Box<_>>();
        let mut semidom = (0..self.get_nnodes()).collect::<Box<_>>();

        for u in (1..self.get_nnodes()).rev() {
            let mut res = usize::MAX;
            let u_block = self.dfn_get_block(u).unwrap();
            let prev = match get_pred(snapshot, u_block) {
                Some(prev) => prev,
                None => continue,
            };
            for (_, blockref) in prev {
                let v = match self.dfs_seq.block_get_dfn(*blockref) {
                    Some(v) => v,
                    None => continue,
                };
                dfn_dsu.find_when(v, |x: usize, old_parent_dfn, _| {
                    let old_parent_elect = best_candidate[old_parent_dfn];
                    let x_elect = best_candidate[x];
                    if semidom[old_parent_elect] < semidom[x_elect] {
                        best_candidate[x] = old_parent_elect;
                    }
                });

                res = if v < u { res.min(v) } else { res.min(semidom[best_candidate[v]]) };
            }
            semidom[u] = res;
            let parent = self.dfs_seq.dfn_get_parent_dfn(u).unwrap();
            dfn_dsu.set_direct_parent(u, parent);
        }

        for (node, &sdom) in semidom.iter().enumerate() {
            let semidom_bb = self.dfn_get_block(sdom).unwrap();
            let node = self.dfn_node_mut(node).unwrap();
            node.semidom_block = semidom_bb;
            node.semidom_dfn = sdom;
        }

        dfn_dsu
    }

    fn _build_idom_semi_nca(&mut self) {
        for w in 1..self.get_nnodes() {
            let w_semidom_dfn = self.dfn_get_semidom_dfn(w).unwrap();
            let w_parent_dfn = self.dfs_seq.dfn_get_parent_dfn(w).unwrap();

            let mut idom = w_parent_dfn;
            while idom != 0 && idom > w_semidom_dfn {
                idom = self.dfn_get_idom_dfn(idom).unwrap();
            }

            let idom_block = self.dfn_get_block(idom).unwrap();
            self.dfn_node_mut(w).map(|node| {
                node.idom_block = idom_block;
                node.idom_dfn = idom;
            });
        }
    }
}
