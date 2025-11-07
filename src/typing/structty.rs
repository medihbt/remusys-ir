use crate::{
    base::ISlabID,
    typing::{
        IValType, TypeAllocs, TypeContext, TypeFormatter, TypeMismatchErr, ValTypeClass, ValTypeID,
    },
};
use smallvec::SmallVec;
use std::{
    cell::Ref,
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
};

/// Nameless structure type, similar to Tuple in Rust.
///
/// 无名结构体类型, 相当于 Rust 的 Tuple.
#[derive(Debug, Clone)]
pub struct StructTypeObj {
    pub fields: SmallVec<[ValTypeID; 8]>,
    pub packed: bool,
    /// 第 i 个元素存储第 i + 1 个字段的偏移量. 最后一个元素存储结构体总大小.
    pub offsets: SmallVec<[usize; 8]>,
    pub hash: usize,
    pub align: usize,
    pub aligned_size: usize,
}

impl StructTypeObj {
    pub fn new_raw(fields: SmallVec<[ValTypeID; 8]>, packed: bool) -> Self {
        let offsets = SmallVec::from_elem(0, fields.len());
        Self { fields, packed, offsets, hash: 0, align: 0, aligned_size: 0 }
    }

    pub fn make_hash_and_len(
        packed: bool,
        fields: impl Iterator<Item = ValTypeID>,
    ) -> (usize, usize) {
        let mut hasher = DefaultHasher::new();
        packed.hash(&mut hasher);
        let mut len = 0;
        for field in fields {
            field.hash(&mut hasher);
            len += 1;
        }
        (hasher.finish() as usize, len)
    }

    fn init_offsets(&mut self, allocs: &TypeAllocs, tctx: &TypeContext) {
        let Self { fields, packed, offsets, align, aligned_size, .. } = self;
        let mut curr_offset = 0usize;
        let mut align_log2 = 0u8;

        for i in 0..fields.len() {
            let field_ty = fields[i];
            let field_align = field_ty
                .try_get_align_full(allocs, tctx)
                .expect("StructTypeObj::init_offsets: field type should make instance");
            let field_align_log2 = field_align.ilog2() as u8;
            if !*packed {
                curr_offset = curr_offset.next_multiple_of(field_align);
            }
            curr_offset += field_ty
                .try_get_size_full(allocs, tctx)
                .expect("StructTypeObj::init_offsets: field type should make instance");
            offsets[i] = curr_offset;
            align_log2 = align_log2.max(field_align_log2);
        }
        *align = 1 << align_log2;
        if !*packed {
            curr_offset = curr_offset.next_multiple_of(*align);
        }
        *aligned_size = curr_offset;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructTypeID(pub u32);

impl ISlabID for StructTypeID {
    type RefObject = StructTypeObj;

    fn from_handle(handle: u32) -> Self {
        StructTypeID(handle)
    }

    fn into_handle(self) -> u32 {
        self.0
    }
}

impl IValType for StructTypeID {
    fn try_from_ir(ty: ValTypeID) -> super::TypingRes<Self> {
        let ValTypeID::Struct(s) = ty else {
            return Err(TypeMismatchErr::NotClass(ty, ValTypeClass::Struct));
        };
        Ok(s)
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

    fn serialize<T: Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        f.write_str("{ ")?;
        let data = self.deref(&f.allocs.structs);
        for (i, field) in data.fields.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            field.serialize(f)?;
        }
        f.write_str(" }")
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, _: &TypeContext) -> Option<usize> {
        let obj = self.deref(&alloc.structs);
        Some(obj.offsets.last().copied().unwrap_or(0))
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, _: &TypeContext) -> Option<usize> {
        let obj = self.deref(&alloc.structs);
        Some(obj.align)
    }
}

impl StructTypeID {
    pub fn deref_ir(self, tctx: &TypeContext) -> Ref<'_, StructTypeObj> {
        let allocs = tctx.allocs.borrow();
        Ref::map(allocs, |a| self.deref(&a.structs))
    }

    pub fn get_fields(self, tctx: &TypeContext) -> Ref<'_, [ValTypeID]> {
        let allocs = tctx.allocs.borrow();
        Ref::map(allocs, |a| &self.deref(&a.structs).fields[..])
    }
    pub fn get_nfields(self, tctx: &TypeContext) -> usize {
        self.get_fields(tctx).len()
    }
    pub fn get_offsets(self, tctx: &TypeContext) -> Ref<'_, [usize]> {
        let allocs = tctx.allocs.borrow();
        Ref::map(allocs, |a| &self.deref(&a.structs).offsets[..])
    }

    pub fn is_packed(self, tctx: &TypeContext) -> bool {
        let allocs = tctx.allocs.borrow();
        self.deref(&allocs.structs).packed
    }

    pub fn try_get_offset(self, tctx: &TypeContext, index: usize) -> Option<usize> {
        if index == 0 {
            return Some(0);
        }
        let offsets = self.get_offsets(tctx);
        offsets.get(index - 1).copied()
    }
    pub fn get_offset(self, tctx: &TypeContext, index: usize) -> usize {
        self.try_get_offset(tctx, index)
            .expect("StructTypeID::get_offset: index out of bounds")
    }

    pub fn new<T>(tctx: &TypeContext, packed: bool, fields: T) -> Self
    where
        T: IntoIterator<Item = ValTypeID>,
        T::IntoIter: Clone,
    {
        let mut allocs = tctx.allocs.borrow_mut();
        let iter = fields.into_iter();
        let (hash, len) = StructTypeObj::make_hash_and_len(packed, iter.clone());
        let alloc_struct = &allocs.structs;
        for (handle, st) in alloc_struct {
            if st.hash != hash || st.fields.len() != len {
                continue;
            }
            if st.fields.iter().zip(iter.clone()).all(|(a, b)| *a == b) {
                return Self(handle as u32);
            }
        }
        let mut new_struct = StructTypeObj::new_raw(iter.collect(), packed);
        new_struct.hash = hash;
        new_struct.init_offsets(&allocs, tctx);
        let handle = allocs.structs.insert(new_struct);
        Self(handle as u32)
    }
    pub fn from_slice(tctx: &TypeContext, packed: bool, fields: &[ValTypeID]) -> Self {
        Self::new(tctx, packed, fields.iter().copied())
    }
}
