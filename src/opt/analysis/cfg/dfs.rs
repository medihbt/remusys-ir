use std::collections::BTreeMap;

use crate::{
    base::NullableValue,
    ir::{
        block::BlockRef,
        global::{GlobalData, GlobalRef},
        module::Module,
    },
    opt::util::DfsOrder,
};

use super::snapshot::CfgSnapshot;

#[derive(Debug, Clone)]
pub struct CfgDfsNode {
    pub block: BlockRef,
    pub parent: BlockRef,
    pub dfn: usize,
    pub parent_dfn: usize,
}

/// The DFS-generated sequence and tree of CFG.
#[derive(Debug, Clone)]
pub struct CfgDfsSeq {
    pub nodes: Vec<CfgDfsNode>,
    pub dfn: BTreeMap<BlockRef, usize>,
    pub order: DfsOrder,
}

impl CfgDfsSeq {
    pub fn get_nnodes(&self) -> usize {
        self.nodes.len()
    }
    pub fn get_root(&self) -> BlockRef {
        self.nodes[0].block
    }

    pub fn block_get_dfn(&self, block: BlockRef) -> Option<usize> {
        self.dfn.get(&block).copied()
    }
    pub fn block_is_reachable(&self, block: BlockRef) -> bool {
        self.dfn.contains_key(&block)
    }
    pub fn block_get_node(&self, block: BlockRef) -> Option<&CfgDfsNode> {
        self.nodes.get(self.block_get_dfn(block)?)
    }
    pub fn block_get_parent(&self, block: BlockRef) -> Option<BlockRef> {
        self.block_get_node(block).map(|node| node.parent)
    }
    pub fn block_is_root(&self, block: BlockRef) -> bool {
        self.block_get_parent(block)
            .map_or(false, |parent| parent.is_null())
    }

    pub fn dfn_get_node(&self, dfn: usize) -> Option<&CfgDfsNode> {
        self.nodes.get(dfn)
    }
    pub fn dfn_get_block(&self, dfn: usize) -> Option<BlockRef> {
        self.dfn_get_node(dfn).map(|node| node.block)
    }
    pub fn dfn_get_parent(&self, dfn: usize) -> Option<BlockRef> {
        self.dfn_get_node(dfn)
            .map(|node| node.parent.to_option())
            .flatten()
    }
    pub fn dfn_get_parent_dfn(&self, dfn: usize) -> Option<usize> {
        self.dfn_get_node(dfn).map(|node| node.parent_dfn)
    }
    pub fn dfn_is_root(&self, dfn: usize) -> bool {
        self.dfn_get_parent(dfn)
            .map_or(false, |parent| parent.is_null())
    }
}

impl CfgDfsSeq {
    pub fn new_empty(order: DfsOrder) -> Self {
        CfgDfsSeq {
            nodes: Vec::new(),
            dfn: BTreeMap::new(),
            order,
        }
    }

    pub fn new_from_func(
        module: &Module,
        func: GlobalRef,
        order: DfsOrder,
    ) -> Result<Self, String> {
        let mut cfg_dfs_seq = Self::new_empty(order);
        cfg_dfs_seq.build_from_func(module, func)?;
        Ok(cfg_dfs_seq)
    }

    pub fn build_from_func(&mut self, module: &Module, func: GlobalRef) -> Result<(), String> {
        let (blocks_view, entry) = {
            let func_data = module.get_global(func);
            let func_data = match &*func_data {
                GlobalData::Func(f) => f,
                _ => return Err("Expected function".into()),
            };
            match func_data.get_blocks() {
                Some(blocks) => (
                    unsafe { blocks.unsafe_load_readonly_view() },
                    func_data.get_entry(),
                ),
                None => return Err("Function has no blocks".into()),
            }
        };

        let mut blocks_seq = Vec::with_capacity(blocks_view.len());
        let mut dfn = BTreeMap::new();

        match self.order.get_first_step() {
            DfsOrder::Pre => {
                Self::build_pre_order(
                    module,
                    entry,
                    BlockRef::new_null(),
                    usize::MAX,
                    &mut blocks_seq,
                    &mut dfn,
                );
            }
            DfsOrder::Post => {
                Self::build_post_order(
                    module,
                    entry,
                    BlockRef::new_null(),
                    &mut blocks_seq,
                    &mut dfn,
                );
            }
            _ => unreachable!(),
        }
        if self.order.should_reverse() {
            Self::reverse_dfs_order(&mut blocks_seq, &mut dfn);
        }
        self.nodes = blocks_seq;
        self.dfn = dfn;
        Ok(())
    }

    fn build_pre_order(
        module: &Module,
        block: BlockRef,
        parent: BlockRef,
        parent_dfn: usize,
        node_seq: &mut Vec<CfgDfsNode>,
        dfn_map: &mut BTreeMap<BlockRef, usize>,
    ) {
        if dfn_map.contains_key(&block) {
            return;
        }
        let terminator = {
            let block_data = module.get_block(block);
            block_data
                .get_terminator_subref(module)
                .expect("Block should have a terminator")
        };
        let dfn = node_seq.len();
        dfn_map.insert(block, dfn);
        node_seq.push(CfgDfsNode {
            block,
            parent,
            dfn,
            parent_dfn,
        });

        for succ in terminator.collect_jump_blocks_from_module(module) {
            Self::build_pre_order(module, succ, block, dfn, node_seq, dfn_map);
        }
    }

