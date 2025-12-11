//! DFS over a Remusys IR function's control flow graph (CFG).
//!
//! 提供多种 DFS 遍历顺序（前序/后序/反向/基于出口的变体等），并记录遍历序列及节点间的父子关系。

use crate::{
    ir::{BlockID, FuncBody, FuncID, IRAllocs, ISubGlobalID, ISubInstID, JumpTargetsBlockIter},
    opt::{CfgErr, CfgRes, analysis::cfg::CfgBlockStat},
};
use mtb_entity_slab::EntityListIter;
use smallvec::SmallVec;
use std::{collections::HashMap, fmt::Debug};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DfsOrder {
    Pre,
    Post,
    RevPre,
    RevPost,
    BackPre,
    BackPost,
    BackRevPre,
    BackRevPost,
}

impl DfsOrder {
    pub fn is_rev(self) -> bool {
        matches!(
            self,
            DfsOrder::RevPre | DfsOrder::RevPost | DfsOrder::BackRevPre | DfsOrder::BackRevPost
        )
    }
    pub fn is_back(self) -> bool {
        matches!(
            self,
            DfsOrder::BackPre | DfsOrder::BackPost | DfsOrder::BackRevPre | DfsOrder::BackRevPost
        )
    }
    pub fn is_post(self) -> bool {
        matches!(
            self,
            DfsOrder::Post | DfsOrder::RevPost | DfsOrder::BackPost | DfsOrder::BackRevPost
        )
    }

    pub fn into_norev(self) -> Self {
        use DfsOrder::*;
        match self {
            Pre | RevPre => Pre,
            Post | RevPost => Post,
            BackPre | BackRevPre => BackPre,
            BackPost | BackRevPost => BackPost,
        }
    }

