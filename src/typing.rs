mod alias;
mod array;
mod compound;
mod context;
mod fmt;
mod func;
mod prim;
mod serde;
mod structty;
mod vec;

use crate::{SymbolStr, base::ISlabID};
use smol_str::SmolStrBuilder;
use std::fmt::Write;

pub use self::{
    alias::{StructAliasID, StructAliasObj},
    array::{ArrayTypeID, ArrayTypeObj},
    compound::{AggrType, ScalarType},
    context::{ArchInfo, TypeAllocs, TypeContext},
    fmt::TypeFormatter,
    func::{FuncTypeID, FuncTypeObj},
    prim::{FPKind, IntType, PtrType},
    structty::{StructTypeID, StructTypeObj},
    vec::FixVecType,
};

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum TypeMismatchErr {
    #[error("Type {0:?} is not equal to type {1:?}")]
    IDNotEqual(ValTypeID, ValTypeID),
    #[error("Type {0:?} layout is not equal to type {1:?}")]
    LayoutNotEqual(ValTypeID, ValTypeID),
    #[error("Type {0:?} kind does not match expected kind {1:?}")]
    KindNotMatch(ValTypeID, ValTypeID),
    #[error("Type {0:?} is not of class {1:?}")]
    NotClass(ValTypeID, ValTypeClass),

    #[error("Type {0:?} is not an aggregate type")]
    NotAggregate(ValTypeID),
    #[error("Type {0:?} is not a primitive type")]
    NotPrimitive(ValTypeID),
}
pub type TypingRes<T = ()> = Result<T, TypeMismatchErr>;

pub trait IValType: Copy {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self>;

    fn from_ir(ty: ValTypeID) -> Self {
        match Self::try_from_ir(ty) {
            Ok(val) => val,
            Err(err) => {
                let thisname = std::any::type_name::<Self>();
                panic!("Failed to convert {ty:?} to {thisname:?}: {err:?}")
            }
        }
    }

    fn into_ir(self) -> ValTypeID;

    fn makes_instance(self) -> bool;

    /// 这个类型的 class ID
    fn class_id(self) -> ValTypeClass;