    fn build_post_order(
        module: &Module,
        block: BlockRef,
        parent: BlockRef,
        node_seq: &mut Vec<CfgDfsNode>,
        dfn_map: &mut BTreeMap<BlockRef, usize>,
    ) -> Option<usize> {
        if dfn_map.contains_key(&block) {
            return None;
        }
        dfn_map.insert(block, usize::MAX);
        let terminator = {
            let block_data = module.get_block(block);
            block_data
                .get_terminator_subref(module)
                .expect("Block should have a terminator")
        };
        let mut succ_dfns = Vec::new();
        for succ in terminator.collect_jump_blocks_from_module(module) {
            let succ_dfn = match Self::build_post_order(module, succ, block, node_seq, dfn_map) {
                Some(dfn) => dfn,
                None => continue,
            };
            succ_dfns.push(succ_dfn);
        }
        let dfn = node_seq.len();
        dfn_map.insert(block, dfn);
        node_seq.push(CfgDfsNode {
            block,
            parent,
            dfn,
            parent_dfn: usize::MAX,
        });

        for succ_dfn in succ_dfns {
            node_seq[succ_dfn].parent_dfn = dfn;
        }
        Some(dfn)
    }

    fn reverse_dfs_order(blocks_seq: &mut Vec<CfgDfsNode>, dfn: &mut BTreeMap<BlockRef, usize>) {
        blocks_seq.reverse();
        let nblocks = blocks_seq.len();
        for i in 0..nblocks {
            let block = blocks_seq[i].block;
            dfn.insert(block, nblocks - i - 1);
        }
    }

    pub fn new_from_snapshot(snapshot: &CfgSnapshot, order: DfsOrder) -> Self {
        let mut nodes = Vec::with_capacity(snapshot.nodes.len());
        let mut dfn = BTreeMap::new();

        match order.get_first_step() {
            DfsOrder::Pre => Self::build_pre_order_from_snapshot(
                snapshot,
                snapshot.entry,
                BlockRef::new_null(),
                usize::MAX,
                &mut nodes,
                &mut dfn,
            ),
            DfsOrder::Post => {
                Self::build_post_order_from_snapshot(
                    snapshot,
                    snapshot.entry,
                    BlockRef::new_null(),
                    &mut nodes,
                    &mut dfn,
                );
            }
            _ => unreachable!(),
        }
        if order.should_reverse() {
            Self::reverse_dfs_order(&mut nodes, &mut dfn);
        }
        Self { nodes, dfn, order }
    }

    fn build_pre_order_from_snapshot(
        snapshot: &CfgSnapshot,
        block: BlockRef,
        parent: BlockRef,
        parent_dfn: usize,
        node_seq: &mut Vec<CfgDfsNode>,
        dfn_map: &mut BTreeMap<BlockRef, usize>,
    ) {
        if dfn_map.contains_key(&block) {
            return;
        }
        let dfn = node_seq.len();
        dfn_map.insert(block, dfn);
        node_seq.push(CfgDfsNode {
            block,
            parent,
            dfn,
            parent_dfn,
        });

        let succ = match snapshot.block_get_node(block) {
            Some(node) => &node.next_seq,
            None => return,
        };
        for (_, succ_block) in succ {
            Self::build_pre_order_from_snapshot(
                snapshot,
                succ_block.clone(),
                block,
                dfn,
                node_seq,
                dfn_map,
            );
        }
    }

    fn build_post_order_from_snapshot(
        snapshot: &CfgSnapshot,
        block: BlockRef,
        parent: BlockRef,
        node_seq: &mut Vec<CfgDfsNode>,
        dfn_map: &mut BTreeMap<BlockRef, usize>,
    ) -> Option<usize> {
        if dfn_map.contains_key(&block) {
            return None;
        }
        dfn_map.insert(block, usize::MAX);

        let mut succ_dfns = Vec::new();
        if let Some(node) = snapshot.block_get_node(block) {
            for (_, succ_block) in node.next_seq.iter() {
                let succ_dfn = Self::build_post_order_from_snapshot(
                    snapshot,
                    succ_block.clone(),
                    block,
                    node_seq,
                    dfn_map,
                );
                if let Some(succ_dfn) = succ_dfn {
                    succ_dfns.push(succ_dfn);
                }
            }
        }
        let dfn = node_seq.len();
        dfn_map.insert(block, dfn);
        node_seq.push(CfgDfsNode {
            block,
            parent,
            dfn,
            parent_dfn: usize::MAX,
        });
        for succ_dfn in succ_dfns {
            node_seq[succ_dfn].parent_dfn = dfn;
        }
        Some(dfn)
    }
}