    pub fn reverse(self) -> Self {
        use DfsOrder::*;
        match self {
            Pre => RevPre,
            Post => RevPost,
            RevPre => Pre,
            RevPost => Post,
            BackPre => BackRevPre,
            BackPost => BackRevPost,
            BackRevPre => BackPre,
            BackRevPost => BackPost,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CfgDfsNode {
    pub block: CfgBlockStat,
    pub dfs_index: usize,
    pub parent: usize,
    pub children: SmallVec<[usize; 4]>,
}

#[derive(Debug, Clone)]
pub struct CfgDfsSeq {
    pub order: DfsOrder,
    pub nodes: Box<[CfgDfsNode]>,
    pub unseq: HashMap<BlockID, usize>,
    pub virt_index: Option<usize>,
}

impl CfgDfsSeq {
    pub const NULL_PARENT: usize = usize::MAX;

    /// 构造并返回指定函数的 DFS 序列。
    ///
    /// - `allocs`: IR 分配器上下文，用于读取函数/基本块结构。
    /// - `func`: 要遍历的函数 `FuncID`。
    /// - `order`: 指定遍历顺序（前序/后序/反向/基于出口的变体等）。
    ///
    /// 返回构造好的 `DfsSeq` 或在函数为 extern / 无法退出等情形下返回错误。
    pub fn new(allocs: &IRAllocs, func: FuncID, order: DfsOrder) -> CfgRes<Self> {
        let mut seq = match order.into_norev() {
            DfsOrder::Pre => Self::new_pre(allocs, func)?,
            DfsOrder::Post => Self::new_post(allocs, func)?,
            DfsOrder::BackPre => Self::new_back_pre(allocs, func)?,
            DfsOrder::BackPost => Self::new_back_post(allocs, func)?,
            _ => unreachable!(),
        };
        if order.is_rev() {
            seq.reverse();
        }
        Ok(seq)
    }

    pub fn reverse(&mut self) {
        self.order = self.order.reverse();
        self.nodes.reverse();
        for (i, node) in self.nodes.iter_mut().enumerate() {
            node.dfs_index = i;
        }
        for (_, v) in self.unseq.iter_mut() {
            *v = self.nodes.len() - 1 - *v;
        }
        if let Some(virt_idx) = &mut self.virt_index {
            *virt_idx = self.nodes.len() - 1 - *virt_idx;
        }
    }
    /// 构造函数的前序 (pre-order) DFS 序列，仅包含从入口可达的基本块。
    ///
    /// 返回的序列记录每个节点的父子关系与在 `nodes` 中的索引映射 `unseq`。
    pub fn new_pre(allocs: &IRAllocs, func: FuncID) -> CfgRes<Self> {
        DfsForwardBuilder::new(allocs, func).build(false)
    }
    /// 构造函数的后序 (post-order) DFS 序列，仅包含从入口可达的基本块。
    ///
    /// 用于需要后序遍历（例如某些数据流分析）的场景。
    pub fn new_post(allocs: &IRAllocs, func: FuncID) -> CfgRes<Self> {
        DfsForwardBuilder::new(allocs, func).build(true)
    }
    /// 基于函数出口（backward）构造的前序序列。
    ///
    /// 以函数的出口块作为起点，沿反边遍历（适用于基于出口的分析）。
    /// 如果函数没有出口（例如所有终结器都不是 `ret`/`unreachable`），会返回 `FuncCannotExit` 错误。
    pub fn new_back_pre(allocs: &IRAllocs, func: FuncID) -> CfgRes<Self> {
        DfsBackwardBuilder::new(allocs, func).build(false)
    }
    /// 基于函数出口（backward）构造的后序序列。
    ///
    /// 如果函数没有出口（例如所有终结器都不是 `ret`/`unreachable`），会返回 `FuncCannotExit` 错误。
    pub fn new_back_post(allocs: &IRAllocs, func: FuncID) -> CfgRes<Self> {
        DfsBackwardBuilder::new(allocs, func).build(true)
    }
    fn func_blocks(allocs: &IRAllocs, func: FuncID) -> CfgRes<EntityListIter<'_, BlockID>> {
        let func_obj = func.deref_ir(allocs);
        match &func_obj.body {
            Some(body) => Ok(body.blocks.iter(&allocs.blocks)),
            None => Err(CfgErr::FuncIsExtern(func)),
        }
    }

    pub fn try_block_dfn(&self, block: BlockID) -> Option<usize> {
        self.unseq.get(&block).copied()
    }
    pub fn block_dfn(&self, block: BlockID) -> usize {
        self.try_block_dfn(block)
            .expect("BlockID not found in DfsSeq")
    }
    pub fn block_reachable(&self, block: BlockID) -> bool {
        self.unseq.contains_key(&block)
    }

    pub fn try_dfn_block(&self, dfn: usize) -> Option<CfgBlockStat> {
        self.nodes.get(dfn).map(|n| n.block)
    }
    pub fn dfn_block(&self, dfn: usize) -> CfgBlockStat {
        self.try_dfn_block(dfn)
            .expect("DFN index out of bounds in DfsSeq")
    }
    pub fn dfn_valid(&self, dfn: usize) -> bool {
        dfn < self.nodes.len()
    }

    pub fn backward_get_exit_dfns(&self) -> Option<&[usize]> {
        let virt_idx = self.virt_index?;
        Some(self.nodes[virt_idx].children.as_slice())
    }
}

struct DfsBuildCommon<'ir> {
    nodes: Vec<CfgDfsNode>,
    unseq: HashMap<BlockID, usize>,
    ir_allocs: &'ir IRAllocs,
    func_id: FuncID,
    virt_index: Option<usize>,
}
impl<'ir> DfsBuildCommon<'ir> {
    fn take(&mut self) -> Self {
        use std::mem::take;
        DfsBuildCommon {
            nodes: take(&mut self.nodes),
            unseq: take(&mut self.unseq),
            ir_allocs: self.ir_allocs,
            func_id: self.func_id,
            virt_index: self.virt_index,
        }
    }