    /// 序列化
    fn format_ir<T: Write>(self, f: &TypeFormatter<T>) -> std::fmt::Result;
    fn get_display_name(self, tctx: &TypeContext) -> SymbolStr {
        let mut buffer = SmolStrBuilder::new();
        let formatter = TypeFormatter::new(&mut buffer, tctx);
        self.format_ir(&formatter)
            .expect("Serialization to String should not fail");
        drop(formatter);
        buffer.finish()
    }
    fn println(&self, tctx: &TypeContext) {
        let name = self.get_display_name(tctx);
        println!("{name}");
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize>;
    fn try_get_size(self, tctx: &TypeContext) -> Option<usize> {
        self.try_get_size_full(&tctx.allocs.borrow(), tctx)
    }
    fn get_size(self, tctx: &TypeContext) -> usize {
        self.try_get_size(tctx).expect("Failed to get size of type")
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize>;
    fn try_get_align(self, tctx: &TypeContext) -> Option<usize> {
        self.try_get_align_full(&tctx.allocs.borrow(), tctx)
    }
    fn get_align(self, tctx: &TypeContext) -> usize {
        self.try_get_align(tctx)
            .expect("Failed to get align of type")
    }
    fn get_align_log2(self, tctx: &TypeContext) -> u8 {
        self.get_align(tctx).ilog2() as u8
    }

    fn try_get_aligned_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        let size = self.try_get_size_full(alloc, tctx)?;
        let align = self.try_get_align_full(alloc, tctx)?;
        Some(size.next_multiple_of(align))
    }
    fn try_get_aligned_size(self, tctx: &TypeContext) -> Option<usize> {
        self.try_get_aligned_size_full(&tctx.allocs.borrow(), tctx)
    }
    fn get_aligned_size(self, tctx: &TypeContext) -> usize {
        self.try_get_aligned_size(tctx)
            .expect("Failed to get aligned size of type")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValTypeClass {
    Void,
    Ptr,
    Int,
    Float,
    FixVec,
    Array,
    Struct,
    StructAlias,
    Func,
    Compound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ValTypeID {
    /// Uninhabited type
    #[default]
    Void,

    /// Opaque pointer type (without pointee type)
    Ptr,

    /// Binary Bits: 1..128
    Int(u8),

    /// Floating Type
    Float(FPKind),

    /// Fixed Vector Type
    FixVec(FixVecType),

    /// Array Type
    Array(ArrayTypeID),

    /// Unnamed Structure Type
    Struct(StructTypeID),

    /// Struct Alias Type
    StructAlias(StructAliasID),

    /// Function type
    Func(FuncTypeID),
}

impl IValType for ValTypeID {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        Ok(ty)
    }

    fn into_ir(self) -> ValTypeID {
        self
    }

    fn makes_instance(self) -> bool {
        !matches!(self, ValTypeID::Void | ValTypeID::Func(_))
    }

    fn class_id(self) -> ValTypeClass {
        match self {
            ValTypeID::Void => ValTypeClass::Void,
            ValTypeID::Ptr => ValTypeClass::Ptr,
            ValTypeID::Int(_) => ValTypeClass::Int,
            ValTypeID::Float(_) => ValTypeClass::Float,
            ValTypeID::FixVec(_) => ValTypeClass::FixVec,
            ValTypeID::Array(_) => ValTypeClass::Array,
            ValTypeID::Struct(_) => ValTypeClass::Struct,
            ValTypeID::StructAlias(_) => ValTypeClass::StructAlias,
            ValTypeID::Func(_) => ValTypeClass::Func,
        }
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        match self {
            ValTypeID::Void | ValTypeID::Func(_) => None,
            ValTypeID::Ptr => PtrType.try_get_size_full(alloc, tctx),
            ValTypeID::Int(bits) => IntType(bits).try_get_size_full(alloc, tctx),
            ValTypeID::Float(fpkind) => fpkind.try_get_size_full(alloc, tctx),
            ValTypeID::FixVec(fixvec) => fixvec.try_get_size_full(alloc, tctx),
            ValTypeID::Array(arr) => arr.try_get_size_full(alloc, tctx),
            ValTypeID::Struct(s) => s.try_get_size_full(alloc, tctx),
            ValTypeID::StructAlias(a) => a.try_get_size_full(alloc, tctx),
        }
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        match self {
            ValTypeID::Void | ValTypeID::Func(_) => None,
            ValTypeID::Ptr => PtrType.try_get_align_full(alloc, tctx),
            ValTypeID::Int(bits) => IntType(bits).try_get_align_full(alloc, tctx),
            ValTypeID::Float(fpkind) => fpkind.try_get_align_full(alloc, tctx),
            ValTypeID::FixVec(fixvec) => fixvec.try_get_align_full(alloc, tctx),
            ValTypeID::Array(arr) => arr.try_get_align_full(alloc, tctx),
            ValTypeID::Struct(s) => s.try_get_align_full(alloc, tctx),
            ValTypeID::StructAlias(a) => a.try_get_align_full(alloc, tctx),
        }
    }

    fn format_ir<T: Write>(self, f: &TypeFormatter<T>) -> std::fmt::Result {
        match self {
            ValTypeID::Void => f.write_str("void"),
            ValTypeID::Ptr => f.write_str("ptr"),
            ValTypeID::Int(bits) => write!(f, "i{}", bits),
            ValTypeID::Float(fpkind) => fpkind.format_ir(f),
            ValTypeID::FixVec(fixvec) => fixvec.format_ir(f),
            ValTypeID::Array(a) => a.format_ir(f),
            ValTypeID::Struct(s) => s.format_ir(f),
            ValTypeID::StructAlias(sa) => sa.format_ir(f),
            ValTypeID::Func(func) => func.format_ir(f),
        }
    }
}

impl ValTypeID {
    pub fn is_alive(self, tctx: &TypeContext) -> bool {
        match self {
            ValTypeID::Int(bits) => bits <= 128,
            ValTypeID::FixVec(FixVecType(ScalarType::Int(i), _)) => i <= 128,
            ValTypeID::Array(arr) => arr.try_deref(&tctx.allocs.borrow().arrays).is_some(),
            ValTypeID::Struct(struc) => struc.try_deref(&tctx.allocs.borrow().structs).is_some(),
            ValTypeID::StructAlias(sa) => sa.try_deref(&tctx.allocs.borrow().aliases).is_some(),
            ValTypeID::Func(f) => f.try_deref(&tctx.allocs.borrow().funcs).is_some(),
            _ => true,
        }
    }
}
