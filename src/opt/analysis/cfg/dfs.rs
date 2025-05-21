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

#[derive(Debug, Clone)]
pub struct CfgDfsNode {
    pub block: BlockRef,
    pub parent: BlockRef,
    pub dfn: usize,
}

/// The DFS-generated sequence and tree of CFG.
pub struct CfgDfsSeq {
    pub nodes: Vec<CfgDfsNode>,
    pub dfn: BTreeMap<BlockRef, usize>,
    pub order: DfsOrder,
}

impl CfgDfsSeq {
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
        self.dfn_get_node(dfn).map(|node| node.parent)
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
        node_seq.push(CfgDfsNode { block, parent, dfn });

        for succ in terminator.collect_jump_blocks_from_module(module) {
            Self::build_pre_order(module, succ, block, node_seq, dfn_map);
        }
    }

    fn build_post_order(
        module: &Module,
        block: BlockRef,
        parent: BlockRef,
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
        for succ in terminator.collect_jump_blocks_from_module(module) {
            Self::build_post_order(module, succ, block, node_seq, dfn_map);
        }
        let dfn = node_seq.len();
        dfn_map.insert(block, dfn);
        node_seq.push(CfgDfsNode { block, parent, dfn });
    }

    fn reverse_dfs_order(blocks_seq: &mut Vec<CfgDfsNode>, dfn: &mut BTreeMap<BlockRef, usize>) {
        blocks_seq.reverse();
        let nblocks = blocks_seq.len();
        for i in 0..nblocks {
            let block = blocks_seq[i].block;
            dfn.insert(block, nblocks - i - 1);
        }
    }
}
