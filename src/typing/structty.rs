use std::{
    cell::{Cell, Ref},
    hash::{Hash, Hasher},
    io::Write,
    ops::Deref,
};

use crate::{
    base::SlabRef,
    typing::{
        IValType, TypeAllocs, TypeContext, TypeFormatter, TypeMismatchError, TypingRes,
        ValTypeClass, ValTypeID,
    },
};

/// 无名结构体类型, 相当于 Rust 的 Tuple.
///
/// 有时结构体类型在创建后要修改一下再放到堆中, 如果不采用懒加载的话就要重新
/// 计算 offset, 这个过程可能会很复杂. 因此结构体类型的 offset 和 size 计算
/// 均采用懒加载的方式.
#[derive(Debug, Clone)]
pub struct StructTypeData {
    pub fields: Box<[ValTypeID]>,
    pub packed: bool,
    size_cache: Box<[Cell<usize>]>,
    cache_top: Cell<usize>,
    align_top: Cell<usize>,
    hash_cache: Cell<usize>,
}

impl StructTypeData {
    pub fn new(elemty: Box<[ValTypeID]>, packed: bool) -> Self {
        let size_cache = vec![Cell::new(0); elemty.len()].into_boxed_slice();
        let align_top = if packed { 1 } else { 0 };
        Self {
            fields: elemty,
            packed,
            size_cache,
            cache_top: Cell::new(0),
            align_top: Cell::new(align_top),
            hash_cache: Cell::new(0),
        }
    }

    fn update_cache(&self, index: usize, alloc: &TypeAllocs, tctx: &TypeContext) -> usize {
        let old_top = self.cache_top.get();
        let mut size = if old_top == 0 { 0 } else { self.size_cache[old_top - 1].get() };
        for id in old_top..index {
            let elem_size = self.elem_size_of(alloc, tctx, id);
            if !self.packed {
                let elem_align = self.elem_align_of(alloc, tctx, id);
                debug_assert!(
                    elem_align.is_power_of_two(),
                    "Element alignment must be a power of two"
                );
                size = size.next_multiple_of(elem_align);
            }
            size += elem_size;
            self.size_cache[id].set(size);
        }
        self.cache_top.set(index);
        size
    }

    fn elem_size_of(&self, alloc: &TypeAllocs, tctx: &TypeContext, id: usize) -> usize {
        let elemty = self.fields[id];
        let opt_size = elemty.try_get_size_full(alloc, tctx);
        Self::unpack_sized(elemty, opt_size)
    }

    fn elem_align_of(&self, alloc: &TypeAllocs, tctx: &TypeContext, id: usize) -> usize {
        let elemty = self.fields[id];
        let align = elemty.try_get_align_full(alloc, tctx);
        let align = Self::unpack_sized(elemty, align);
        self.align_top.set(align.max(self.align_top.get()));
        align
    }

    fn unpack_sized(elemty: ValTypeID, opt_size: Option<usize>) -> usize {
        match opt_size {
            Some(x) => x,
            None => panic!("Element type {elemty:?} should make instance but got None"),
        }
    }

    fn get_hash(&self) -> usize {
        if self.hash_cache.get() != 0 {
            return self.hash_cache.get();
        }
        let (hash, _) = Self::make_hash_and_len(self.packed, self.fields.iter().cloned());
        self.hash_cache.set(hash);
        hash
    }

    fn make_hash_and_len(packed: bool, fields: impl Iterator<Item = ValTypeID>) -> (usize, usize) {
        use std::collections::hash_map::DefaultHasher;
        let mut state = DefaultHasher::new();
        packed.hash(&mut state);
        let mut len = 0;
        for field in fields {
            field.hash(&mut state);
            len += 1;
        }
        (state.finish() as usize, len)
    }

    pub(super) fn get_offset_full(
        &self,
        index: usize,
        alloc: &TypeAllocs,
        tctx: &TypeContext,
    ) -> Option<usize> {
        // 处理特殊情况: index 一眼就能看出是否出界
        if index == 0 {
            return Some(0);
        } else if index <= self.cache_top.get() {
            return Some(self.size_cache[index - 1].get());
        } else if index >= self.fields.len() {
            return None;
        }
        // index 没出界, 但缓存也还没加载到这个 index 上. 因此
        // 需要从 _size_top 开始, 逐个计算每个元素的偏移量.
        let size = self.update_cache(index, alloc, tctx);
        Some(size)
    }

    fn get_size_full(&self, alloc: &TypeAllocs, tctx: &TypeContext) -> usize {
        self.update_cache(self.fields.len(), alloc, tctx);
        self.size_cache[self.fields.len() - 1].get()
    }
    fn get_align_full(&self, alloc: &TypeAllocs, tctx: &TypeContext) -> usize {
        if self.packed {
            return 1; // packed 结构体的对齐为 1
        }
        self.update_cache(self.fields.len(), alloc, tctx);
        self.align_top.get()
    }

