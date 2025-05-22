//! Dominance Tree Analysis

use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

use slab::Slab;

use crate::{
    base::{NullableValue, slablist::SlabRefListNodeRef},
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
    pub dfn: usize,
    pub semidom: BlockRef,
    pub idom: BlockRef,

    pub dominator_cache: RefCell<BTreeSet<BlockRef>>,
}

impl DominatorTreeNode {
    pub fn new(blockref: BlockRef, dfn: usize) -> Self {
        Self {
            blockref,
            dfn,
            semidom: BlockRef::new_null(),
            idom: BlockRef::new_null(),
            dominator_cache: RefCell::new(BTreeSet::new()),
        }
    }
}

pub struct DominatorTree {
    pub dfs_seq_pre: Rc<CfgDfsSeq>,
    pub dfs_seq_rpo: Rc<CfgDfsSeq>,
    pub nodes: Vec<DominatorTreeNode>,
    pub root: BlockRef,
}

impl DominatorTree {
    pub fn rpo_dfn_get_node(&self, dfn: usize) -> Option<&DominatorTreeNode> {
        self.nodes.get(dfn)
    }
    pub fn rpo_dfn_get_block(&self, dfn: usize) -> Option<BlockRef> {
        self.rpo_dfn_get_node(dfn).map(|node| node.blockref)
    }
    pub fn rpo_dfn_get_idom(&self, dfn: usize) -> Option<BlockRef> {
        self.rpo_dfn_get_node(dfn).map(|node| node.idom)
    }
    pub fn rpo_dfn_get_semidom(&self, dfn: usize) -> Option<BlockRef> {
        self.rpo_dfn_get_node(dfn).map(|node| node.semidom)
    }
    pub fn rpo_dfn_get_dfsnode(&self, dfn: usize) -> Option<&CfgDfsNode> {
        self.dfs_seq_rpo.dfn_get_node(dfn)
    }

    pub fn block_get_rpo_dfn(&self, block: BlockRef) -> Option<usize> {
        self.dfs_seq_rpo.block_get_dfn(block)
    }
    pub fn block_get_node(&self, block: BlockRef) -> Option<&DominatorTreeNode> {
        self.nodes.get(self.block_get_rpo_dfn(block)?)
    }
    pub fn block_get_idom(&self, block: BlockRef) -> Option<BlockRef> {
        self.block_get_node(block).map(|node| node.idom)
    }
    pub fn block_get_semidom(&self, block: BlockRef) -> Option<BlockRef> {
        self.block_get_node(block).map(|node| node.semidom)
    }
    pub fn block_is_reachable(&self, block: BlockRef) -> bool {
        self.dfs_seq_rpo.block_is_reachable(block)
    }
    pub fn dump_dfs_seq(&self) -> CfgDfsSeq {
        CfgDfsSeq::clone(&self.dfs_seq_rpo)
    }

    pub fn block_dominates_block(&self, domaintor: BlockRef, dominee: BlockRef) -> bool {
        if domaintor == dominee {
            return true;
        }
        if !self.block_is_reachable(domaintor) || !self.block_is_reachable(dominee) {
            return false;
        }
        let domaintor_dfn = self.block_get_rpo_dfn(domaintor).unwrap();
        let dominee_dfn = self.block_get_rpo_dfn(dominee).unwrap();
        if domaintor_dfn <= dominee_dfn {
            return false;
        }
        if self
            .rpo_dfn_get_node(dominee_dfn)
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
                self.rpo_dfn_get_node(dominee_dfn)
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
        let mut nodes = Vec::with_capacity(dfs_rpo.get_nnodes());
        for i in 0..dfs_rpo.get_nnodes() {
            let block = dfs_rpo.dfn_get_block(i).unwrap();
            nodes.push(DominatorTreeNode::new(block, i));
        }
        let root = dfs_rpo.get_root();
        Self {
            dfs_seq_pre: dfs_pre,
            dfs_seq_rpo: dfs_rpo,
            nodes,
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
        let mut dominator_tree = Self::new_empty(dfs_seq_pre.clone(), dfs_seq_rpo.clone());
        dominator_tree._build_semidom();
        dominator_tree._build_idom();
        dominator_tree
    }

    fn _build_semidom(&mut self) {}
    fn _build_idom(&mut self) {}
}
