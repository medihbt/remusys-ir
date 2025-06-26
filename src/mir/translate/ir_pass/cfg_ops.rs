use std::collections::BTreeSet;

use crate::{
    base::NullableValue,
    ir::{
        block::{BlockRef, jump_target::JumpTargetRef},
        global::GlobalData,
        module::Module,
    },
};

#[derive(Debug, Clone)]
pub(super) enum CfgOP {
    SplitBlock { bb: u32, new_bb: u32 },
    RedirectEdge { edge: u32, to_bb: u32 },
    InsertBlock { new_bb: u32, after: u32 },
    RemoveEdge { edge: u32 },
}

pub(super) struct CfgOpQueue {
    pub ops: Vec<CfgOP>,
    pub edges: Vec<JumpTargetRef>,
    pub blocks: Vec<BlockRef>,
}

impl CfgOpQueue {
    fn new() -> Self {
        Self {
            ops: Vec::new(),
            edges: Vec::new(),
            blocks: Vec::new(),
        }
    }

    fn add_split_block(&mut self, bb: BlockRef) -> u32 {
        let old_bb_index = self.find_insert_bb(bb);
        let new_bb = self.add_unnamed_block();
        self.ops.push(CfgOP::SplitBlock {
            bb: old_bb_index,
            new_bb,
        });
        new_bb
    }
    fn add_split_block_by_pos(&mut self, bb_index: u32) -> u32 {
        let new_bb = self.add_unnamed_block();
        self.ops.push(CfgOP::SplitBlock {
            bb: bb_index,
            new_bb,
        });
        new_bb
    }

    fn add_redirect_edge(&mut self, edge: JumpTargetRef, to_bb: BlockRef) -> u32 {
        let edge_index = self.find_insert_edge(edge);
        let to_bb_index = self.find_insert_bb(to_bb);
        self.ops.push(CfgOP::RedirectEdge {
            edge: edge_index,
            to_bb: to_bb_index,
        });
        edge_index
    }
    fn add_redirect_edge_by_pos(&mut self, edge_index: u32, to_bb: BlockRef) -> u32 {
        let edge = self.edges[edge_index as usize];
        let to_bb_index = self.find_insert_bb(to_bb);
        self.ops.push(CfgOP::RedirectEdge {
            edge: edge_index,
            to_bb: to_bb_index,
        });
        edge_index
    }

    fn add_insert_block(&mut self, after: BlockRef) -> u32 {
        let new_bb = self.find_insert_bb(after);
        self.ops.push(CfgOP::InsertBlock {
            new_bb,
            after: new_bb,
        });
        new_bb
    }
    fn add_insert_block_by_pos(&mut self, after_index: u32) -> u32 {
        let after = self.blocks[after_index as usize];
        let new_bb = self.add_unnamed_block();
        self.ops.push(CfgOP::InsertBlock {
            new_bb,
            after: after_index,
        });
        new_bb
    }

    fn add_remove_edge(&mut self, edge: JumpTargetRef) -> u32 {
        let edge_index = self.find_insert_edge(edge);
        self.ops.push(CfgOP::RemoveEdge { edge: edge_index });
        edge_index
    }

    fn add_remove_edge_by_pos(&mut self, edge_index: u32) -> u32 {
        let edge = self.edges[edge_index as usize];
        self.ops.push(CfgOP::RemoveEdge { edge: edge_index });
        edge_index
    }

    fn find_insert_bb(&mut self, after: BlockRef) -> u32 {
        if let Some(index) = self.blocks.iter().position(|b| *b == after) {
            return index as u32;
        }
        let new_bb = self.blocks.len() as u32;
        self.blocks.push(after);
        new_bb
    }
    fn add_unnamed_block(&mut self) -> u32 {
        let new_bb = self.blocks.len() as u32;
        self.blocks.push(BlockRef::new_null());
        new_bb
    }

    fn find_insert_edge(&mut self, edge: JumpTargetRef) -> u32 {
        if let Some(index) = self.edges.iter().position(|e| *e == edge) {
            return index as u32;
        }
        let new_edge = self.edges.len() as u32;
        self.edges.push(edge);
        new_edge
    }
    fn add_unnamed_edge(&mut self) -> u32 {
        let new_edge = self.edges.len() as u32;
        self.edges.push(JumpTargetRef::new_null());
        new_edge
    }

    pub fn new_break_key_edges(ir_module: &Module) -> Self {
        let mut ret = Self::new();
        let mut key_edges: Vec<(BlockRef, BlockRef, JumpTargetRef)> =
            Vec::with_capacity(ir_module.borrow_jt_alloc().len());

        let all_live_bbs = {
            let global_defs = ir_module.global_defs.borrow();
            let mut live_bbs = Vec::with_capacity(global_defs.len());

            for (_, gdef) in global_defs.iter() {
                if let GlobalData::Func(f) = &*ir_module.get_global(*gdef) {
                    let (bbs, nbbs) = if let Some(bbs) = f.get_blocks() {
                        (bbs.load_range(), bbs.len())
                    } else {
                        continue;
                    };
                    if nbbs != 0 {
                        live_bbs.push((bbs, nbbs));
                    }
                }
            }

            live_bbs
        };

        let alloc_value = ir_module.borrow_value_alloc();
        let alloc_block = &alloc_value.alloc_block;
        let mut block_with_multi_preds = BTreeSet::new();

        for (bbs, nbbs) in all_live_bbs {
            let mut all_live_insts = Vec::with_capacity(nbbs);
            for (bb_id, bb) in bbs.view(alloc_block) {
                if ir_module
                    .borrow_rcfg_alloc()
                    .unwrap()
                    .get_node(bb_id)
                    .preds
                    .borrow()
                    .len()
                    > 1
                {
                    block_with_multi_preds.insert(bb);
                }
            }
        }

        ret
    }
}