    fn new(ir_allocs: &'ir IRAllocs, func_id: FuncID) -> Self {
        DfsBuildCommon {
            nodes: Vec::new(),
            unseq: HashMap::new(),
            ir_allocs,
            func_id,
            virt_index: None,
        }
    }
}

type DfsFrame = SmallVec<[BlockID; 16]>;
type PreDfsStack = SmallVec<[(BlockID, usize); 16]>;

struct PostDfsFrame {
    block: BlockID,
    succ_frame: DfsFrame,
    children_index: SmallVec<[usize; 4]>,
    process_count: usize,
}
type PostDfsStack = SmallVec<[PostDfsFrame; 16]>;

const NONEXIST_ID: usize = usize::MAX;

trait DfsBuild<'ir> {
    fn new(ir_allocs: &'ir IRAllocs, func_id: FuncID) -> Self;

    fn get_common(&self) -> &DfsBuildCommon<'ir>;
    fn common_mut(&mut self) -> &mut DfsBuildCommon<'ir>;
    fn order(&self, is_post: bool) -> DfsOrder;

    fn build_fill(&mut self, is_post: bool) -> CfgRes;

    fn get_succs(&mut self, block: BlockID, frame: &mut DfsFrame);

    fn next_id(&self) -> usize {
        self.get_common().nodes.len()
    }
    fn is_visited(&self, bb: BlockID) -> bool {
        let Some(id) = self.get_common().unseq.get(&bb) else {
            return false;
        };
        *id != NONEXIST_ID
    }
    fn is_visiting_or_visited(&self, bb: BlockID) -> bool {
        self.get_common().unseq.contains_key(&bb)
    }
    fn mark_visiting(&mut self, bb: BlockID) {
        self.common_mut().unseq.insert(bb, NONEXIST_ID);
    }

    fn func_body(allocs: &IRAllocs, func: FuncID) -> CfgRes<&FuncBody> {
        let func_obj = func.deref_ir(allocs);
        match &func_obj.body {
            Some(body) => Ok(body),
            None => Err(CfgErr::FuncIsExtern(func)),
        }
    }

    fn build(&mut self, is_post: bool) -> CfgRes<CfgDfsSeq> {
        self.build_fill(is_post)?;
        let common = self.common_mut().take();
        Ok(CfgDfsSeq {
            order: self.order(is_post),
            nodes: common.nodes.into_boxed_slice(),
            unseq: common.unseq,
            virt_index: common.virt_index,
        })
    }

    fn pre_dfs_visit(&mut self, bb: BlockID, parent_index: usize) {
        let mut stack = PreDfsStack::new();
        let mut frame = DfsFrame::new();
        stack.push((bb, parent_index));
        while let Some((bb, parent_index)) = stack.pop() {
            if self.is_visited(bb) {
                continue;
            }
            let curr_idx = self.next_id();
            self.common_mut().unseq.insert(bb, curr_idx);

            // temporarily collect children
            self.get_succs(bb, &mut frame);

            let node = CfgDfsNode {
                block: CfgBlockStat::from(bb),
                dfs_index: curr_idx,
                parent: parent_index,
                children: SmallVec::with_capacity(frame.len()),
            };
            self.common_mut().nodes.push(node);
            if parent_index != CfgDfsSeq::NULL_PARENT {
                self.common_mut().nodes[parent_index]
                    .children
                    .push(curr_idx);
            }
            stack.reserve(frame.len());
            while let Some(child_bb) = frame.pop() {
                stack.push((child_bb, curr_idx));
            }
        }
    }

    fn get_post_frame(&mut self, block: BlockID) -> PostDfsFrame {
        let mut frame = PostDfsFrame {
            block,
            succ_frame: SmallVec::new(),
            children_index: SmallVec::new(),
            process_count: 0,
        };
        self.get_succs(block, &mut frame.succ_frame);
        frame
    }
    fn insert_block(&mut self, block: BlockID, dfs_index: usize) {
        let common = self.common_mut();
        common.unseq.insert(block, dfs_index);
        common.nodes.push(CfgDfsNode {
            block: CfgBlockStat::from(block),
            dfs_index,
            parent: CfgDfsSeq::NULL_PARENT,
            children: SmallVec::new(),
        });
    }
    fn post_dfs_visit(&mut self, bb: BlockID) -> usize {
        let mut stack = PostDfsStack::new();
        stack.push(self.get_post_frame(bb));

        while !stack.is_empty() {
            let PostDfsFrame { block, succ_frame, process_count, .. } = stack.last_mut().unwrap();
            let block = *block;

            assert!(
                !self.is_visited(block),
                "Internal error: visited {block:?} twice in post DFS which should be blocked in branch #0"
            );
            if *process_count < succ_frame.len() {
                let succ_bb = succ_frame[*process_count];
                *process_count += 1;

                if !self.is_visiting_or_visited(succ_bb) {
                    self.mark_visiting(succ_bb);
                    stack.push(self.get_post_frame(succ_bb));
                }
            } else {
                // allocate node index and append node
                let frame = stack.pop().unwrap();
                let dfs_index = self.next_id();
                self.insert_block(block, dfs_index);
                // wire children -> parent and set node.children
                let children_index = frame.children_index;
                let common = self.common_mut();
                for &child_idx in children_index.iter() {
                    assert_eq!(
                        common.nodes[child_idx].parent,
                        CfgDfsSeq::NULL_PARENT,
                        "Internal error: child node already has a parent in post DFS"
                    );
                    common.nodes[child_idx].parent = dfs_index;
                }
                common.nodes[dfs_index].children = children_index;

                // propagate this node as child to parent frame if any
                if let Some(parent_frame) = stack.last_mut() {
                    parent_frame.children_index.push(dfs_index);
                } else {
                    return dfs_index;
                }
            }
        }
        unreachable!("Internal error: post_dfs_visit should always return from inside the loop");
    }
}

struct DfsForwardBuilder<'ir>(DfsBuildCommon<'ir>);

