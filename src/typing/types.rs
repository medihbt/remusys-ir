use std::{
    cell::{Cell, Ref},
    ops::Mul,
};

use super::{IValType, context::TypeContext, id::ValTypeID};
use crate::{base::SlabRef, impl_slabref};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatTypeKind {
    Ieee32,
    Ieee64,
}
impl IValType for FloatTypeKind {
    fn get_instance_size(&self, _: &TypeContext) -> Option<usize> {
        match self {
            Self::Ieee32 => Some(4),
            Self::Ieee64 => Some(8),
        }
    }
    fn makes_instance(&self) -> bool {
        true
    }
    fn deep_eq(&self, rhs: &Self) -> bool {
        self == rhs
    }

    fn get_display_name(&self, _: &TypeContext) -> String {
        match self {
            Self::Ieee32 => "float".to_string(),
            Self::Ieee64 => "double".to_string(),
        }
    }

    fn gc_trace(&self, _: impl Fn(ValTypeID)) {}
}
impl FloatTypeKind {
    pub fn get_binary_bits(&self) -> u8 {
        match self {
            Self::Ieee32 => 32,
            Self::Ieee64 => 64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArrayTypeData {
    pub length: usize,
    pub elemty: ValTypeID,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayTypeRef(usize);
impl_slabref!(ArrayTypeRef, ArrayTypeData);
impl IValType for ArrayTypeData {
    fn get_instance_size(&self, type_ctx: &TypeContext) -> Option<usize> {
        Some(
            self.elemty
                .get_instance_size(type_ctx)
                .expect("Type should make instance")
                .mul(self.length),
        )
    }

    fn makes_instance(&self) -> bool {
        true
    }

    fn get_display_name(&self, type_ctx: &TypeContext) -> String {
        format!(
            "[{} x {}]",
            self.length,
            self.elemty.get_display_name(type_ctx)
        )
    }

    fn deep_eq(&self, rhs: &Self) -> bool {
        self.length == rhs.length && self.elemty == rhs.elemty
    }

    fn gc_trace(&self, gather_func: impl Fn(ValTypeID)) {
        gather_func(self.elemty);
    }
}

impl ArrayTypeRef {
    pub fn get_nelements(&self, type_ctx: &TypeContext) -> usize {
        self.load_data(type_ctx).length
    }
    pub fn get_element_type(&self, type_ctx: &TypeContext) -> ValTypeID {
        self.load_data(type_ctx).elemty
    }
    pub fn get_elem_size(&self, type_ctx: &TypeContext) -> usize {
        self.get_element_type(type_ctx)
            .get_instance_size_unwrap(type_ctx)
    }
    pub fn get_elem_aligned_size(&self, type_ctx: &TypeContext) -> usize {
        let elem_size = self.get_elem_size(type_ctx);
        let elem_align = self
            .get_element_type(type_ctx)
            .try_get_instance_align(type_ctx)
            .unwrap();
        elem_size.next_multiple_of(elem_align)
    }
    pub fn get_instance_size(&self, type_ctx: &TypeContext) -> usize {
        self.load_data(type_ctx)
            .get_instance_size(type_ctx)
            .expect("Array type should have a valid instance size")
    }
    pub(super) fn load_data(&self, type_ctx: &TypeContext) -> ArrayTypeData {
        self.to_data(&type_ctx._inner.borrow()._alloc_array).clone()
    }
}

/// 结构体类型数据.
#[derive(Debug, Clone)]
pub struct StructTypeData {
    pub elemty: Box<[ValTypeID]>,
    pub packed: bool,
    /// 第 i-1 个数字表示第 i 元素的偏移量.
    /// 最后一个数字表示结构体的总大小.
    _size_cache: Box<[Cell<usize>]>,
    _size_top: Cell<usize>,
    _align_cache: Cell<usize>,
}

impl StructTypeData {
    pub fn new(elems: &[ValTypeID], packed: bool) -> Self {
        for elem in elems {
            assert!(
                elem.makes_instance(),
                "Struct type can only hold types that make instance but got {elem:?}",
            );
        }

        let sizes = vec![Cell::new(0); elems.len()].into_boxed_slice();
        Self {
            elemty: elems.into(),
            packed,
            _size_cache: sizes,
            _size_top: Cell::new(0),
            _align_cache: Cell::new(0),
        }
    }

    /// 获取索引为 index 的元素在结构体中的偏移量, 并更新大小缓存.
    pub fn get_offset(&self, index: usize, type_ctx: &TypeContext) -> Option<usize> {
        // 处理特殊情况: index 一眼就能看出是否出界
        if index == 0 {
            return Some(0);
        } else if index <= self._size_top.get() {
            return Some(self._size_cache[index - 1].get());
        } else if index >= self.elemty.len() {
            return None;
        }

        // index 没出界, 但缓存也还没加载到这个 index 上. 因此
        // 需要从 _size_top 开始, 逐个计算每个元素的偏移量.
        let size = self.update_size_cache(index, type_ctx);
        Some(size)
    }

    /// 计算并缓存到 index 的偏移量, 并返回该偏移量.
    /// 当 `index == self.elemty.len()` 时, 返回结构体的总大小.
    /// 注意, 该方法会更新 `_size_top` 和 `_size_cache`.
    fn update_size_cache(&self, index: usize, type_ctx: &TypeContext) -> usize {
        let old_top = self._size_top.get();
        let mut size = if old_top == 0 { 0 } else { self._size_cache[old_top - 1].get() };
        for id in old_top..index {
            let elem_size = self.elemty[id].get_instance_size_unwrap(type_ctx);
            if !self.packed {
                let elem_align = self.elemty[id].try_get_instance_align(type_ctx).unwrap();
                debug_assert!(
                    elem_align.is_power_of_two(),
                    "Element alignment must be a power of two"
                );
                size = size.next_multiple_of(elem_align);
            }
            size += elem_size;
            self._size_cache[id].set(size);
        }
        self._size_top.set(index);
        size
    }

    pub fn get_instance_align(&self, type_ctx: &TypeContext) -> usize {
        if self._align_cache.get() != 0 {
            return self._align_cache.get();
        }
        if self.packed {
            self._align_cache.set(1);
            return 1; // 最小对齐
        }
        let align = self
            .elemty
            .iter()
            .map(|ty| ty.try_get_instance_align(type_ctx).unwrap())
            .max()
            .unwrap_or(1);
        self._align_cache.set(align);
        align
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructTypeRef(usize);
impl_slabref!(StructTypeRef, StructTypeData);
impl IValType for StructTypeData {
    fn get_instance_size(&self, type_ctx: &TypeContext) -> Option<usize> {
        let nelems = self.elemty.len();
        let size = self.update_size_cache(nelems, type_ctx);
        Some(size)
    }

    fn makes_instance(&self) -> bool {
        true
    }

    fn get_display_name(&self, type_ctx: &TypeContext) -> String {
        let mut ret = String::from("{");
        for (index, t) in self.elemty.iter().enumerate() {
            if index > 0 {
                ret.push_str(", ");
            }
            ret.push_str(t.get_display_name(type_ctx).as_str());
        }
        ret.push_str("}");
        ret
    }

    fn deep_eq(&self, rhs: &Self) -> bool {
        self.elemty == rhs.elemty && self.packed == rhs.packed
    }

    fn gc_trace(&self, gather_func: impl Fn(ValTypeID)) {
        for i in &self.elemty {
            gather_func(i.clone())
        }
    }
}

impl StructTypeRef {
    pub(super) fn read_data_ref<R>(
        &self,
        type_ctx: &TypeContext,
        reader: impl FnOnce(&StructTypeData) -> R,
    ) -> R {
        reader(self.to_data(&type_ctx._inner.borrow()._alloc_struct))
    }
    pub fn get_nelements(&self, type_ctx: &TypeContext) -> usize {
        self.read_data_ref(type_ctx, |s| s.elemty.len())
    }
    pub fn get_element_type(&self, type_ctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        self.read_data_ref(type_ctx, |s| s.elemty.get(index).cloned())
    }

    pub fn get_offset(&self, type_ctx: &TypeContext, index: usize) -> Option<usize> {
        self.read_data_ref(type_ctx, |s| s.get_offset(index, type_ctx))
    }
    pub fn offset_unwrap(&self, type_ctx: &TypeContext, index: usize) -> usize {
        self.get_offset(type_ctx, index)
            .expect("Struct type can only hold types that make instance")
    }

    pub fn get_instance_size(&self, type_ctx: &TypeContext) -> usize {
        self.read_data_ref(type_ctx, |s| s.get_instance_size(type_ctx).unwrap())
    }
    pub fn get_instance_align(&self, type_ctx: &TypeContext) -> usize {
        self.read_data_ref(type_ctx, |s| s.get_instance_align(type_ctx))
    }

    pub fn is_packed(&self, type_ctx: &TypeContext) -> bool {
        self.read_data_ref(type_ctx, |s| s.packed)
    }

    pub fn iter_offsets<'a>(self, type_ctx: &'a TypeContext) -> StructOffsetIter<'a> {
        StructOffsetIter::new(self.clone(), type_ctx)
    }
}

pub struct StructOffsetIter<'a> {
    type_ref: StructTypeRef,
    type_ctx: &'a TypeContext,
    current_index: usize,
}

impl<'a> StructOffsetIter<'a> {
    pub fn new(type_ref: StructTypeRef, type_ctx: &'a TypeContext) -> Self {
        Self { type_ref, type_ctx, current_index: 0 }
    }

    pub fn get(&self) -> Option<usize> {
        self.type_ref.get_offset(self.type_ctx, self.current_index)
    }
}

impl<'a> Iterator for StructOffsetIter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.type_ref.get_nelements(self.type_ctx) {
            let offset = self.get();
            self.current_index += 1;
            offset
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructAliasData {
    pub name: String,
    pub aliasee: StructTypeRef,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructAliasRef(usize);
impl_slabref!(StructAliasRef, StructAliasData);
impl IValType for StructAliasData {
    fn get_instance_size(&self, type_ctx: &TypeContext) -> Option<usize> {
        let inner = type_ctx._inner.borrow();
        self.aliasee
            .to_data(&inner._alloc_struct)
            .get_instance_size(type_ctx)
    }

    fn makes_instance(&self) -> bool {
        true
    }
    fn get_display_name(&self, _: &TypeContext) -> String {
        format!("%{}", self.name)
    }

    fn deep_eq(&self, rhs: &Self) -> bool {
        self.name == rhs.name && self.aliasee == rhs.aliasee
    }

    fn gc_trace(&self, gather_func: impl Fn(ValTypeID)) {
        gather_func(ValTypeID::Struct(self.aliasee.clone()))
    }
}
impl StructAliasRef {
    pub fn load_data(&self, type_ctx: &TypeContext) -> StructAliasData {
        self.to_data(&type_ctx._inner.borrow()._alloc_struct_alias)
            .clone()
    }
    pub fn get_name(&self, type_ctx: &TypeContext) -> String {
        self.load_data(type_ctx).name
    }
    pub fn get_aliasee(&self, type_ctx: &TypeContext) -> StructTypeRef {
        self.to_data(&type_ctx._inner.borrow()._alloc_struct_alias)
            .aliasee
    }
}

#[derive(Debug, Clone)]
pub struct FuncTypeData {
    pub args: Box<[ValTypeID]>,
    pub ret_ty: ValTypeID,
    pub is_vararg: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncTypeRef(usize);
impl_slabref!(FuncTypeRef, FuncTypeData);
impl IValType for FuncTypeData {
    fn get_instance_size(&self, _: &TypeContext) -> Option<usize> {
        None
    }

    fn makes_instance(&self) -> bool {
        false
    }

    /// Syntax: `fn<(<arg1>, <arg2>, ...): <return type>>`
    fn get_display_name(&self, type_ctx: &TypeContext) -> String {
        let mut ret = String::from("fn<(");

        for (idx, arg) in self.args.iter().enumerate() {
            if idx > 0 {
                ret.push_str(", ");
            }
            let arg = arg.get_display_name(type_ctx);
            ret.push_str(arg.as_str());
        }

        if self.is_vararg {
            if !self.args.is_empty() {
                ret.push_str(", ");
            }
            ret.push_str("...");
        }
        ret.push_str("):");
        ret.push_str(self.ret_ty.get_display_name(type_ctx).as_str());
        ret.push('>');
        ret
    }

    fn deep_eq(&self, rhs: &Self) -> bool {
        self.ret_ty == rhs.ret_ty && self.args == rhs.args && self.is_vararg == rhs.is_vararg
    }

    fn gc_trace(&self, gather_func: impl Fn(ValTypeID)) {
        gather_func(self.ret_ty.clone());
        for i in &self.args {
            gather_func(i.clone())
        }
    }
}

impl FuncTypeRef {
    pub(super) fn read_data_ref<R>(
        &self,
        type_ctx: &TypeContext,
        reader: impl FnOnce(&FuncTypeData) -> R,
    ) -> R {
        reader(self.to_data(&type_ctx._inner.borrow()._alloc_func))
    }

    pub fn get_return_type(&self, type_ctx: &TypeContext) -> ValTypeID {
        self.read_data_ref(type_ctx, |f| f.ret_ty.clone())
    }
    pub fn get_arg(&self, type_ctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        self.read_data_ref(type_ctx, |f| f.args.get(index).cloned())
    }

    // Returns the number of fixed arguments.
    // If the function is variadic, this does not include the variable arguments.
    pub fn get_nargs(&self, type_ctx: &TypeContext) -> usize {
        self.read_data_ref(type_ctx, |f| f.args.len())
    }

    pub fn get_args<'a>(&self, type_ctx: &'a TypeContext) -> Ref<'a, [ValTypeID]> {
        let alloc = type_ctx._inner.borrow();
        Ref::map(alloc, |a| self.to_data(&a._alloc_func).args.as_ref())
    }
    pub fn is_vararg(&self, type_ctx: &TypeContext) -> bool {
        self.read_data_ref(type_ctx, |f| f.is_vararg)
    }
}
