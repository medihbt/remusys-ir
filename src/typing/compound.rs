use std::{fmt::Debug, io::Write};

use crate::{
    base::INullableValue,
    typing::{
        ArrayTypeID, FPKind, FixVecType, IValType, IntType, PtrType, StructAliasID, StructTypeID,
        TypeAllocs, TypeContext, TypeFormatter, TypeMismatchErr, TypingRes, ValTypeClass,
        ValTypeID,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ScalarType {
    Ptr,
    Int(u8),
    Float(FPKind),
}

impl Debug for ScalarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarType::Ptr => write!(f, "Ptr"),
            ScalarType::Int(bits) => write!(f, "i{bits}"),
            ScalarType::Float(FPKind::Ieee32) => write!(f, "float"),
            ScalarType::Float(FPKind::Ieee64) => write!(f, "double"),
        }
    }
}

impl IValType for ScalarType {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        match ty {
            ValTypeID::Ptr => Ok(ScalarType::Ptr),
            ValTypeID::Int(bits) => Ok(ScalarType::Int(bits)),
            ValTypeID::Float(fp_kind) => Ok(ScalarType::Float(fp_kind)),
            _ => Err(TypeMismatchErr::NotClass(ty, ValTypeClass::Compound)),
        }
    }

    fn into_ir(self) -> ValTypeID {
        match self {
            ScalarType::Ptr => ValTypeID::Ptr,
            ScalarType::Int(bits) => ValTypeID::Int(bits),
            ScalarType::Float(fp_kind) => ValTypeID::Float(fp_kind),
        }
    }

    fn makes_instance(self) -> bool {
        true
    }

    fn class_id(self) -> ValTypeClass {
        match self {
            ScalarType::Ptr => ValTypeClass::Ptr,
            ScalarType::Int(_) => ValTypeClass::Int,
            ScalarType::Float(_) => ValTypeClass::Float,
        }
    }

    fn format_ir<T: Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        match self {
            ScalarType::Ptr => f.write_str("ptr"),
            ScalarType::Int(bits) => write!(f, "i{}", bits),
            ScalarType::Float(FPKind::Ieee32) => f.write_str("float"),
            ScalarType::Float(FPKind::Ieee64) => f.write_str("double"),
        }
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        match self {
            ScalarType::Ptr => PtrType.try_get_size_full(alloc, tctx),
            ScalarType::Int(bits) => IntType(bits).try_get_size_full(alloc, tctx),
            ScalarType::Float(fpkind) => fpkind.try_get_size_full(alloc, tctx),
        }
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        match self {
            ScalarType::Ptr => PtrType.try_get_align_full(alloc, tctx),
            ScalarType::Int(bits) => IntType(bits).try_get_align_full(alloc, tctx),
            ScalarType::Float(fpkind) => fpkind.try_get_align_full(alloc, tctx),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AggrType {
    Array(ArrayTypeID),
    Struct(StructTypeID),
    Alias(StructAliasID),
    FixVec(FixVecType),
}
impl Debug for AggrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggrType::Array(id) => id.fmt(f),
            AggrType::Struct(id) => id.fmt(f),
            AggrType::Alias(id) => id.fmt(f),
            AggrType::FixVec(fv) => fv.fmt(f),
        }
    }
}

impl INullableValue for AggrType {
    fn new_null() -> Self {
        AggrType::Array(ArrayTypeID::new_null())
    }

    fn is_null(&self) -> bool {
        match self {
            AggrType::Array(id) => id.is_null(),
            AggrType::Struct(id) => id.is_null(),
            AggrType::Alias(id) => id.is_null(),
            AggrType::FixVec(_) => false,
        }
    }
}

