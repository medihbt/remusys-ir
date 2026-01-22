use crate::{
    SymbolStr,
    base::ISlabID,
    typing::{
        IValType, StructTypeID, TypeAllocs, TypeContext, TypeFormatter, TypeMismatchErr, TypingRes,
        ValTypeClass, ValTypeID,
    },
};
use std::{cell::Ref, io::Write};

#[derive(Debug, Clone)]
pub struct StructAliasObj {
    pub name: SymbolStr,
    pub aliasee: StructTypeID,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructAliasID(pub u32);

impl ISlabID for StructAliasID {
    type RefObject = StructAliasObj;

    fn from_handle(handle: u32) -> Self {
        StructAliasID(handle)
    }
    fn into_handle(self) -> u32 {
        self.0
    }
}

impl IValType for StructAliasID {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        let ValTypeID::StructAlias(a) = ty else {
            return Err(TypeMismatchErr::NotClass(ty, ValTypeClass::StructAlias));
        };
        Ok(a)
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

    fn serialize<T: Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        let name = &self.deref(&f.allocs.aliases).name;
        write!(f, "%{name}")
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        self.deref(&alloc.aliases)
            .aliasee
            .try_get_size_full(alloc, tctx)
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        self.deref(&alloc.aliases)
            .aliasee
            .try_get_align_full(alloc, tctx)
    }
}

impl StructAliasID {
    pub fn deref_ir(self, tctx: &TypeContext) -> Ref<'_, StructAliasObj> {
        let allocs = tctx.allocs.borrow();
        Ref::map(allocs, |allocs| self.deref(&allocs.aliases))
    }

    pub fn get_name(self, tctx: &TypeContext) -> Ref<'_, str> {
        let allocs = tctx.allocs.borrow();
        Ref::map(allocs, |allocs| &self.deref(&allocs.aliases).name[..])
    }
    pub fn get_aliasee(self, tctx: &TypeContext) -> StructTypeID {
        self.deref_ir(tctx).aliasee
    }

    pub fn new(tctx: &TypeContext, name: impl Into<SymbolStr>, aliasee: StructTypeID) -> Self {
        tctx.set_alias(name, aliasee)
    }
}
