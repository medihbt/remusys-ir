use crate::{
    base::{FixBitSet, SlabRef},
    ir::{IRWriter, attributes::attrset::AttrSet},
};
use slab::Slab;

#[derive(Debug, Clone)]
pub struct AttrList {
    pub includes: Vec<AttrListID>,
    pub self_id: AttrListID,
    pub attr: AttrSet,
}

impl AttrList {
    pub fn fmt_ir(&self, f: &IRWriter) -> std::io::Result<()> {
        for include in &self.includes {
            include.fmt_ir(f)?;
            f.write_str(" ")?;
        }
        self.attr.fmt_ir(f)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AttrListID(pub usize);

impl SlabRef for AttrListID {
    type RefObject = AttrList;

    fn from_handle(handle: usize) -> Self {
        Self(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl AttrListID {
    pub fn merge_all(self, alloc: &mut Slab<AttrList>) -> AttrSet {
        let mut visited = FixBitSet::<2>::with_len(alloc.len());
        let mut stack = vec![self];
        let mut merged = AttrSet::new();

        while let Some(current_id) = stack.pop() {
            if visited.get(current_id.0) {
                continue; // 已访问，跳过
            }
            visited.enable(current_id.0);

            if let Some(attr_list) = alloc.get(current_id.0) {
                // 先处理包含的 AttrList
                for &included_id in &attr_list.includes {
                    if !visited.get(included_id.0) {
                        stack.push(included_id);
                    }
                }
                // 然后合并当前 AttrList 的属性
                merged.merge_from(&attr_list.attr);
            }
        }

        merged
    }

    pub fn fmt_ir(&self, f: &IRWriter) -> std::io::Result<()> {
        write!(f, "#{}", self.0)
    }
}