impl From<ArrayTypeID> for AggrType {
    fn from(id: ArrayTypeID) -> Self {
        AggrType::Array(id)
    }
}
impl From<StructTypeID> for AggrType {
    fn from(id: StructTypeID) -> Self {
        AggrType::Struct(id)
    }
}
impl From<StructAliasID> for AggrType {
    fn from(id: StructAliasID) -> Self {
        AggrType::Alias(id)
    }
}
impl From<FixVecType> for AggrType {
    fn from(id: FixVecType) -> Self {
        AggrType::FixVec(id)
    }
}
impl IValType for AggrType {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        match ty {
            ValTypeID::Array(id) => Ok(AggrType::Array(id)),
            ValTypeID::Struct(id) => Ok(AggrType::Struct(id)),
            ValTypeID::StructAlias(id) => Ok(AggrType::Alias(id)),
            ValTypeID::FixVec(fv) => Ok(AggrType::FixVec(fv)),
            _ => Err(TypeMismatchErr::NotClass(ty, ValTypeClass::Compound)),
        }
    }

    fn into_ir(self) -> ValTypeID {
        match self {
            AggrType::Array(id) => ValTypeID::Array(id),
            AggrType::Struct(id) => ValTypeID::Struct(id),
            AggrType::Alias(id) => ValTypeID::StructAlias(id),
            AggrType::FixVec(fv) => ValTypeID::FixVec(fv),
        }
    }

    fn makes_instance(self) -> bool {
        true
    }

    fn class_id(self) -> ValTypeClass {
        match self {
            AggrType::Array(_) => ValTypeClass::Array,
            AggrType::Struct(_) => ValTypeClass::Struct,
            AggrType::Alias(_) => ValTypeClass::StructAlias,
            AggrType::FixVec(_) => ValTypeClass::FixVec,
        }
    }

    fn format_ir<T: Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        match self {
            AggrType::Array(id) => id.format_ir(f),
            AggrType::Struct(id) => id.format_ir(f),
            AggrType::Alias(id) => id.format_ir(f),
            AggrType::FixVec(fv) => fv.format_ir(f),
        }
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        match self {
            AggrType::Array(id) => id.try_get_size_full(alloc, tctx),
            AggrType::Struct(id) => id.try_get_size_full(alloc, tctx),
            AggrType::Alias(id) => id.try_get_size_full(alloc, tctx),
            AggrType::FixVec(fv) => fv.try_get_size_full(alloc, tctx),
        }
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        match self {
            AggrType::Array(id) => id.try_get_align_full(alloc, tctx),
            AggrType::Struct(id) => id.try_get_align_full(alloc, tctx),
            AggrType::Alias(id) => id.try_get_align_full(alloc, tctx),
            AggrType::FixVec(fv) => fv.try_get_align_full(alloc, tctx),
        }
    }
}

impl AggrType {
    pub fn try_get_field(self, tctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        use AggrType::*;
        match self {
            Array(id) => Some(id.get_element_type(tctx)),
            Struct(id) => id.get_fields(tctx).get(index).cloned(),
            Alias(id) => id.get_aliasee(tctx).get_fields(tctx).get(index).cloned(),
            FixVec(id) => Some(id.get_elem().into_ir()),
        }
    }

    pub fn get_field(self, tctx: &TypeContext, index: usize) -> ValTypeID {
        self.try_get_field(tctx, index)
            .expect("Index out of bounds")
    }

    pub fn try_get_offset(self, tctx: &TypeContext, index: usize) -> Option<usize> {
        use AggrType::*;
        match self {
            Array(id) => Some(id.get_offset(tctx, index)),
            Struct(id) => id.try_get_offset(tctx, index),
            Alias(id) => id.get_aliasee(tctx).try_get_offset(tctx, index),
            FixVec(id) => id.try_get_offset(index, tctx),
        }
    }
    pub fn get_offset(self, tctx: &TypeContext, index: usize) -> usize {
        self.try_get_offset(tctx, index)
            .expect("Index out of bounds or size overflow")
    }

    pub fn nfields(self, tctx: &TypeContext) -> usize {
        use AggrType::*;
        match self {
            Array(id) => id.get_num_elements(tctx),
            Struct(id) => id.get_fields(tctx).len(),
            Alias(id) => id.get_aliasee(tctx).get_fields(tctx).len(),
            FixVec(id) => id.get_len(),
        }
    }
}
