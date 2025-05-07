use crate::base::slabref::SlabRef;

use super::{
    IValType,
    context::{TypeContext, binary_bits_to_bytes},
    types::{ArrayTypeRef, FloatTypeKind, FuncTypeRef, StructAliasRef, StructTypeRef},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValTypeID {
    Void,
    Ptr,
    Int(u8),
    Float(FloatTypeKind),
    Array(ArrayTypeRef),
    Struct(StructTypeRef),
    StructAlias(StructAliasRef),
    Func(FuncTypeRef),
}

impl ValTypeID {
    pub fn new_boolean() -> Self {
        Self::Int(1)
    }
    pub fn int_get_binary_bits(&self) -> u8 {
        match self {
            Self::Int(bin_bits) => *bin_bits,
            _ => panic!("type mismatch: requries Int but got {:?}", self),
        }
    }
    pub fn float_get_kind(&self) -> FloatTypeKind {
        match self {
            Self::Float(fp_kind) => *fp_kind,
            _ => panic!("type mismatch: requries Float but got {:?}", self),
        }
    }

    pub fn get_instance_size(&self, type_ctx: &TypeContext) -> Option<usize> {
        let inner_ref = type_ctx._inner.borrow();
        match self {
            ValTypeID::Void => None,
            ValTypeID::Ptr => Some(type_ctx.platform_policy.ptr_nbits / 8),
            ValTypeID::Int(binbits) => Some(binary_bits_to_bytes(*binbits as usize)),
            ValTypeID::Float(fp) => fp.get_instance_size(type_ctx),
            ValTypeID::Array(arr) => arr.load_data(type_ctx).get_instance_size(type_ctx),
            ValTypeID::Struct(st) => st
                .to_slabref_unwrap(&inner_ref._alloc_struct)
                .get_instance_size(type_ctx),
            ValTypeID::StructAlias(sa) => sa
                .to_slabref_unwrap(&inner_ref._alloc_struct_alias)
                .get_instance_size(type_ctx),
            ValTypeID::Func(_) => None,
        }
    }

    pub fn makes_instance(&self) -> bool {
        !matches!(self, Self::Void | Self::Func(_))
    }

    pub fn get_display_name(&self, type_ctx: &TypeContext) -> String {
        let inner_ref = type_ctx._inner.borrow();
        match self {
            ValTypeID::Void => "void".to_string(),
            ValTypeID::Ptr => "ptr".to_string(),
            ValTypeID::Int(binbits) => format!("i{}", binbits),
            ValTypeID::Float(fp) => fp.get_display_name(type_ctx),
            ValTypeID::Array(arr) => arr
                .to_slabref_unwrap(&inner_ref._alloc_array)
                .get_display_name(type_ctx),
            ValTypeID::Struct(st) => st
                .to_slabref_unwrap(&inner_ref._alloc_struct)
                .get_display_name(type_ctx),
            ValTypeID::StructAlias(sa) => sa
                .to_slabref_unwrap(&inner_ref._alloc_struct_alias)
                .get_display_name(type_ctx),
            ValTypeID::Func(func) => func
                .to_slabref_unwrap(&inner_ref._alloc_func)
                .get_display_name(type_ctx),
        }
    }
}
