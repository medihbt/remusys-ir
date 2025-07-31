use std::fmt::Debug;

use crate::base::SlabRef;

use super::{
    IValType,
    context::{TypeContext, binary_bits_to_bytes},
    types::{ArrayTypeRef, FloatTypeKind, FuncTypeRef, StructAliasRef, StructTypeRef},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

impl Debug for ValTypeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValTypeID::Void => write!(f, "void"),
            ValTypeID::Ptr => write!(f, "ptr"),
            ValTypeID::Int(binbits) => write!(f, "i{binbits}"),
            ValTypeID::Float(fp_kind) => write!(f, "f:{fp_kind:?}"),
            ValTypeID::Array(arr) => write!(f, "Array(ref:{})", arr.get_handle()),
            ValTypeID::Struct(st) => write!(f, "Struct(ref:{})", st.get_handle()),
            ValTypeID::StructAlias(sa) => write!(f, "StructAlias(ref:{})", sa.get_handle()),
            ValTypeID::Func(func) => write!(f, "Func(ref:{})", func.get_handle()),
        }
    }
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
            ValTypeID::Struct(st) => Some(st.get_instance_size(type_ctx)),
            ValTypeID::StructAlias(sa) => sa
                .to_data(&inner_ref._alloc_struct_alias)
                .get_instance_size(type_ctx),
            ValTypeID::Func(_) => None,
        }
    }

    pub fn get_instance_size_unwrap(&self, type_ctx: &TypeContext) -> usize {
        self.get_instance_size(type_ctx)
            .expect("ValTypeID must have a valid instance size")
    }

    pub fn get_instance_align(&self, type_ctx: &TypeContext) -> Option<usize> {
        match self {
            ValTypeID::Ptr => Some(type_ctx.platform_policy.ptr_nbits / 8),
            ValTypeID::Int(binbits) => {
                let bytes = binary_bits_to_bytes(*binbits as usize);
                Some(if bytes.is_power_of_two() { bytes } else { bytes.next_power_of_two() })
            }
            ValTypeID::Float(fp) => match fp {
                FloatTypeKind::Ieee32 => Some(4),
                FloatTypeKind::Ieee64 => Some(8),
            },
            ValTypeID::Array(arr) => arr.get_element_type(type_ctx).get_instance_align(type_ctx),
            ValTypeID::Struct(st) => Some(st.get_instance_align(type_ctx)),
            ValTypeID::StructAlias(sa) => {
                let sty = sa.get_aliasee(type_ctx);
                ValTypeID::Struct(sty).get_instance_align(type_ctx)
            }
            ValTypeID::Void | ValTypeID::Func(_) => None,
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
                .to_data(&inner_ref._alloc_array)
                .get_display_name(type_ctx),
            ValTypeID::Struct(st) => st
                .to_data(&inner_ref._alloc_struct)
                .get_display_name(type_ctx),
            ValTypeID::StructAlias(sa) => sa
                .to_data(&inner_ref._alloc_struct_alias)
                .get_display_name(type_ctx),
            ValTypeID::Func(func) => func
                .to_data(&inner_ref._alloc_func)
                .get_display_name(type_ctx),
        }
    }
}
