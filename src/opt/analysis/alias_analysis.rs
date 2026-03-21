use std::{fmt, num::NonZeroU64};

/// 内存位置大小的语义表示
/// 注意：Remusys-IR 不支持 Scalable Vector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocSize {
    /// 精确大小 (必须是 N 字节)
    Precise(u64),

    /// 上界大小 (最多 N 字节)
    UpperBound(NonZeroU64),

    /// 哨兵：指针之后的任意位置
    AfterPtr,

    /// 哨兵：指针前后任意位置
    Any,
}

impl LocSize {
    /// 构造一个上界大小
    pub const fn upper_bound(max: u64) -> Self {
        match NonZeroU64::new(max) {
            Some(nz) => Self::UpperBound(nz),
            None => Self::Precise(0),
        }
    }

    /// 核心：合并两个大小 (取并集)
    pub fn union(self, other: Self) -> Self {
        use LocSize::*;

        if self == other {
            return self;
        }

        match (self, other) {
            (Any, _) | (_, Any) => Any,
            (AfterPtr, _) | (_, AfterPtr) => AfterPtr,
            (Precise(a), Precise(b)) => Self::upper_bound(a.max(b)),
            (Precise(a), UpperBound(b)) => Self::upper_bound(a.max(b.get())),
            (UpperBound(a), Precise(b)) => Self::upper_bound(a.get().max(b)),
            (UpperBound(a), UpperBound(b)) => Self::UpperBound(a.max(b)),
        }
    }

    /// 获取具体数值 (如果不是哨兵)
    pub const fn value(self) -> Option<u64> {
        match self {
            Self::Precise(v) => Some(v),
            Self::UpperBound(v) => Some(v.get()),
            _ => None,
        }
    }

    pub const fn is_precise(self) -> bool {
        matches!(self, Self::Precise(_))
    }
}

// 格式化输出
impl fmt::Display for LocSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Precise(v) => write!(f, "{}", v),
            Self::UpperBound(v) => write!(f, "<={}", v),
            Self::AfterPtr => write!(f, "AfterPtr"),
            Self::Any => write!(f, "Any"),
        }
    }
}