    pub fn get_offset(&self, index: usize, tctx: &TypeContext) -> Option<usize> {
        self.get_offset_full(index, &tctx.allocs.borrow(), tctx)
    }
}

pub struct StructOffsetIter<'a> {
    pub data: Ref<'a, StructTypeData>,
    pub alloc: Ref<'a, TypeAllocs>,
    pub tctx: &'a TypeContext,
    pub index: usize,
}

impl<'a> StructOffsetIter<'a> {
    pub fn new(type_ref: StructTypeRef, tctx: &'a TypeContext) -> Self {
        Self {
            data: type_ref.typectx_to_data(tctx),
            alloc: tctx.allocs.borrow(),
            tctx,
            index: 0,
        }
    }

    pub fn get(&self) -> Option<usize> {
        self.data
            .get_offset_full(self.index, &self.alloc, self.tctx)
    }

    pub fn get_field(&self) -> Option<ValTypeID> {
        self.data.fields.get(self.index).cloned()
    }
}

impl<'a> Iterator for StructOffsetIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.index < self.data.fields.len() {
            let offset = self.get();
            self.index += 1;
            offset
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructTypeRef(pub usize);

impl SlabRef for StructTypeRef {
    type RefObject = StructTypeData;
    fn from_handle(handle: usize) -> Self {
        Self(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl IValType for StructTypeRef {
    fn try_from_ir(value: ValTypeID) -> TypingRes<Self> {
        if let ValTypeID::Struct(s) = value {
            Ok(s)
        } else {
            Err(TypeMismatchError::NotClass(value, ValTypeClass::Struct))
        }
    }
    fn into_ir(self) -> ValTypeID {
        ValTypeID::Struct(self)
    }
    fn makes_instance(self) -> bool {
        true
    }
    fn class_id(self) -> ValTypeClass {
        ValTypeClass::Struct
    }
    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        Some(self.to_data(&alloc.structs).get_size_full(alloc, tctx))
    }
    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        Some(self.to_data(&alloc.structs).get_align_full(alloc, tctx))
    }

    fn serialize<T: Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        f.write_str("{ ")?;
        let data = self.to_data(&f.allocs.structs);
        for (i, field) in data.fields.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            field.serialize(f)?;
        }
        f.write_str(" }")
    }
}

impl StructTypeRef {
    fn typectx_to_data<'a>(self, tctx: &'a TypeContext) -> Ref<'a, StructTypeData> {
        let allocs = tctx.allocs.borrow();
        Ref::map(allocs, |allocs| self.to_data(&allocs.structs))
    }
    pub fn try_get_field(self, tctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        let data = self.typectx_to_data(tctx);
        data.fields.get(index).cloned()
    }
    pub fn get_field(self, tctx: &TypeContext, index: usize) -> ValTypeID {
        self.try_get_field(tctx, index)
            .expect("Failed to get element type from struct type")
    }
    pub fn get_nfields(self, tctx: &TypeContext) -> usize {
        self.typectx_to_data(tctx).fields.len()
    }

    pub fn fields<'a>(self, tctx: &'a TypeContext) -> Ref<'a, [ValTypeID]> {
        let data = self.typectx_to_data(tctx);
        Ref::map(data, |data| data.fields.deref())
    }
    pub fn is_packed(self, tctx: &TypeContext) -> bool {
        self.typectx_to_data(tctx).packed
    }

    pub fn try_get_offset(self, tctx: &TypeContext, index: usize) -> Option<usize> {
        self.typectx_to_data(tctx).get_offset(index, tctx)
    }
    pub fn get_offset(self, tctx: &TypeContext, index: usize) -> usize {
        self.try_get_offset(tctx, index)
            .expect("Failed to get offset from struct type")
    }

    fn from_allocs<T>(allocs: &mut TypeAllocs, packed: bool, fields: T) -> Self
    where
        T: IntoIterator<Item = ValTypeID>,
        T::IntoIter: Clone,
    {
        let iter = fields.into_iter();
        let (hash, len) = StructTypeData::make_hash_and_len(packed, iter.clone());
        for (handle, s) in allocs.structs.iter() {
            if len != s.fields.len() || packed != s.packed || hash != s.get_hash() {
                continue;
            }
            let key_iter = iter.clone();
            let s_iter = s.fields.iter().cloned();
            if key_iter.zip(s_iter).all(|(a, b)| a == b) {
                return StructTypeRef::from_handle(handle);
            }
        }
        let structty = StructTypeData::new(iter.collect(), packed);
        let handle = allocs.structs.insert(structty);
        Self(handle)
    }
    pub fn new<T>(tctx: &TypeContext, packed: bool, fields: T) -> Self
    where
        T: IntoIterator<Item = ValTypeID>,
        T::IntoIter: Clone,
    {
        let mut allocs = tctx.allocs.borrow_mut();
        Self::from_allocs(&mut allocs, packed, fields)
    }
    pub fn from_slice(tctx: &TypeContext, packed: bool, fields: &[ValTypeID]) -> Self {
        Self::new(tctx, packed, fields.iter().cloned())
    }
}
