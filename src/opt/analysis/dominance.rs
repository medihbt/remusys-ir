use crate::{base::DSU, ir::BlockID, opt::CfgBlockStat};
use std::collections::HashSet;

pub struct DominatorTreeNode {
    pub block: CfgBlockStat,
    /// 半支配结点. 使用 CfgBlockStat 来考虑根节点是 Virtual Exit 的情况.
    pub semidom: CfgBlockStat,
    /// 直接支配结点. 使用 CfgBlockStat 来考虑根节点是 Virtual Exit 的情况.
    pub idom: CfgBlockStat,
    /// 所有的子结点. 与重构前不同的是, 这次不要懒加载，在构建时就要把所有子结点算出来.
    pub children: HashSet<BlockID>,
}
