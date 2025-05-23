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
    pub fn n_logical_nodes(&self) -> usize {
        self.nodes.len()
    }
    pub fn n_real_nodes(&self) -> usize {
        if self.root_is_virtual() {
            self.nodes.len() - 1
        } else {
            self.nodes.len()
        }
    }
    pub fn get_root_dfn(&self) -> usize {
        match self.order {
            DfsOrder::Pre | DfsOrder::ReversePost => 0,
            DfsOrder::Post | DfsOrder::ReversePre => self.nodes.len() - 1,
        }
    }
    pub fn get_root(&self) -> BlockRef {
        match self.order {
            DfsOrder::Pre | DfsOrder::ReversePost => self.nodes[0].block,
            DfsOrder::Post | DfsOrder::ReversePre => self.nodes.last().unwrap().block,
        }
    }
    /// Since a CFG in a function may contain multiple exit blocks,
    /// we always add a virtual root block to the CFG.
    pub fn root_is_virtual(&self) -> bool {
        self.get_root().is_null()
    }
    pub fn is_from_reverse_cfg(&self) -> bool {
        self.root_is_virtual()
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
                &CfgSnapshot::block_get_next,
            ),
            DfsOrder::Post => {
                Self::build_post_order_from_snapshot(
                    snapshot,
                    snapshot.entry,
                    BlockRef::new_null(),
                    &mut nodes,
                    &mut dfn,
                    &CfgSnapshot::block_get_next,
                );
            }
            _ => unreachable!(),
        }
        if order.should_reverse() {
            Self::reverse_dfs_order(&mut nodes, &mut dfn);
        }
        Self { nodes, dfn, order }
    }

    fn build_pre_order_from_snapshot<'a>(
        snapshot: &'a CfgSnapshot,
        block: BlockRef,
        parent: BlockRef,
        parent_dfn: usize,
        node_seq: &mut Vec<CfgDfsNode>,
        dfn_map: &mut BTreeMap<BlockRef, usize>,
        get_succ: &impl Fn(&'a CfgSnapshot, BlockRef) -> Option<&'a [(usize, BlockRef)]>,
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

        let succ = match get_succ(snapshot, block) {
            Some(node) => node,
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
                get_succ,
            );
        }
    }

    fn build_post_order_from_snapshot<'a>(
        snapshot: &'a CfgSnapshot,
        block: BlockRef,
        parent: BlockRef,
        node_seq: &mut Vec<CfgDfsNode>,
        dfn_map: &mut BTreeMap<BlockRef, usize>,
        get_succ: &impl Fn(&'a CfgSnapshot, BlockRef) -> Option<&'a [(usize, BlockRef)]>,
    ) -> Option<usize> {
        if dfn_map.contains_key(&block) {
            return None;
        }
        dfn_map.insert(block, usize::MAX);

        let mut succ_dfns = Vec::new();
        if let Some(succ) = get_succ(snapshot, block) {
            for (_, succ_block) in succ {
                let succ_dfn = Self::build_post_order_from_snapshot(
                    snapshot,
                    succ_block.clone(),
                    block,
                    node_seq,
                    dfn_map,
                    get_succ,
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

impl CfgDfsSeq {
    pub fn new_from_rcfg_snapshot(
        snapshot: &CfgSnapshot,
        order: DfsOrder,
    ) -> (Self, Box<[BlockRef]>) {
        // +1 for the virtual root.
        let mut nodes = Vec::with_capacity(snapshot.nodes.len() + 1);
        let mut dfn_map = BTreeMap::new();

        let real_exits = Self::dump_real_exits(snapshot);
        assert!(real_exits.is_sorted());

        match order.get_first_step() {
            DfsOrder::Pre => {
                // push a virtual root block.
                nodes.push(CfgDfsNode {
                    block: BlockRef::new_vexit(),
                    parent: BlockRef::new_null(),
                    dfn: 0,
                    parent_dfn: usize::MAX,
                });
                dfn_map.insert(BlockRef::new_vexit(), 0);
                for &block in &real_exits {
                    Self::build_pre_order_from_snapshot(
                        snapshot,
                        block,
                        BlockRef::new_vexit(),
                        0,
                        &mut nodes,
                        &mut dfn_map,
                        &CfgSnapshot::block_get_prev,
                    );
                }
            }
            DfsOrder::Post => {
                let mut succ_dfns = Vec::new();
                dfn_map.insert(BlockRef::new_vexit(), usize::MAX);
                for &block in &real_exits {
                    let dfn = Self::build_post_order_from_snapshot(
                        snapshot,
                        block,
                        BlockRef::new_vexit(),
                        &mut nodes,
                        &mut dfn_map,
                        &CfgSnapshot::block_get_prev,
                    );
                    if let Some(dfn) = dfn {
                        succ_dfns.push(dfn);
                    }
                }
                // push a virtual root block.
                let root_dfn = nodes.len();
                nodes.push(CfgDfsNode {
                    block: BlockRef::new_vexit(),
                    parent: BlockRef::new_null(),
                    dfn: root_dfn,
                    parent_dfn: usize::MAX,
                });
                dfn_map.insert(BlockRef::new_vexit(), root_dfn);
                for succ_dfn in succ_dfns {
                    nodes[succ_dfn].parent_dfn = root_dfn;
                }
            }
            _ => unreachable!(),
        }

        if order.should_reverse() {
            Self::reverse_dfs_order(&mut nodes, &mut dfn_map);
        }
        (
            Self {
                nodes,
                dfn: dfn_map,
                order,
            },
            real_exits,
        )
    }

    /// NOTE: Since the CFG snapshot nodes are already sorted by `BlockRef` order,
    /// the return value is also sorted.
    fn dump_real_exits(snapshot: &CfgSnapshot) -> Box<[BlockRef]> {
        snapshot
            .nodes
            .iter()
            .filter(|node| node.next_seq.is_empty())
            .map(|node| node.block)
            .collect()
    }
}
