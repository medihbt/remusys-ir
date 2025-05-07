use std::ops::Mul;

use super::{IValType, context::TypeContext, id::ValTypeID};
use crate::{base::slabref::SlabRef, impl_slabref};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct ArrayTypeData {
    pub length: usize,
    pub elemty: ValTypeID,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub(super) fn load_data(&self, type_ctx: &TypeContext) -> ArrayTypeData {
        self.to_slabref_unwrap(&type_ctx._inner.borrow()._alloc_array)
            .clone()
    }
}

#[derive(Debug, Clone)]
pub struct StructTypeData {
    pub elemty: Box<[ValTypeID]>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructTypeRef(usize);
impl_slabref!(StructTypeRef, StructTypeData);
impl IValType for StructTypeData {
    fn get_instance_size(&self, type_ctx: &TypeContext) -> Option<usize> {
        self.elemty
            .iter()
            .map(|ty| ty.get_instance_size(type_ctx))
            .sum()
    }

    fn makes_instance(&self) -> bool {
        true
    }

    fn get_display_name(&self, type_ctx: &TypeContext) -> String {
        let mut ret = String::from("{");
        for t in &self.elemty {
            ret.push_str(t.get_display_name(type_ctx).as_str());
        }
        ret.push_str("}");
        ret
    }

    fn deep_eq(&self, rhs: &Self) -> bool {
        self.elemty == rhs.elemty
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
        reader(self.to_slabref_unwrap(&type_ctx._inner.borrow()._alloc_struct))
    }
    pub fn get_nelements(&self, type_ctx: &TypeContext) -> usize {
        self.read_data_ref(type_ctx, |s| s.elemty.len())
    }
    pub fn get_element_type(&self, type_ctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        self.read_data_ref(type_ctx, |s| s.elemty.get(index).cloned())
    }
}

#[derive(Debug, Clone)]
pub struct StructAliasData {
    pub name: String,
    pub aliasee: StructTypeRef,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructAliasRef(usize);
impl_slabref!(StructAliasRef, StructAliasData);
impl IValType for StructAliasData {
    fn get_instance_size(&self, type_ctx: &TypeContext) -> Option<usize> {
        let inner = type_ctx._inner.borrow();
        self.aliasee
            .to_slabref_unwrap(&inner._alloc_struct)
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
        self.to_slabref_unwrap(&type_ctx._inner.borrow()._alloc_struct_alias)
            .clone()
    }
    pub fn get_name(&self, type_ctx: &TypeContext) -> String {
        self.load_data(type_ctx).name
    }
}

#[derive(Debug, Clone)]
pub struct FuncTypeData {
    pub args: Box<[ValTypeID]>,
    pub ret_ty: ValTypeID,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuncTypeRef(usize);
impl_slabref!(FuncTypeRef, FuncTypeData);
impl IValType for FuncTypeData {
    fn get_instance_size(&self, _: &TypeContext) -> Option<usize> {
        None
    }

    fn makes_instance(&self) -> bool {
        false
    }

    fn get_display_name(&self, type_ctx: &TypeContext) -> String {
        let mut ret = String::from("fn<(");
        for (idx, arg) in self.args.iter().enumerate() {
            if idx > 0 {
                ret.push_str(", ");
            }
            let arg = arg.get_display_name(type_ctx);
            ret.push_str(arg.as_str());
        }
        ret
    }

    fn deep_eq(&self, rhs: &Self) -> bool {
        self.ret_ty == rhs.ret_ty && self.args == rhs.args
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
        reader(self.to_slabref_unwrap(&type_ctx._inner.borrow()._alloc_func))
    }

    pub fn get_return_type(&self, type_ctx: &TypeContext) -> ValTypeID {
        self.read_data_ref(type_ctx, |f| f.ret_ty.clone())
    }
    pub fn get_arg(&self, type_ctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        self.read_data_ref(type_ctx, |f| f.args.get(index).cloned())
    }
    pub fn get_nargs(&self, type_ctx: &TypeContext) -> usize {
        self.read_data_ref(type_ctx, |f| f.args.len())
    }
}
