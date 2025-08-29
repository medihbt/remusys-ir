use crate::{
    base::{FixBitSet, INullableValue, SlabRef},
    ir::{Attr, IRWriter, attributes::attrset::AttrSet},
};
use slab::Slab;

#[derive(Debug, Clone, Default)]
pub struct AttrList {
    pub includes: Vec<AttrListID>,
    pub self_id: AttrListID,
    pub attr: AttrSet,
}

impl AttrList {
    /// 直接插入属性到本地属性集
    pub fn insert_attr(&mut self, attr: Attr) {
        self.attr.merge_attr(attr);
    }

    /// 检查是否包含特定属性（包括继承的）
    pub fn has_attr(&self, attr: &Attr, alloc: &Slab<AttrList>) -> bool {
        // 先检查本地属性
        if self.attr.has_attr(attr) {
            return true;
        }

        // 然后检查继承的属性
        for &include_id in &self.includes {
            if let Some(included_list) = alloc.get(include_id.0) {
                if included_list.has_attr(attr, alloc) {
                    return true;
                }
            }
        }

        false
    }

    /// 获取完整的合并属性集（包括继承）
    pub fn get_merged_attrs(&self, alloc: &Slab<AttrList>) -> AttrSet {
        let mut merged = self.attr.clone();

        // 合并所有包含的属性列表
        for &include_id in &self.includes {
            if let Some(included_list) = alloc.get(include_id.0) {
                let included_attrs = included_list.get_merged_attrs(alloc);
                merged.merge_from(&included_attrs);
            }
        }

        merged
    }

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

impl Default for AttrListID {
    fn default() -> Self {
        AttrListID::new_null()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::attributes::{Attr, InlineAttr};

    #[test]
    fn test_attrlist_better_api() {
        let mut alloc = Slab::new();

        // 创建基础属性列表
        let mut base_list = AttrList::default();
        base_list.insert_attr(Attr::NoReturn);
        base_list.insert_attr(Attr::Inline(InlineAttr::Hint));

        let base_id = AttrListID(alloc.insert(base_list));

        // 创建扩展属性列表，继承基础列表
        let mut ext_list = AttrList::default();
        ext_list.includes.push(base_id);
        ext_list.insert_attr(Attr::NoRecurse);
        ext_list.insert_attr(Attr::Inline(InlineAttr::Always)); // 覆盖继承的值

        // 现在 API 更清晰了：

        // ✅ 清楚地访问本地属性
        assert!(ext_list.attr.norecurse);
        assert_eq!(ext_list.attr.inline, Some(InlineAttr::Always));

        // ✅ 清楚地获取合并后的属性（包括继承）
        let merged = ext_list.get_merged_attrs(&alloc);
        assert!(merged.noreturn); // 从基础列表继承
        assert!(merged.norecurse); // 本地属性
        assert_eq!(merged.inline, Some(InlineAttr::Always)); // 覆盖继承的值

        // ✅ 清楚地检查属性（包括继承）
        assert!(ext_list.has_attr(&Attr::NoReturn, &alloc)); // 继承的
        assert!(ext_list.has_attr(&Attr::NoRecurse, &alloc)); // 本地的
    }
}
