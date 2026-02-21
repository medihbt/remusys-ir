use crate::{
    ir::{BlockID, FuncID, IRAllocs, ISubInstID},
    opt::{CfgDfsSeq, DfsOrder},
};
use smallvec::SmallVec;
use std::{
    collections::{BTreeSet, HashMap},
    fmt::Debug,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfgBlockStat {
    Block(BlockID),
    Virtual,
}
impl From<BlockID> for CfgBlockStat {
    fn from(bid: BlockID) -> Self {
        CfgBlockStat::Block(bid)
    }
}
impl From<Option<BlockID>> for CfgBlockStat {
    fn from(bid: Option<BlockID>) -> Self {
        match bid {
            Some(bid) => CfgBlockStat::Block(bid),
            None => CfgBlockStat::Virtual,
        }
    }
}
impl CfgBlockStat {
    pub fn is_virtual(&self) -> bool {
        matches!(self, CfgBlockStat::Virtual)
    }

    pub fn map<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(BlockID) -> R,
    {
        match self {
            CfgBlockStat::Block(bid) => Some(f(*bid)),
            CfgBlockStat::Virtual => None,
        }
    }
    pub fn unwrap(&self) -> BlockID {
        match self {
            CfgBlockStat::Block(bid) => *bid,
            CfgBlockStat::Virtual => panic!("called `CfgBlockStat::unwrap()` on a `Virtual` value"),
        }
    }
    pub fn expect(&self, msg: &str) -> BlockID {
        match self {
            CfgBlockStat::Block(bid) => *bid,
            CfgBlockStat::Virtual => panic!("{}", msg),
        }
    }
    pub fn expect_string(&self, msg: String) -> BlockID {
        match self {
            CfgBlockStat::Block(bid) => *bid,
            CfgBlockStat::Virtual => panic!("{}", msg),
        }
    }
}

#[derive(thiserror::Error)]
pub enum CfgErr {
    #[error("function {0:?} is extern")]
    FuncIsExtern(FuncID),

    #[error("function {0:?} cannot exit")]
    FuncCannotExit(FuncID),
}
pub type CfgRes<T = ()> = Result<T, CfgErr>;
impl Debug for CfgErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

#[derive(Default)]
pub struct CfgCache {
    nodes: HashMap<BlockID, CfgNode>,
}
pub struct CfgNode {
    pub block: BlockID,
    pub preds: SmallVec<[BlockID; 8]>,
    pub succs: SmallVec<[BlockID; 8]>,
}

impl CfgNode {
    pub fn n_preds(&self) -> usize {
        self.preds.len()
    }
    pub fn n_succs(&self) -> usize {
        self.succs.len()
    }

    fn contains(slice: &[BlockID], target: BlockID) -> bool {
        if slice.len() < 8 { slice.contains(&target) } else { slice.binary_search(&target).is_ok() }
    }
    pub fn has_pred(&self, pred: BlockID) -> bool {
        Self::contains(&self.preds, pred)
    }
    pub fn has_succ(&self, succ: BlockID) -> bool {
        Self::contains(&self.succs, succ)
    }
}

impl CfgCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_node(&mut self, allocs: &IRAllocs, block: BlockID) -> &CfgNode {
        self.nodes
            .entry(block)
            .or_insert_with(|| Self::make_node(allocs, block))
    }
    pub fn get_preds(&mut self, allocs: &IRAllocs, block: BlockID) -> &[BlockID] {
        &self.get_node(allocs, block).preds
    }
    pub fn get_succs(&mut self, allocs: &IRAllocs, block: BlockID) -> &[BlockID] {
        &self.get_node(allocs, block).succs
    }
    pub fn has_pred(&mut self, allocs: &IRAllocs, block: BlockID, pred: BlockID) -> bool {
        CfgNode::contains(self.get_preds(allocs, block), pred)
    }
    pub fn has_succ(&mut self, allocs: &IRAllocs, block: BlockID, succ: BlockID) -> bool {
        CfgNode::contains(self.get_succs(allocs, block), succ)
    }
    pub fn invalidate_node(&mut self, block: BlockID) {
        self.nodes.remove(&block);
    }

    fn make_node(allocs: &IRAllocs, block: BlockID) -> CfgNode {
        let mut preds = BTreeSet::new();
        let mut succs = BTreeSet::new();
        for (_, pred_jt) in block.get_preds(allocs).iter(&allocs.jts) {
            let pred_inst = pred_jt.terminator.get().expect("pred jt has no terminator");
            let pred_block = pred_inst
                .get_parent(allocs)
                .expect("pred inst has no parent block");
            preds.insert(pred_block);
        }
        let terminator = block.get_terminator(allocs);
        for succ in terminator.blocks_iter(allocs) {
            succs.insert(succ.unwrap());
        }
        CfgNode {
            block,
            preds: SmallVec::from_iter(preds),
            succs: SmallVec::from_iter(succs),
        }
    }
}