impl<'ir> DfsBuild<'ir> for DfsForwardBuilder<'ir> {
    fn new(ir_allocs: &'ir IRAllocs, func_id: FuncID) -> Self {
        Self(DfsBuildCommon::new(ir_allocs, func_id))
    }

    fn get_common(&self) -> &DfsBuildCommon<'ir> {
        &self.0
    }
    fn common_mut(&mut self) -> &mut DfsBuildCommon<'ir> {
        &mut self.0
    }

    fn order(&self, is_post: bool) -> DfsOrder {
        if is_post { DfsOrder::Post } else { DfsOrder::Pre }
    }

    fn build_fill(&mut self, is_post: bool) -> CfgRes {
        let body = Self::func_body(self.0.ir_allocs, self.0.func_id)?;
        let entry_bb = body.entry;
        if is_post {
            self.post_dfs_visit(entry_bb);
        } else {
            self.pre_dfs_visit(entry_bb, CfgDfsSeq::NULL_PARENT);
        }
        Ok(())
    }

    fn get_succs(&mut self, block: BlockID, frame: &mut DfsFrame) {
        let allocs = self.0.ir_allocs;
        let bb_terminator = block.get_terminator(allocs);
        let succs = bb_terminator.get_jts(allocs);
        frame.reserve(succs.len());
        for succ_bb in JumpTargetsBlockIter::new(succs, allocs) {
            match succ_bb {
                Some(bid) => frame.push(bid),
                None => panic!(
                    "IR module sanity violated: discovered a terminator with null jump target"
                ),
            }
        }
    }
}

struct DfsBackwardBuilder<'ir>(DfsBuildCommon<'ir>);

impl<'ir> DfsBuild<'ir> for DfsBackwardBuilder<'ir> {
    fn new(ir_allocs: &'ir IRAllocs, func_id: FuncID) -> Self {
        Self(DfsBuildCommon::new(ir_allocs, func_id))
    }

    fn get_common(&self) -> &DfsBuildCommon<'ir> {
        &self.0
    }
    fn common_mut(&mut self) -> &mut DfsBuildCommon<'ir> {
        &mut self.0
    }
    fn order(&self, is_post: bool) -> DfsOrder {
        if is_post { DfsOrder::BackPost } else { DfsOrder::BackPre }
    }

    fn build_fill(&mut self, is_post: bool) -> CfgRes {
        let exits = self.dump_exits()?;
        if exits.is_empty() {
            return Err(CfgErr::FuncCannotExit(self.0.func_id));
        }
        let root_index = if is_post {
            let mut exit_indices: SmallVec<[usize; 4]> = SmallVec::new();
            for &exit_bb in &exits {
                exit_indices.push(self.post_dfs_visit(exit_bb));
            }
            let root_index = self.next_id();
            for &index in &exit_indices {
                self.0.nodes[index].parent = root_index;
            }
            self.0.nodes.push(CfgDfsNode {
                block: CfgBlockStat::Virtual,
                dfs_index: root_index,
                parent: CfgDfsSeq::NULL_PARENT,
                children: exit_indices,
            });
            root_index
        } else {
            // create virtual root and attach exits as its children
            let root_index = self.next_id();
            self.0.nodes.push(CfgDfsNode {
                block: CfgBlockStat::Virtual,
                dfs_index: root_index,
                parent: CfgDfsSeq::NULL_PARENT,
                children: SmallVec::new(),
            });
            for &exit_bb in &exits {
                self.pre_dfs_visit(exit_bb, root_index);
            }
            root_index
        };
        self.0.virt_index = Some(root_index);
        let vexit_node = &mut self.0.nodes[root_index];
        vexit_node.dfs_index = root_index;
        vexit_node.children = {
            let mut exits_index = SmallVec::with_capacity(exits.len());
            for exit_bb in &exits {
                let exit_id = self.0.unseq[exit_bb];
                exits_index.push(exit_id);
            }
            exits_index
        };
        Ok(())
    }

    /// "backward succs" are actually preds.
    fn get_succs(&mut self, block: BlockID, frame: &mut DfsFrame) {
        let allocs = self.0.ir_allocs;
        let preds = block.get_preds(allocs);
        for (_, pred_jt) in preds.iter(&allocs.jts) {
            let termi = pred_jt.terminator.get().expect("Null terminator");
            let parent_bb = termi.get_parent(allocs).expect("Null parent");
            frame.push(parent_bb);
        }
    }
}

