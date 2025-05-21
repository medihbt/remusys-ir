/// DFS order for traversing a graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DfsOrder {
    Pre,
    Post,
    ReversePre,
    ReversePost,
}

impl DfsOrder {
    /// 这里放一个暴论不知道对不对：
    ///
    /// `ReversePre` 和 `ReversePost` 在 Remusys 的实现中基本上就是
    /// 先 `Post` 或者 `Pre`, 然后再翻转 DFS 顺序. 这里放一个工具方法
    /// 方便使用.
    pub fn should_reverse(self) -> bool {
        matches!(self, DfsOrder::ReversePre | DfsOrder::ReversePost)
    }

    pub fn get_first_step(self) -> Self {
        match self {
            Self::Pre | Self::ReversePost => Self::Pre,
            Self::Post | Self::ReversePre => Self::Post,
        }
    }
}