pub struct CfgSnapshot {
    pub func: FuncID,
    pub entry: BlockID,
    pub exits: SmallVec<[BlockID; 4]>,
    pub nodes: HashMap<BlockID, CfgNode>,
}

impl CfgSnapshot {
    pub fn new(allocs: &IRAllocs, func: FuncID) -> CfgRes<Self> {
        let Some(body) = func.get_body(allocs) else {
            return Err(CfgErr::FuncIsExtern(func));
        };
        let mut cache = CfgCache { nodes: HashMap::with_capacity(body.blocks.len()) };
        let entry = body.entry;
        let mut exits: SmallVec<[BlockID; 4]> = SmallVec::new();
        for (block, _) in body.blocks.iter(&allocs.blocks) {
            let node = cache.get_node(allocs, block);
            if node.succs.is_empty() {
                exits.push(block);
            }
        }
        // allows this function to have no exits
        let snapshot = CfgSnapshot { func, entry, exits, nodes: cache.nodes };
        Ok(snapshot)
    }

    pub fn succ_of(&self, block: impl Into<CfgBlockStat>) -> Option<&[BlockID]> {
        match block.into() {
            CfgBlockStat::Block(bid) => self.nodes.get(&bid).map(|n| n.succs.as_slice()),
            CfgBlockStat::Virtual => None,
        }
    }
    pub fn pred_of(&self, block: impl Into<CfgBlockStat>) -> Option<&[BlockID]> {
        match block.into() {
            CfgBlockStat::Block(bid) => self.nodes.get(&bid).map(|n| n.preds.as_slice()),
            CfgBlockStat::Virtual => {
                let slice = self.exits.as_slice();
                if slice.is_empty() { None } else { Some(slice) }
            }
        }
    }

    pub fn has_succ(&self, block: impl Into<CfgBlockStat>, succ: BlockID) -> bool {
        match block.into() {
            CfgBlockStat::Block(bid) => self.nodes[&bid].has_succ(succ),
            CfgBlockStat::Virtual => false,
        }
    }
    pub fn has_pred(&self, block: impl Into<CfgBlockStat>, pred: BlockID) -> bool {
        match block.into() {
            CfgBlockStat::Block(bid) => self.nodes[&bid].has_pred(pred),
            CfgBlockStat::Virtual => CfgNode::contains(&self.exits, pred),
        }
    }

    pub fn write_to_dot(&self, allocs: &IRAllocs, writer: &mut dyn std::io::Write) {
        let dfs = CfgDfsSeq::new(allocs, self.func, DfsOrder::RevPost).unwrap();
        writeln!(writer, "digraph CFG {{").unwrap();
        for dfs_node in &dfs.nodes {
            let dfn = dfs_node.dfs_index;
            let label = match dfs_node.block {
                CfgBlockStat::Block(block_id) => format!("{:#x}", block_id.inner()),
                CfgBlockStat::Virtual => "%VIRTUAL".to_string(),
            };
            writeln!(writer, "    {dfn} [label=\"{label}\"];").unwrap();
            let node = match dfs_node.block {
                CfgBlockStat::Block(bid) => &self.nodes[&bid],
                CfgBlockStat::Virtual => continue,
            };
            for &succ in node.succs.iter() {
                let succ_dfn = dfs.block_dfn(succ);
                writeln!(writer, "    {dfn} -> {succ_dfn};").unwrap();
            }
        }
        writeln!(writer, "}}").unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{FuncID, ISubGlobalID};
    use crate::testing::cases::test_case_cfg_deep_while_br;
    use std::fs::File;

    #[test]
    fn test_cfg_snapshot() {
        let module = test_case_cfg_deep_while_br().module;
        let allocs = &module.allocs;
        let func_id = module
            .get_global_by_name("main")
            .map(FuncID::raw_from)
            .unwrap();
        let cfg_snapshot = CfgSnapshot::new(allocs, func_id).unwrap();
        let mut cfg_file =
            File::create("../target/test_cfg_snapshot.dot").expect("Failed to create dot file");
        cfg_snapshot.write_to_dot(allocs, &mut cfg_file);
    }
}
