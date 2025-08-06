#[derive(Debug, Clone)]
pub enum FixBitSet<const FIXN: usize = 2> {
    /// 小容量的 BitSet，完全在栈上存储
    Small([u64; FIXN], usize),
    /// 大容量的 BitSet，使用堆分配
    /// usize 表示实际使用的位数
    Large(Box<[u64]>, usize),
}

impl<const N: usize> FixBitSet<N> {
    pub fn with_len(len: usize) -> Self {
        if len <= N * 64 {
            Self::Small([0; N], len)
        } else {
            let size = (len + 63) / 64;
            let vec = vec![0; size];
            Self::Large(vec.into_boxed_slice(), len)
        }
    }

    fn get_mut_slice(&mut self) -> &mut [u64] {
        match self {
            Self::Small(arr, _) => arr,
            Self::Large(boxed, _) => boxed,
        }
    }
    fn get_slice(&self) -> &[u64] {
        match self {
            Self::Small(arr, _) => arr,
            Self::Large(boxed, _) => boxed,
        }
    }

    pub fn enable(&mut self, bit: usize) {
        if bit >= self.len() {
            return; // 超出逻辑长度，静默忽略
        }
        let idx = bit / 64;
        let offset = bit % 64;
        self.get_mut_slice()[idx] |= 1 << offset;
    }
    pub fn disable(&mut self, bit: usize) {
        if bit >= self.len() {
            return; // 超出逻辑长度，静默忽略
        }
        let idx = bit / 64;
        let offset = bit % 64;
        self.get_mut_slice()[idx] &= !(1 << offset);
    }

    pub fn try_get(&self, bit: usize) -> Option<bool> {
        if bit >= self.len() {
            return None; // 超出逻辑长度
        }
        let idx = bit / 64;
        let offset = bit % 64;
        Some((self.get_slice()[idx] & (1 << offset)) != 0)
    }
    pub fn get(&self, bit: usize) -> bool {
        self.try_get(bit).unwrap_or(false)
    }
    pub fn set(&mut self, bit: usize, value: bool) {
        if value {
            self.enable(bit);
        } else {
            self.disable(bit);
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Small(_, len) => *len,
            Self::Large(_, len) => *len,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 清空所有位
    pub fn clear(&mut self) {
        for word in self.get_mut_slice() {
            *word = 0;
        }
    }

    /// 计算设置为 true 的位数
    pub fn count_ones(&self) -> usize {
        let mut count = 0;
        let slice = self.get_slice();
        let full_words = self.len() / 64;
        
        // 计算完整 u64 字的位数
        for i in 0..full_words {
            count += slice[i].count_ones() as usize;
        }
        
        // 处理最后一个不完整的字
        let remaining_bits = self.len() % 64;
        if remaining_bits > 0 && full_words < slice.len() {
            let mask = (1u64 << remaining_bits) - 1;
            count += (slice[full_words] & mask).count_ones() as usize;
        }
        
        count
    }
}

pub struct FixBitSetIter<'a> {
    bitset: &'a [u64],
    len: usize,
    current: usize,
}

impl<'a> FixBitSetIter<'a> {
    pub fn new<const N: usize>(bitset: &'a FixBitSet<N>) -> Self {
        let (slice, len) = match bitset {
            FixBitSet::Small(arr, len) => (&arr[..], *len),
            FixBitSet::Large(boxed, len) => (&boxed[..], *len),
        };
        Self { bitset: slice, len, current: 0 }
    }

    pub fn get(&self) -> Option<bool> {
        if self.current >= self.len {
            return None;
        }
        let idx = self.current / 64;
        let offset = self.current % 64;
        Some((self.bitset[idx] & (1 << offset)) != 0)
    }
}

impl<'a> Iterator for FixBitSetIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.len {
            let idx = self.current / 64;
            let offset = self.current % 64;
            if (self.bitset[idx] & (1 << offset)) != 0 {
                let result = self.current;
                self.current += 1;
                return Some(result);
            }
            self.current += 1;
        }
        None
    }
}

impl<'a, const N: usize> IntoIterator for &'a FixBitSet<N> {
    type Item = usize;
    type IntoIter = FixBitSetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FixBitSetIter::new(self)
    }
}

#[cfg(test)]
mod testing {
    use super::FixBitSet;

    #[test]
    fn test_small_bitset() {
        let mut bitset = FixBitSet::<2>::with_len(100); // 应该使用 Small 变体
        
        // 测试基本操作
        assert_eq!(bitset.len(), 100);
        assert!(!bitset.get(50));
        
        bitset.enable(50);
        assert!(bitset.get(50));
        
        bitset.disable(50);
        assert!(!bitset.get(50));
        
        // 测试边界检查
        bitset.enable(150); // 超出范围，应该被忽略
        assert!(!bitset.get(150));
        assert_eq!(bitset.try_get(150), None);
    }

    #[test]
    fn test_large_bitset() {
        let mut bitset = FixBitSet::<1>::with_len(100); // 应该使用 Large 变体
        
        assert_eq!(bitset.len(), 100);
        bitset.enable(99);
        assert!(bitset.get(99));
        
        // 测试越界访问
        bitset.enable(100); // 超出范围
        assert_eq!(bitset.try_get(100), None);
    }

    #[test]
    fn test_iterator() {
        let mut bitset = FixBitSet::<2>::with_len(10);
        bitset.enable(2);
        bitset.enable(5);
        bitset.enable(7);
        
        let set_bits: Vec<usize> = bitset.into_iter().collect();
        assert_eq!(set_bits, vec![2, 5, 7]);
    }

    #[test]
    fn test_count_ones() {
        let mut bitset = FixBitSet::<2>::with_len(100);
        assert_eq!(bitset.count_ones(), 0);
        
        bitset.enable(10);
        bitset.enable(20);
        bitset.enable(99);
        assert_eq!(bitset.count_ones(), 3);
        
        bitset.disable(20);
        assert_eq!(bitset.count_ones(), 2);
    }

    #[test]
    fn test_clear() {
        let mut bitset = FixBitSet::<2>::with_len(100);
        bitset.enable(10);
        bitset.enable(50);
        assert_eq!(bitset.count_ones(), 2);
        
        bitset.clear();
        assert_eq!(bitset.count_ones(), 0);
        assert!(!bitset.get(10));
        assert!(!bitset.get(50));
    }

    #[test]
    fn test_boundary_cases() {
        // 测试 64 位边界情况
        let mut bitset = FixBitSet::<2>::with_len(65);
        bitset.enable(63); // 第一个 u64 的最后一位
        bitset.enable(64); // 第二个 u64 的第一位
        
        assert!(bitset.get(63));
        assert!(bitset.get(64));
        assert_eq!(bitset.count_ones(), 2);
        
        let set_bits: Vec<usize> = bitset.into_iter().collect();
        assert_eq!(set_bits, vec![63, 64]);
    }
}
