/// DFS order for traversing a graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DfsOrder {
    Pre,
    Post,
    ReversePre,
    ReversePost,
}

impl DfsOrder {
    /// `ReversePre` 和 `ReversePost` 在 Remusys 的实现中基本上就是
    /// 先 `Post` 或者 `Pre`, 然后再翻转 DFS 顺序. 这里放一个工具方法
    /// 方便使用.
    pub const fn should_reverse(self) -> bool {
        matches!(self, DfsOrder::ReversePre | DfsOrder::ReversePost)
    }

    /// 逆前序和逆后序是从前序/后序遍历翻转顺序得到的. 该方法给出的是
    /// 某个遍历顺序第一步应当做什么.
    /// 
    /// 对于前序与后序遍历, 第一步就是它们自己.
    /// 
    /// 对于逆前序遍历，第一步是前序遍历. 对于逆后序遍历，第一步是后序遍历.
    pub const fn get_first_step(self) -> Self {
        match self {
            Self::Pre | Self::ReversePre => Self::Pre,
            Self::Post | Self::ReversePost => Self::Post,
        }
    }
}