impl<'ir> DfsBackwardBuilder<'ir> {
    fn dump_exits(&self) -> CfgRes<SmallVec<[BlockID; 4]>> {
        let allocs = self.0.ir_allocs;
        let mut exits = SmallVec::new();
        let blocks = CfgDfsSeq::func_blocks(allocs, self.0.func_id)?;
        for (id, bb) in blocks {
            use crate::ir::TerminatorID::*;
            match bb.get_terminator(allocs) {
                Unreachable(_) | Ret(_) => exits.push(id),
                Jump(_) | Br(_) | Switch(_) => continue,
            }
        }
        Ok(exits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::cases::test_case_cfg_deep_while_br;

    #[test]
    fn dfs_pre_post_properties() {
        let builder = test_case_cfg_deep_while_br();
        let module = builder.module;
        let allocs = &module.allocs;
        let fid = module
            .get_global_by_name("main")
            .map(FuncID::raw_from)
            .expect("func not found");

        let pre = CfgDfsSeq::new_pre(allocs, fid).expect("pre dfs failed");
        let post = CfgDfsSeq::new_post(allocs, fid).expect("post dfs failed");

        // find a parent node (should be entry)
        let (parent_idx_pre, parent_node) = pre
            .nodes
            .iter()
            .enumerate()
            .find(|(_, n)| !n.children.is_empty())
            .expect("no parent found");
        let child_idx_pre = parent_node.children[0];

        // map to the block ids so we can locate same nodes in post-order
        let parent_block = match pre.nodes[parent_idx_pre].block {
            CfgBlockStat::Block(b) => b,
            CfgBlockStat::Virtual => panic!("parent is virtual"),
        };
        let child_block = match pre.nodes[child_idx_pre].block {
            CfgBlockStat::Block(b) => b,
            CfgBlockStat::Virtual => panic!("child is virtual"),
        };

        let parent_idx_post = *post
            .unseq
            .get(&parent_block)
            .expect("parent missing in post");
        let child_idx_post = *post.unseq.get(&child_block).expect("child missing in post");

        assert!(pre.nodes[parent_idx_pre].dfs_index < pre.nodes[child_idx_pre].dfs_index);
        assert!(post.nodes[parent_idx_post].dfs_index > post.nodes[child_idx_post].dfs_index);
    }

    #[test]
    fn dfs_backward_has_virtual_root() {
        let builder = test_case_cfg_deep_while_br();
        let module = builder.module;
        let allocs = &module.allocs;
        let fid = module
            .get_global_by_name("main")
            .map(FuncID::raw_from)
            .expect("func not found");

        let back_pre = CfgDfsSeq::new_back_pre(allocs, fid).expect("back pre failed");
        let back_post = CfgDfsSeq::new_back_post(allocs, fid).expect("back post failed");

        assert!(back_pre.virt_index.is_some());
        assert!(back_post.virt_index.is_some());
        let vidx = back_pre.virt_index.unwrap();
        assert!(!back_pre.nodes[vidx].children.is_empty());
    }
}
