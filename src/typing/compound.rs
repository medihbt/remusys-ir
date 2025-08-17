use std::fmt::Debug;

use crate::typing::{
    ArrayTypeRef, FPKind, IValType, IntType, PtrType, StructAliasRef, StructTypeRef, TypeAllocs,
    TypeContext, TypeFormatter, TypeMismatchError, TypingRes, ValTypeClass, ValTypeID,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimType {
    Ptr,
    Int(u8),
    Float(FPKind),
}

impl Debug for PrimType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimType::Ptr => write!(f, "ptr"),
            PrimType::Int(bits) => write!(f, "i{bits}"),
            PrimType::Float(FPKind::Ieee32) => write!(f, "float"),
            PrimType::Float(FPKind::Ieee64) => write!(f, "double"),
        }
    }
}

impl IValType for PrimType {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        match ty {
            ValTypeID::Ptr => Ok(Self::Ptr),
            ValTypeID::Int(bits) => Ok(Self::Int(bits)),
            ValTypeID::Float(fpkind) => Ok(Self::Float(fpkind)),
            _ => Err(TypeMismatchError::NotPrimitive(ty)),
        }
    }

    fn into_ir(self) -> ValTypeID {
        match self {
            Self::Ptr => ValTypeID::Ptr,
            Self::Int(bits) => ValTypeID::Int(bits),
            Self::Float(fpkind) => ValTypeID::Float(fpkind),
        }
    }

    fn makes_instance(self) -> bool {
        true
    }

    fn class_id(self) -> ValTypeClass {
        match self {
            Self::Ptr => ValTypeClass::Ptr,
            Self::Int(_) => ValTypeClass::Int,
            Self::Float(_) => ValTypeClass::Float,
        }
    }

    fn serialize<T: std::io::Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        match self {
            Self::Ptr => f.write_str("ptr"),
            Self::Int(bits) => IntType(bits).serialize(f),
            Self::Float(fpkind) => fpkind.serialize(f),
        }
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        match self {
            PrimType::Ptr => PtrType.try_get_size_full(alloc, tctx),
            PrimType::Int(bits) => IntType(bits).try_get_size_full(alloc, tctx),
            PrimType::Float(fpkind) => fpkind.try_get_size_full(alloc, tctx),
        }
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        match self {
            PrimType::Ptr => PtrType.try_get_align_full(alloc, tctx),
            PrimType::Int(bits) => IntType(bits).try_get_align_full(alloc, tctx),
            PrimType::Float(fpkind) => fpkind.try_get_align_full(alloc, tctx),
        }
    }
}

impl From<IntType> for PrimType {
    fn from(value: IntType) -> Self {
        PrimType::Int(value.0)
    }
}

impl From<PtrType> for PrimType {
    fn from(_: PtrType) -> Self {
        PrimType::Ptr
    }
}

