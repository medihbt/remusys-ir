use std::collections::BTreeMap;

use crate::{
    ir::{
        block::BlockRef,
        global::{GlobalData, GlobalRef},
        module::Module,
    },
    opt::util::DfsOrder,
};

pub struct CfgDfsSeq {
    pub blocks: Vec<BlockRef>,
    pub dfn: BTreeMap<BlockRef, usize>,
    pub order: DfsOrder,
}

impl CfgDfsSeq {
    pub fn new_empty(order: DfsOrder) -> Self {
        CfgDfsSeq {
            blocks: Vec::new(),
            dfn: BTreeMap::new(),
            order,
        }
    }

    pub fn new_from_func(module: &Module, func: GlobalRef, order: DfsOrder) -> Result<Self, String> {
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
                Self::build_pre_order(module, entry, &mut blocks_seq, &mut dfn);
            }
            DfsOrder::Post => {
                Self::build_post_order(module, entry, &mut blocks_seq, &mut dfn);
            }
            _ => unreachable!(),
        }
        if self.order.should_reverse() {
            Self::reverse_dfs_order(&mut blocks_seq, &mut dfn);
        }
        self.blocks = blocks_seq;
        self.dfn = dfn;
        Ok(())
    }

    fn build_pre_order(
        module: &Module,
        block: BlockRef,
        block_seq: &mut Vec<BlockRef>,
        dfn: &mut BTreeMap<BlockRef, usize>,
    ) {
        if dfn.contains_key(&block) {
            return;
        }
        let terminator = {
            let block_data = module.get_block(block);
            block_data
                .get_terminator_subref(module)
                .expect("Block should have a terminator")
        };
        dfn.insert(block, block_seq.len());
        block_seq.push(block);

        for succ in terminator.collect_jump_blocks_from_module(module) {
            if dfn.contains_key(&succ) {
                continue;
            }
            Self::build_pre_order(module, succ, block_seq, dfn);
        }
    }

    fn build_post_order(
        module: &Module,
        block: BlockRef,
        block_seq: &mut Vec<BlockRef>,
        dfn: &mut BTreeMap<BlockRef, usize>,
    ) {
        if dfn.contains_key(&block) {
            return;
        }
        let terminator = {
            let block_data = module.get_block(block);
            block_data
                .get_terminator_subref(module)
                .expect("Block should have a terminator")
        };

        for succ in terminator.collect_jump_blocks_from_module(module) {
            if dfn.contains_key(&succ) {
                continue;
            }
            Self::build_post_order(module, succ, block_seq, dfn);
        }

        dfn.insert(block, block_seq.len());
        block_seq.push(block);
    }

    fn reverse_dfs_order(blocks_seq: &mut Vec<BlockRef>, dfn: &mut BTreeMap<BlockRef, usize>) {
        blocks_seq.reverse();
        let nblocks = blocks_seq.len();
        for i in 0..nblocks {
            let block = blocks_seq[i];
            dfn.insert(block, nblocks - i - 1);
        }
    }
}
