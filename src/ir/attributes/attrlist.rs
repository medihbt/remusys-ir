use std::{
    cell::{Ref, RefCell},
    ops::Deref,
};

use crate::{
    base::{FixBitSet, INullableValue, MixRef, SlabRef},
    ir::{Attr, IRAllocsEditable, IRWriter, attributes::attrset::AttrSet},
};
use slab::Slab;

#[derive(Debug, Clone, Default)]
pub struct AttrList {
    pub includes: Vec<AttrListID>,
    pub self_id: AttrListID,
    attr: AttrSet,
    merged_cache: RefCell<Option<Box<AttrSet>>>,
}

impl AttrList {
    pub fn new(includes: Vec<AttrListID>, attr: AttrSet) -> Self {
        Self {
            includes,
            self_id: AttrListID::new_null(),
            attr,
            merged_cache: RefCell::new(None),
        }
    }

    pub fn attached(&self) -> bool {
        self.self_id.is_nonnull()
    }

    pub fn attr(&self) -> &AttrSet {
        &self.attr
    }
    pub fn try_attrs_mut(&mut self) -> Result<&mut AttrSet, AttrList> {
        if self.attached() {
            Err(self.clone())
        } else {
            self.merged_cache.get_mut().take();
            Ok(&mut self.attr)
        }
    }
    pub fn attr_mut(&mut self) -> &mut AttrSet {
        if self.attached() {
            panic!("Cannot modify attributes of an attached AttrList");
        }
        self.merged_cache.get_mut().take();
        &mut self.attr
    }

    /// 直接插入属性到本地属性集
    pub fn add_attr(&mut self, attr: Attr) -> &mut Self {
        self.attr_mut().merge_attr(attr);
        self
    }

    /// 检查是否包含特定属性（包括继承的）
    pub fn has_attr(&self, attr: &Attr, alloc: &Slab<AttrList>) -> bool {
        self.merge_all(alloc).has_attr(attr)
    }

    /// 获取完整的合并属性集（包括继承）
    pub fn merge_all(&self, alloc: &Slab<AttrList>) -> MixRef<'_, AttrSet> {
        let mut detect_map = FixBitSet::with_len(alloc.capacity());
        detect_map.set(self.self_id.0, true);
        self.do_merge_all(alloc, &mut detect_map)
    }

    fn do_merge_all(
        &self,
        alloc: &Slab<AttrList>,
        detect_map: &mut FixBitSet<2>,
    ) -> MixRef<'_, AttrSet> {
        if self.includes.is_empty() {
            return MixRef::Fix(&self.attr);
        }

        if let Some(_) = self.merged_cache.borrow().as_ref() {
            return MixRef::Dyn(Ref::map(self.merged_cache.borrow(), |c| {
                c.as_ref().map(|x| x.deref()).unwrap()
            }));
        }

        let mut merged = self.attr.clone();
        for &include_id in &self.includes {
            if detect_map.get(include_id.0) {
                panic!(
                    "Cycle detected [{:?} => {:?}] in attribute list inclusion",
                    self.self_id, include_id
                );
            }
            if let Some(included_list) = alloc.get(include_id.0) {
                let included_attrs = included_list.do_merge_all(alloc, detect_map);
                merged.merge_from(&included_attrs);
            }
        }
        self.merged_cache.replace(Some(Box::new(merged)));
        MixRef::Dyn(Ref::map(self.merged_cache.borrow(), |c| {
            c.as_ref().map(|x| x.deref()).unwrap()
        }))
    }

    pub fn is_empty(&self) -> bool {
        self.attr.is_empty() && self.includes.is_empty()
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
    pub fn from_alloc(alloc: &mut Slab<AttrList>, mut data: AttrList) -> Self {
        let index = alloc.vacant_key();
        data.self_id = AttrListID(index);
        alloc.insert(data);
        Self(index)
    }
    pub fn new(allocs: &mut impl IRAllocsEditable, data: AttrList) -> Self {
        let alloc = allocs.get_allocs_mutref();
        Self::from_alloc(&mut alloc.attrs, data)
    }
    pub fn from_iter(
        allocs: &mut impl IRAllocsEditable,
        includes: Vec<AttrListID>,
        attrs: impl IntoIterator<Item = Attr>,
    ) -> Self {
        let attrset = AttrSet::from_attrs(attrs);
        Self::new(allocs, AttrList::new(includes, attrset))
    }

    pub fn merge_all(self, alloc: &Slab<AttrList>) -> MixRef<'_, AttrSet> {
        self.to_data(alloc).merge_all(alloc)
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
        base_list.add_attr(Attr::NoReturn);
        base_list.add_attr(Attr::Inline(InlineAttr::Hint));

        let base_id = AttrListID(alloc.insert(base_list));

        // 创建扩展属性列表，继承基础列表
        let mut ext_list = AttrList::default();
        ext_list.includes.push(base_id);
        ext_list.add_attr(Attr::NoRecurse);
        ext_list.add_attr(Attr::Inline(InlineAttr::Always)); // 覆盖继承的值

        // 现在 API 更清晰了：

        // ✅ 清楚地访问本地属性
        assert!(ext_list.attr.norecurse);
        assert_eq!(ext_list.attr.inline, Some(InlineAttr::Always));

        // ✅ 清楚地获取合并后的属性（包括继承）
        let merged = ext_list.merge_all(&alloc);
        assert!(merged.noreturn); // 从基础列表继承
        assert!(merged.norecurse); // 本地属性
        assert_eq!(merged.inline, Some(InlineAttr::Always)); // 覆盖继承的值

        // ✅ 清楚地检查属性（包括继承）
        assert!(ext_list.has_attr(&Attr::NoReturn, &alloc)); // 继承的
        assert!(ext_list.has_attr(&Attr::NoRecurse, &alloc)); // 本地的
    }
}