impl From<FPKind> for PrimType {
    fn from(value: FPKind) -> Self {
        PrimType::Float(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AggrType {
    Array(ArrayTypeRef),
    Struct(StructTypeRef),
    Alias(StructAliasRef),
}

impl From<ArrayTypeRef> for AggrType {
    fn from(value: ArrayTypeRef) -> Self {
        AggrType::Array(value)
    }
}

impl From<StructTypeRef> for AggrType {
    fn from(value: StructTypeRef) -> Self {
        AggrType::Struct(value)
    }
}

impl From<StructAliasRef> for AggrType {
    fn from(value: StructAliasRef) -> Self {
        AggrType::Alias(value)
    }
}

impl IValType for AggrType {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        match ty {
            ValTypeID::Array(arr) => Ok(Self::Array(arr)),
            ValTypeID::Struct(st) => Ok(Self::Struct(st)),
            ValTypeID::StructAlias(alias) => Ok(Self::Alias(alias)),
            _ => Err(TypeMismatchError::NotAggregate(ty)),
        }
    }

    fn into_ir(self) -> ValTypeID {
        match self {
            Self::Array(arr) => ValTypeID::Array(arr),
            Self::Struct(st) => ValTypeID::Struct(st),
            Self::Alias(alias) => ValTypeID::StructAlias(alias),
        }
    }

    fn makes_instance(self) -> bool {
        true
    }

    fn class_id(self) -> ValTypeClass {
        match self {
            Self::Array(arr) => arr.class_id(),
            Self::Struct(st) => st.class_id(),
            Self::Alias(alias) => alias.class_id(),
        }
    }

    fn serialize<T: std::io::Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        match self {
            Self::Array(arr) => arr.serialize(f),
            Self::Struct(st) => st.serialize(f),
            Self::Alias(alias) => alias.serialize(f),
        }
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        match self {
            Self::Array(arr) => arr.try_get_size_full(alloc, tctx),
            Self::Struct(st) => st.try_get_size_full(alloc, tctx),
            Self::Alias(alias) => alias.try_get_size_full(alloc, tctx),
        }
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        match self {
            Self::Array(arr) => arr.try_get_align_full(alloc, tctx),
            Self::Struct(st) => st.try_get_align_full(alloc, tctx),
            Self::Alias(alias) => alias.try_get_align_full(alloc, tctx),
        }
    }
}

impl AggrType {
    pub fn try_get_field(self, tctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        match self {
            AggrType::Array(arr) => Some(arr.get_element_type(tctx)),
            AggrType::Struct(st) => st.try_get_field(tctx, index),
            AggrType::Alias(alias) => alias.get_aliasee(tctx).try_get_field(tctx, index),
        }
    }
    pub fn get_field(self, tctx: &TypeContext, index: usize) -> ValTypeID {
        self.try_get_field(tctx, index)
            .expect("Failed to get element type from aggregate type")
    }

    pub fn try_get_offset(self, tctx: &TypeContext, index: usize) -> Option<usize> {
        match self {
            AggrType::Array(arr) => Some(arr.get_offset(tctx, index)),
            AggrType::Struct(st) => st.try_get_offset(tctx, index),
            AggrType::Alias(alias) => alias.get_aliasee(tctx).try_get_offset(tctx, index),
        }
    }
    pub fn get_offset(self, tctx: &TypeContext, index: usize) -> usize {
        self.try_get_offset(tctx, index)
            .expect("Failed to get offset from aggregate type")
    }

    pub fn nfields(self, tctx: &TypeContext) -> usize {
        match self {
            AggrType::Array(arr) => arr.get_num_elements(tctx),
            AggrType::Struct(st) => st.get_nfields(tctx),
            AggrType::Alias(alias) => alias.get_aliasee(tctx).get_nfields(tctx),
        }
    }
}

pub struct AggrTypeIter<'a> {
    aggr_type: AggrType,
    type_ctx: &'a TypeContext,
    index: usize,
    nfields: usize,
}

impl<'a> AggrTypeIter<'a> {
    pub fn new(aggr_type: AggrType, type_ctx: &'a TypeContext) -> Self {
        let nfields = aggr_type.nfields(type_ctx);
        Self { aggr_type, type_ctx, index: 0, nfields }
    }
}

impl<'a> Iterator for AggrTypeIter<'a> {
    type Item = (usize, ValTypeID);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.nfields {
            return None;
        }
        let field_type = self.aggr_type.get_field(self.type_ctx, self.index);
        let item = (self.index, field_type);
        self.index += 1;
        Some(item)
    }
}

#[cfg(test)]
mod testing {
    use crate::typing::*;

    #[test]
    fn calc_aggrtype_size() {
        let tctx = TypeContext::new(ArchInfo::new_host());
        let i32ty = ValTypeID::Int(32);
        let i64ty = ValTypeID::Int(64);
        let i8ty = ValTypeID::Int(8);
        let sty = StructTypeRef::new(&tctx, false, [i32ty, i32ty, i8ty, i64ty]);
        let aty = ArrayTypeRef::new(&tctx, i32ty, 10);

        assert_eq!(sty.get_size(&tctx), 24);
        assert_eq!(sty.get_align(&tctx), 8);

        assert_eq!(aty.get_size(&tctx), 40);
        assert_eq!(aty.get_align(&tctx), 4);

        let aggr_ty = AggrType::Array(aty);
        assert_eq!(aggr_ty.get_size(&tctx), 40);
        assert_eq!(aggr_ty.get_align(&tctx), 4);
    }
}
