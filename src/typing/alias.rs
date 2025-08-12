use crate::{
    base::SlabRef,
    typing::{
        IValType, StructTypeRef, TypeAllocs, TypeContext, TypeFormatter, TypeMismatchError,
        TypingRes, ValTypeClass, ValTypeID,
    },
};
use std::{cell::Ref, io::Write};

#[derive(Debug, Clone)]
pub struct StructAliasData {
    pub name: String,
    pub aliasee: StructTypeRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructAliasRef(pub usize);

impl SlabRef for StructAliasRef {
    type RefObject = StructAliasData;
    fn from_handle(handle: usize) -> Self {
        Self(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl IValType for StructAliasRef {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        match ty {
            ValTypeID::StructAlias(s) => Ok(s),
            _ => Err(TypeMismatchError::NotClass(ty, ValTypeClass::StructAlias)),
        }
    }

    fn into_ir(self) -> ValTypeID {
        ValTypeID::StructAlias(self)
    }

    fn makes_instance(self) -> bool {
        true
    }

    fn class_id(self) -> ValTypeClass {
        ValTypeClass::StructAlias
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        self.to_data(&alloc.aliases)
            .aliasee
            .try_get_align_full(alloc, tctx)
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        self.to_data(&alloc.aliases)
            .aliasee
            .try_get_align_full(alloc, tctx)
    }

    fn serialize<T: Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        let name = &self.to_data(&f.allocs.aliases).name;
        write!(f, "%{name}")
    }
}

impl StructAliasRef {
    pub fn get_aliasee(self, tctx: &TypeContext) -> StructTypeRef {
        self.to_data(&tctx.allocs.borrow().aliases).aliasee
    }

    pub fn get_name<'a>(self, tctx: &'a TypeContext) -> Ref<'a, str> {
        let allocs = tctx.allocs.borrow();
        Ref::map(allocs, |allocs| self.to_data(&allocs.aliases).name.as_str())
    }
}
