//! A mininal read-only CFG snapshot at a specific point in time.
//!
//! Visiting a CFG inside a Module is very hard and expensive, while only
//! a small part of transform passes changes the CFG. Having a snapshot
//! makes it easier to visit the CFG.

use std::collections::{BTreeMap, BTreeSet};

use crate::ir::{
    block::BlockRef,
    global::{GlobalData, GlobalRef},
    module::Module,
};

/// ## CFG Snapshot
///
/// A CFG snapshot is a read-only view of the CFG at a specific point in time.
/// It is used to provide a consistent view of the CFG to the analysis passes.
///
/// This snapshot does not include a DFS sequence because there are too many
/// kinds of orders and I cannot find a good one.
///
/// This snapshot will include all the blocks in one function, whether they are
/// reachable or not.
pub struct CfgSnapshot {
    /// All nodes sorted by `BlockRef` handle number.
    pub nodes: Box<[CfgSnapshotNode]>,

    pub func: GlobalRef,
    pub entry: BlockRef,
}

pub struct CfgSnapshotNode {
    pub block: BlockRef,

    /// Sorted by `BlockRef` handle number. Since the `usize` member represents
    /// the index of the block in the `nodes` array, you can also say that the
    /// array is sorted by the index of the block in the `nodes` array.
    pub prev_set: Box<[(usize, BlockRef)]>,

    /// Sorted by `BlockRef` handle number. Since the `usize` member represents
    /// the index of the block in the `nodes` array, you can also say that the
    /// array is sorted by the index of the block in the `nodes` array.
    pub next_set: Box<[(usize, BlockRef)]>,

    /// Kept the original order of the `next` edges. This is used to decrease the
    /// differece between the original CFG and the snapshot.
    pub next_seq: Box<[(usize, BlockRef)]>,
}

impl CfgSnapshot {
    pub fn get_nnodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn block_get_node_pos(&self, block: BlockRef) -> Option<usize> {
        self.nodes.binary_search_by(|l| l.block.cmp(&block)).ok()
    }
    pub fn block_get_node(&self, block: BlockRef) -> Option<&CfgSnapshotNode> {
        let pos = self.block_get_node_pos(block)?;
        Some(&self.nodes[pos])
    }
    pub fn block_is_entry(&self, block: BlockRef) -> bool {
        self.entry == block
    }
    pub fn block_is_exit(&self, block: BlockRef) -> bool {
        match self.block_get_node(block) {
            Some(node) => node.next_set.is_empty(),
            None => false,
        }
    }
    pub fn block_get_prev(&self, block: BlockRef) -> Option<&[(usize, BlockRef)]> {
        self.block_get_node(block).map(|node| &*node.prev_set)
    }
    pub fn block_get_next(&self, block: BlockRef) -> Option<&[(usize, BlockRef)]> {
        self.block_get_node(block).map(|node| &*node.next_set)
    }
    pub fn block_has_next(&self, block: BlockRef, next: BlockRef) -> bool {
        let successors = match self.block_get_next(block) {
            Some(next) => next,
            None => return false,
        };
        successors.binary_search_by(|(_, b)| b.cmp(&next)).is_ok()
    }
    pub fn block_has_prev(&self, block: BlockRef, prev: BlockRef) -> bool {
        let predecessors = match self.block_get_prev(block) {
            Some(prev) => prev,
            None => return false,
        };
        predecessors.binary_search_by(|(_, b)| b.cmp(&prev)).is_ok()
    }
}

impl CfgSnapshotNode {
    pub fn new(block: BlockRef) -> Self {
        Self {
            block,
            prev_set: Box::new([]),
            next_set: Box::new([]),
            next_seq: Box::new([]),
        }
    }

    pub fn new_with_prev_next(
        block: BlockRef,
        prev_set: Box<[(usize, BlockRef)]>,
        next_set: Box<[(usize, BlockRef)]>,
        next_seq: Box<[(usize, BlockRef)]>,
    ) -> Self {
        Self {
            block,
            prev_set,
            next_set,
            next_seq,
        }
    }
}

impl CfgSnapshot {
    pub fn new_empty(func: GlobalRef, entry: BlockRef) -> Self {
        Self {
            nodes: Box::new([]),
            func,
            entry,
        }
    }

    pub fn new_from_func(module: &Module, func: GlobalRef) -> Self {
        let (blocks_view, entry) = {
            let func_data = module.get_global(func);
            if let GlobalData::Func(f) = &*func_data {
                match f.get_blocks() {
                    Some(blocks) => (unsafe { blocks.unsafe_load_readonly_view() }, f.get_entry()),
                    None => panic!("Function has no blocks"),
                }
            } else {
                panic!("Expected function");
            }
        };

        let alloc_value = module.borrow_value_alloc();
        let alloc_jt = module.borrow_jt_alloc();
        let alloc_block = &alloc_value.alloc_block;
        let alloc_inst = &alloc_value.alloc_inst;

        // Get successors and predecessors of each block.
        let mut succ_map: BTreeMap<BlockRef, Vec<BlockRef>> = BTreeMap::new();
        let mut pred_map: BTreeMap<BlockRef, BTreeSet<BlockRef>> = BTreeMap::new();

        for (blockref, block) in blocks_view.view(alloc_block) {
            let terminator = block
                .get_terminator_subref(module)
                .expect("Block has no terminator");
            let succ_vec = terminator.collect_jump_blocks(alloc_inst, &alloc_jt);
            for succ in &succ_vec {
                pred_map
                    .entry(*succ)
                    .or_insert_with(BTreeSet::new)
                    .insert(blockref);
            }
            succ_map.insert(blockref, succ_vec);
        }

        // Collect predecessors and successors of each block into nodes.
        // The collected vector is sorted by `BlockRef` handle number naturally.
        let mut nodes = Vec::with_capacity(succ_map.len());
        for (blockref, succ_vec) in succ_map {
            let mut prev_set = Vec::with_capacity(pred_map.len());
            let mut next_set = Vec::with_capacity(succ_vec.len());
            let mut next_seq = Vec::with_capacity(succ_vec.len());

            if let Some(pred_vec) = pred_map.get(&blockref) {
                for pred in pred_vec {
                    prev_set.push((0, *pred));
                }
            }

            for succ in &succ_vec {
                next_set.push((0, *succ));
                next_seq.push((0, *succ));
            }

            assert!(prev_set.is_sorted_by(|(_, a), (_, b)| a < b));
            next_set.sort_by(|(_, a), (_, b)| a.cmp(b));

            nodes.push(CfgSnapshotNode::new_with_prev_next(
                blockref,
                prev_set.into_boxed_slice(),
                next_set.into_boxed_slice(),
                next_seq.into_boxed_slice(),
            ));
        }
        assert!(nodes.is_sorted_by(|a, b| a.block < b.block));

        // Fill in the `prev_set` and `next_set` with the index of the block
        let node_index_map: Vec<BlockRef> = nodes.iter().map(|node| node.block).collect();
        for node in nodes.iter_mut() {
            for (n, prev) in node.prev_set.iter_mut() {
                *n = node_index_map
                    .binary_search_by(|b| b.cmp(prev))
                    .expect("Predecessor not found");
            }
            for (n, next) in node.next_set.iter_mut() {
                *n = node_index_map
                    .binary_search_by(|b| b.cmp(next))
                    .expect("Successor not found");
            }
            for (n, next) in node.next_seq.iter_mut() {
                *n = node_index_map
                    .binary_search_by(|b| b.cmp(next))
                    .expect("Successor not found");
            }
        }

        Self {
            nodes: nodes.into_boxed_slice(),
            func,
            entry,
        }
    }
}
